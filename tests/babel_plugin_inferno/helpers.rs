//! Helpers mirroring `tests/helpers.js` of babel-plugin-inferno.
//!
//! The plugin runs the way swc runs it: after `resolver` and before the
//! TypeScript strip, `hygiene` and `fixer` passes.
//!
//! babel-plugin-inferno pins the code babel prints. swc prints code differently
//! (indentation, quoted keys, string escapes, pure annotations), so
//! [`assert_js_eq`] compares the two after [`normalize`].

use swc_core::{
    common::{
        BytePos, FileName, LineCol, Mark, SourceFile, SourceMap, comments::SingleThreadedComments,
        sync::Lrc, util::take::Take,
    },
    ecma::{
        ast::*,
        transforms::base::{fixer::fixer, hygiene::hygiene, resolver},
        utils::is_valid_prop_ident,
        visit::{VisitMut, VisitMutWith, visit_mut_pass},
    },
};
use swc_ecma_codegen::{Config, Emitter, text_writer::JsWriter};
use swc_ecma_parser::{
    EsSyntax, Syntax, TsSyntax, parse_file_as_module, parse_file_as_program, parse_file_as_script,
};
use swc_ecma_transforms_typescript::{TsxConfig, tsx, typescript};
use swc_plugin_inferno::{Options, inferno};

pub use swc_core::ecma::ast::noop_pass;

/// The language of the compiled input.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Jsx,
    /// TSX; the types are stripped after the plugin runs.
    Tsx,
}

/// How the compiled input is parsed, like babel's `sourceType`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Module,
    Script,
    /// A module when the input has imports or exports, a script otherwise.
    Unambiguous,
}

/// How [`compile`] compiles its input.
#[derive(Clone, Copy)]
pub struct Setup<'a> {
    pub lang: Lang,
    pub source_type: SourceType,
    /// Plugin options, as written in `.swcrc`.
    pub options: &'a str,
    /// Options of swc's TypeScript transform, as written in `.swcrc`.
    pub typescript: &'a str,
    /// The JSX pragma the TypeScript transform keeps imports of.
    pub jsx_pragma: Option<&'a str>,
}

impl Default for Setup<'_> {
    fn default() -> Self {
        Setup {
            lang: Lang::Jsx,
            source_type: SourceType::Module,
            options: "{}",
            typescript: "{}",
            jsx_pragma: None,
        }
    }
}

