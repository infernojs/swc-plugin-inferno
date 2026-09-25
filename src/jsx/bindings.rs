//! Program scope bindings, like the `scope.hasBinding` and `scope.generateUidIdentifier`
//! calls of babel-plugin-inferno.

use super::{HELPER_COUNT, Helper};
use rustc_hash::FxHashSet;
use swc_core::{
    atoms::Atom,
    common::SyntaxContext,
    ecma::{
        ast::*,
        utils::for_each_binding_ident,
        visit::{Visit, VisitWith, noop_visit_type},
    },
};

/// The context of each helper that the program declares itself, or `None`
pub(super) type HelperBindings = [Option<SyntaxContext>; HELPER_COUNT];

/// The helpers declared in the program scope of a module: imports, top-level declarations and
/// hoisted `var`s.
pub(super) fn module_bindings(module: &Module) -> HelperBindings {
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

/// The helpers declared in the program scope of a script
pub(super) fn script_bindings(script: &Script) -> HelperBindings {
    let mut collector = BindingCollector::default();

    for stmt in &script.body {
        collector.stmt(stmt);
    }

    collector.bindings
}

#[derive(Default)]
struct BindingCollector {
    bindings: HelperBindings,
}

impl BindingCollector {
    fn add(&mut self, ident: &Ident) {
        if let Some(helper) = Helper::from_name(&ident.sym) {
            // The first declaration wins
            self.bindings[helper as usize].get_or_insert(ident.ctxt);
        }
    }

    fn add_pat(&mut self, pat: &Pat) {
        for_each_binding_ident(pat, |binding| self.add(&binding.id));
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

    let mut uid = Atom::from(format!("_{name}"));
    for i in 2.. {
        if !used.contains(&uid) {
            break;
        }
        uid = format!("_{name}{i}").into();
    }
    uid
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

    // babel prefixes a reserved word with `_` here, which generate_uid strips again
    camel_cased
}
