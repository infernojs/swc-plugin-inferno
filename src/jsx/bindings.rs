//! Program scope bindings, like the `scope.hasBinding` and `scope.generateUidIdentifier`
//! calls of babel-plugin-inferno.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{
    atoms::Atom,
    common::SyntaxContext,
    ecma::{
        ast::*,
        utils::find_pat_ids,
        visit::{Visit, VisitWith, noop_visit_type},
    },
};

/// Bindings declared in the program scope of a module: imports, top-level declarations and
/// hoisted `var`s.
pub(super) fn module_bindings(module: &Module) -> FxHashMap<Atom, SyntaxContext> {
    let mut collector = BindingCollector::default();

    for item in &module.body {
        match item {
            ModuleItem::ModuleDecl(decl) => collector.module_decl(decl),
            ModuleItem::Stmt(stmt) => collector.stmt(stmt),
            #[cfg(swc_ast_unknown)]
            _ => {}
        }
    }

    collector.bindings
}

/// Bindings declared in the program scope of a script
pub(super) fn script_bindings(script: &Script) -> FxHashMap<Atom, SyntaxContext> {
    let mut collector = BindingCollector::default();

    for stmt in &script.body {
        collector.stmt(stmt);
    }

    collector.bindings
}

#[derive(Default)]
struct BindingCollector {
    bindings: FxHashMap<Atom, SyntaxContext>,
}

impl BindingCollector {
    fn add(&mut self, ident: &Ident) {
        self.bindings.entry(ident.sym.clone()).or_insert(ident.ctxt);
    }

    fn add_pat(&mut self, pat: &Pat) {
        for ident in find_pat_ids::<_, Ident>(pat) {
            self.add(&ident);
        }
    }

    fn module_decl(&mut self, decl: &ModuleDecl) {
        match decl {
            ModuleDecl::Import(import) if !import.type_only => {
                for specifier in &import.specifiers {
                    match specifier {
                        ImportSpecifier::Named(named) if !named.is_type_only => {
                            self.add(&named.local)
                        }
                        ImportSpecifier::Default(default) => self.add(&default.local),
                        ImportSpecifier::Namespace(namespace) => self.add(&namespace.local),
                        _ => {}
                    }
                }
            }
            ModuleDecl::ExportDecl(export) => self.decl(&export.decl),
            ModuleDecl::ExportDefaultDecl(export) => match &export.decl {
                DefaultDecl::Class(ClassExpr {
                    ident: Some(ident), ..
                })
                | DefaultDecl::Fn(FnExpr {
                    ident: Some(ident), ..
                }) => self.add(ident),
                _ => {}
            },
            ModuleDecl::TsImportEquals(import) if !import.is_type_only => self.add(&import.id),
            _ => {}
        }
    }

    fn decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Class(class) => self.add(&class.ident),
            Decl::Fn(function) => self.add(&function.ident),
            Decl::Var(var) => {
                for declarator in &var.decls {
                    self.add_pat(&declarator.name);
                }
            }
            Decl::Using(using) => {
                for declarator in &using.decls {
                    self.add_pat(&declarator.name);
                }
            }
            Decl::TsEnum(ts_enum) => self.add(&ts_enum.id),
            Decl::TsModule(module) => {
                if let TsModuleName::Ident(ident) = &module.id {
                    self.add(ident);
                }
            }
            _ => {}
        }
    }

    fn stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Decl(decl) = stmt {
            self.decl(decl);
        } else {
            // `var` declarations in blocks are hoisted to the program scope
            stmt.visit_with(&mut HoistedVars(self));
        }
    }
}

struct HoistedVars<'a>(&'a mut BindingCollector);

impl Visit for HoistedVars<'_> {
    noop_visit_type!();

    fn visit_var_decl(&mut self, var: &VarDecl) {
        if var.kind == VarDeclKind::Var {
            for declarator in &var.decls {
                self.0.add_pat(&declarator.name);
            }
        }
    }

    // Expressions only declare variables inside functions and classes
    fn visit_expr(&mut self, _: &Expr) {}

    fn visit_function(&mut self, _: &Function) {}

    fn visit_class(&mut self, _: &Class) {}
}

/// Every identifier name used in a script
pub(super) fn used_names(script: &Script) -> FxHashSet<Atom> {
    struct Names(FxHashSet<Atom>);

    impl Visit for Names {
        noop_visit_type!();

        fn visit_ident(&mut self, ident: &Ident) {
            self.0.insert(ident.sym.clone());
        }
    }

    let mut names = Names(FxHashSet::default());
    script.visit_with(&mut names);
    names.0
}

/// A name that is not used in the program, like `scope.generateUidIdentifier(name)` in babel:
/// `inferno` becomes `_inferno`, or `_inferno2` when that is taken.
pub(super) fn generate_uid(name: &str, used: &FxHashSet<Atom>) -> Atom {
    let name = to_identifier(name);
    let name = name
        .trim_start_matches('_')
        .trim_end_matches(|c: char| c.is_ascii_digit());

    (1..)
        .map(|i| {
            if i > 1 {
                format!("_{name}{i}")
            } else {
                format!("_{name}")
            }
        })
        .map(Atom::from)
        .find(|uid| !used.contains(uid))
        .unwrap()
}

/// `toIdentifier` of `@babel/types`: `inferno-compat` becomes `infernoCompat`
fn to_identifier(input: &str) -> String {
    let name: String = input
        .chars()
        .map(|c| if Ident::is_valid_continue(c) { c } else { '-' })
        .collect();
    let name = name.trim_start_matches(|c: char| c == '-' || c.is_ascii_digit());

    let mut camel_cased = String::with_capacity(name.len());
    let mut upper_next = false;
    for c in name.chars() {
        if c == '-' {
            upper_next = true;
        } else if upper_next {
            camel_cased.extend(c.to_uppercase());
            upper_next = false;
        } else {
            camel_cased.push(c);
        }
    }

    if Ident::verify_symbol(&camel_cased).is_err() {
        camel_cased.insert(0, '_');
    }
    camel_cased
}