/// Generated code, with its source map.
pub struct Compiled {
    pub code: String,
    /// Maps positions in `code` to positions in the input. Lines start from 1,
    /// columns from 0, like in babel's source maps.
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping {
    pub generated: Position,
    pub original: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// Compiles `input`. Returns the generated code, or the reported errors when
/// compilation fails.
pub fn compile(setup: Setup, input: &str) -> Result<String, String> {
    compile_with(setup, input, noop_pass()).map(|compiled| compiled.code)
}

/// Like [`compile`], but runs `before` ahead of the plugin and returns the
/// source map too.
pub fn compile_with(setup: Setup, input: &str, before: impl Pass) -> Result<Compiled, String> {
    let options: Options = serde_json::from_str(setup.options)
        .unwrap_or_else(|err| panic!("invalid plugin options {}: {err}", setup.options));
    let typescript_config: swc_ecma_transforms_typescript::Config =
        serde_json::from_str(setup.typescript)
            .unwrap_or_else(|err| panic!("invalid typescript options {}: {err}", setup.typescript));

    testing::run_test(false, |cm, handler| {
        let comments = SingleThreadedComments::default();
        let is_tsx = setup.lang == Lang::Tsx;
        let fm = cm.new_source_file(
            FileName::Real(if is_tsx { "input.tsx" } else { "input.js" }.into()).into(),
            input.to_string(),
        );
        let syntax = if is_tsx {
            Syntax::Typescript(TsSyntax {
                tsx: true,
                ..Default::default()
            })
        } else {
            Syntax::Es(EsSyntax {
                jsx: true,
                ..Default::default()
            })
        };

        let mut errors = vec![];
        let target = EsVersion::latest();
        let comments_ref = Some(&comments as &dyn swc_core::common::comments::Comments);
        let program = match setup.source_type {
            SourceType::Module => {
                parse_file_as_module(&fm, syntax, target, comments_ref, &mut errors)
                    .map(Program::Module)
            }
            SourceType::Script => {
                parse_file_as_script(&fm, syntax, target, comments_ref, &mut errors)
                    .map(Program::Script)
            }
            SourceType::Unambiguous => {
                parse_file_as_program(&fm, syntax, target, comments_ref, &mut errors)
            }
        }
        .map_err(|err| err.into_diagnostic(handler).emit())?;

        for err in errors {
            err.into_diagnostic(handler).emit();
        }
        if handler.has_errors() {
            return Err(());
        }

        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        let mut program = program.apply((
            resolver(unresolved_mark, top_level_mark, is_tsx),
            before,
            inferno(
                cm.clone(),
                Some(comments.clone()),
                options,
                top_level_mark,
                unresolved_mark,
            ),
        ));

        if handler.has_errors() {
            return Err(());
        }

        if is_tsx {
            program = program.apply((
                tsx(
                    cm.clone(),
                    typescript_config,
                    TsxConfig {
                        pragma: setup.jsx_pragma.map(|pragma| pragma.to_string().into()),
                        pragma_frag: None,
                    },
                    comments.clone(),
                    unresolved_mark,
                    top_level_mark,
                ),
                typescript(typescript_config, unresolved_mark, top_level_mark),
            ));
        }
        let program = program.apply((hygiene(), fixer(Some(&comments))));

        let mut srcmap = vec![];
        let code = print(&cm, &program, Some(&comments), Some(&mut srcmap));
        let mappings = mappings(&cm, &fm, &code, &srcmap);

        Ok(Compiled { code, mappings })
    })
    .map_err(|err| err.to_string())
}

/// Like `transformWith` of babel-plugin-inferno: returns the full generated code.
pub fn transform_with(options: &str, input: &str) -> String {
    compile_ok(
        Setup {
            options,
            ..Default::default()
        },
        input,
    )
}

/// Like `transform` of babel-plugin-inferno: compiles with the default options and
/// drops the import the plugin adds.
pub fn transform(input: &str) -> String {
    strip_inferno_import(&transform_with("{}", input))
}

/// Like `transformTSX` of babel-plugin-inferno: compiles TSX and strips the types.
pub fn transform_tsx(options: &str, input: &str) -> String {
    compile_ok(
        Setup {
            lang: Lang::Tsx,
            options,
            ..Default::default()
        },
        input,
    )
}

/// Compiles `input` and panics with the reported errors when compilation fails.
pub fn compile_ok(setup: Setup, input: &str) -> String {
    compile(setup, input).unwrap_or_else(|err| panic!("failed to compile {input:?}:\n{err}"))
}

/// Removes the first `import ... from "inferno";` line, like `stripInfernoImport`.
pub fn strip_inferno_import(code: &str) -> String {
    let mut stripped = false;

    code.lines()
        .filter(|line| {
            if !stripped && line.starts_with("import") && line.ends_with("\"inferno\";") {
                stripped = true;
                return false;
            }
            true
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Asserts that `input` compiles, with the default options, to `expected`.
#[track_caller]
pub fn assert_transform(input: &str, expected: &str) {
    assert_js_eq(&transform(input), expected);
}

/// Asserts that compiling `input` with the default options fails with an error
/// containing `message`.
#[track_caller]
pub fn assert_transform_error(input: &str, message: &str) {
    assert_compile_error(Setup::default(), input, message);
}

#[track_caller]
pub fn assert_compile_error(setup: Setup, input: &str, message: &str) {
    match compile(setup, input) {
        Ok(code) => {
            panic!("expected {input:?} to fail with {message:?}, but it compiled to:\n{code}")
        }
        Err(err) => assert!(
            err.contains(message),
            "expected {input:?} to fail with {message:?}, but it failed with:\n{err}"
        ),
    }
}

/// Asserts that `code` is plain JavaScript: it parses without the JSX syntax.
#[track_caller]
pub fn assert_valid_js(code: &str) {
    if let Err(err) = parse(code, false) {
        panic!("generated code is not valid JavaScript ({err}):\n{code}");
    }
}

/// Asserts that two pieces of code are the same program after [`normalize`].
#[track_caller]
pub fn assert_js_eq(actual: &str, expected: &str) {
    let normalized_actual = normalize(actual);
    let normalized_expected = normalize(expected);

    assert!(
        normalized_actual == normalized_expected,
        "generated code differs from the expected code\n\n--- actual (normalized) ---\n{normalized_actual}\n--- expected (normalized) ---\n{normalized_expected}\n--- actual ---\n{actual}\n--- expected ---\n{expected}"
    );
}

/// Runs `check` for every case and reports all failing cases at once.
#[track_caller]
pub fn assert_all<T: std::fmt::Debug>(cases: impl IntoIterator<Item = T>, check: impl Fn(&T)) {
    let failures: Vec<String> = cases
        .into_iter()
        .filter_map(|case| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(&case)))
                .err()
                .map(|_| format!("{case:?}"))
        })
        .collect();

    assert!(failures.is_empty(), "failing cases: {failures:#?}");
}

/// Prints `code` in a canonical form, so that code printed by babel and by swc can be
/// compared: comments, parentheses, the raw text of literals and quotes around
/// property names do not matter.
pub fn normalize(code: &str) -> String {
    let (cm, module) = parse(code, true).unwrap_or_else(|err| panic!("{err}:\n{code}"));

    let program = swc_core::common::GLOBALS.set(&Default::default(), || {
        Program::Module(module).apply((visit_mut_pass(Normalizer), fixer(None)))
    });

    print(&cm, &program, None, None)
}

fn parse(code: &str, jsx: bool) -> Result<(Lrc<SourceMap>, Module), String> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Anon.into(), code.to_string());
    let mut errors = vec![];
    let syntax = Syntax::Es(EsSyntax {
        jsx,
        ..Default::default()
    });

