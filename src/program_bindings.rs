//! Bindings declared in the program scope: imports, top-level declarations and hoisted `var`s.

use swc_core::ecma::{
    ast::*,
    utils::for_each_binding_ident,
    visit::{Visit, VisitWith, noop_visit_type},
};

/// Calls `f` with each binding declared in the program scope of a module, in source order
pub(crate) fn for_each_module_binding(items: &[ModuleItem], f: impl FnMut(&Ident)) {
    let mut bindings = Bindings(f);

    for item in items {
        match item {
            ModuleItem::ModuleDecl(decl) => bindings.module_decl(decl),
            ModuleItem::Stmt(stmt) => bindings.stmt(stmt),
            #[cfg(swc_ast_unknown)]
            _ => {}
        }
    }
}

/// Calls `f` with each binding declared in the program scope of a script, in source order
pub(crate) fn for_each_script_binding(stmts: &[Stmt], f: impl FnMut(&Ident)) {
    let mut bindings = Bindings(f);

    for stmt in stmts {
        bindings.stmt(stmt);
    }
}

struct Bindings<F>(F);

impl<F: FnMut(&Ident)> Bindings<F> {
    fn add(&mut self, ident: &Ident) {
        (self.0)(ident);
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
                            self.add(&named.local);
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
            // `declare global { ... }` augments the global scope and declares no binding
            Decl::TsModule(module) if !module.global => {
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

struct HoistedVars<'a, F>(&'a mut Bindings<F>);

impl<F: FnMut(&Ident)> Visit for HoistedVars<'_, F> {
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