    let module = parse_file_as_module(&fm, syntax, EsVersion::latest(), None, &mut errors)
        .map_err(|err| format!("failed to parse: {}", err.kind().msg()))?;

    if let Some(err) = errors.first() {
        return Err(format!("failed to parse: {}", err.kind().msg()));
    }

    Ok((cm, module))
}

fn print(
    cm: &Lrc<SourceMap>,
    program: &Program,
    comments: Option<&SingleThreadedComments>,
    srcmap: Option<&mut Vec<(BytePos, LineCol)>>,
) -> String {
    let mut buf = vec![];

    Emitter {
        cfg: Config::default(),
        cm: cm.clone(),
        wr: Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, srcmap)),
        comments: comments.map(|c| c as _),
    }
    .emit_program(program)
    .unwrap();

    String::from_utf8(buf).unwrap()
}

fn mappings(
    cm: &Lrc<SourceMap>,
    fm: &SourceFile,
    code: &str,
    srcmap: &[(BytePos, LineCol)],
) -> Vec<Mapping> {
    let lines: Vec<&str> = code.lines().collect();

    srcmap
        .iter()
        // Generated code without a position in the input is not mapped
        .filter(|(pos, _)| fm.start_pos <= *pos && *pos <= fm.end_pos)
        .map(|(pos, generated)| {
            let original = cm.lookup_char_pos(*pos);
            let line = lines
                .get(generated.line as usize)
                .copied()
                .unwrap_or_default();

            Mapping {
                generated: Position {
                    line: generated.line as usize + 1,
                    column: utf16_to_char_column(line, generated.col as usize),
                },
                original: Position {
                    line: original.line,
                    column: original.col.0,
                },
            }
        })
        .collect()
}

fn utf16_to_char_column(line: &str, utf16_column: usize) -> usize {
    let mut units = 0;

    for (column, c) in line.chars().enumerate() {
        if units >= utf16_column {
            return column;
        }
        units += c.len_utf16();
    }
    line.chars().count()
}

/// Like `originalPosition` of babel-plugin-inferno: maps the first occurrence of
/// `needle` in the generated code back to the input.
pub fn original_position(compiled: &Compiled, needle: &str) -> Option<Position> {
    let (line, column) = compiled
        .code
        .lines()
        .enumerate()
        .find_map(|(i, line)| {
            line.find(needle)
                .map(|byte| (i + 1, line[..byte].chars().count()))
        })
        .unwrap_or_else(|| panic!("{needle:?} not found in generated code:\n{}", compiled.code));

    // The closest mapping at or before the needle, like the source-map libraries do
    compiled
        .mappings
        .iter()
        .filter(|m| m.generated.line == line && m.generated.column <= column)
        .max_by_key(|m| m.generated.column)
        .map(|m| m.original)
}

struct Normalizer;

impl VisitMut for Normalizer {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        while let Expr::Paren(paren) = expr {
            *expr = *paren.expr.take();
        }

        expr.visit_mut_children_with(self);
    }

    fn visit_mut_prop(&mut self, prop: &mut Prop) {
        if let Prop::Shorthand(ident) = prop {
            *prop = Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(ident.clone().into()),
                value: Box::new(Expr::Ident(ident.clone())),
            });
        }

        prop.visit_mut_children_with(self);
    }

    fn visit_mut_prop_name(&mut self, name: &mut PropName) {
        if let PropName::Str(s) = name
            && let Some(value) = s.value.as_str()
            && is_valid_prop_ident(value)
        {
            *name = PropName::Ident(IdentName::new(value.into(), s.span));
        }

        name.visit_mut_children_with(self);
    }

    fn visit_mut_str(&mut self, s: &mut Str) {
        s.raw = None;
    }

    fn visit_mut_number(&mut self, n: &mut Number) {
        n.raw = None;
    }

    fn visit_mut_big_int(&mut self, n: &mut BigInt) {
        n.raw = None;
    }
}
