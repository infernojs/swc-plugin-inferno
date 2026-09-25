use rustc_hash::FxHashSet;
use swc_core::{
    common::{DUMMY_SP, Spanned, SyntaxContext},
    ecma::ast::*,
    ecma::utils::{ExprFactory, find_pat_ids},
    ecma::visit::{Visit, VisitWith, noop_visit_type},
};

/// The context that swc's `resolver` gives the bindings declared in the module scope.
///
/// A plugin only receives the unresolved mark, so the top-level context is read from the first
/// binding the module declares. Without any binding no module-level hook can be in scope, and
/// `SyntaxContext::empty()` matches no resolved identifier.
pub fn top_level_ctxt(items: &[ModuleItem]) -> SyntaxContext {
    items
        .iter()
        .find_map(|item| match item {
            ModuleItem::ModuleDecl(ModuleDecl::Import(import)) if !import.type_only => import
                .specifiers
                .iter()
                .find_map(|specifier| match specifier {
                    ImportSpecifier::Named(named) if !named.is_type_only => Some(named.local.ctxt),
                    ImportSpecifier::Default(default) => Some(default.local.ctxt),
                    ImportSpecifier::Namespace(namespace) => Some(namespace.local.ctxt),
                    _ => None,
                }),
            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl { decl, .. }))
            | ModuleItem::Stmt(Stmt::Decl(decl)) => decl_ctxt(decl),
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(ExportDefaultDecl {
                decl:
                    DefaultDecl::Fn(FnExpr {
                        ident: Some(ident), ..
                    })
                    | DefaultDecl::Class(ClassExpr {
                        ident: Some(ident), ..
                    }),
                ..
            })) => Some(ident.ctxt),
            _ => None,
        })
        .unwrap_or_default()
}

fn decl_ctxt(decl: &Decl) -> Option<SyntaxContext> {
    match decl {
        Decl::Fn(function) => Some(function.ident.ctxt),
        Decl::Class(class) => Some(class.ident.ctxt),
        Decl::Var(var) => var.decls.iter().find_map(|declarator| {
            find_pat_ids::<_, Ident>(&declarator.name)
                .first()
                .map(|ident| ident.ctxt)
        }),
        _ => None,
    }
}

pub fn is_builtin_hook(name: &str) -> bool {
    matches!(
        name,
        "useState"
            | "useReducer"
            | "useEffect"
            | "useLayoutEffect"
            | "useMemo"
            | "useCallback"
            | "useRef"
            | "useContext"
            | "useImperativeHandle"
            | "useDebugValue"
    )
}

pub fn is_body_arrow_fn(body: &ArrowFunctionBody) -> bool {
    if let ArrowFunctionBody::Expr(body) = body {
        body.is_arrow()
    } else {
        false
    }
}

fn assert_hygiene(e: &Expr) {
    if let Expr::Ident(i) = e {
        debug_assert!(i.ctxt != SyntaxContext::empty(), "`{i}` should be resolved");
    }
}

/// `handle = expr`
pub fn make_assign_expr(handle: Ident, expr: Box<Expr>) -> Expr {
    assert_hygiene(&expr);

    AssignExpr {
        span: expr.span(),
        op: op!("="),
        left: handle.into(),
        right: expr,
    }
    .into()
}

/// `handle = expr;`
pub fn make_assign_stmt(handle: Ident, expr: Box<Expr>) -> Stmt {
    ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(make_assign_expr(handle, expr)),
    }
    .into()
}

/// Whether a statement is a directive like `'use strict'`, which must stay at the start of a
/// module or function body
pub fn is_directive(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::Expr(ExprStmt { expr, .. }) if matches!(**expr, Expr::Lit(Lit::Str(_))))
}

/// The position after the directives at the start of statements
pub fn after_directives(stmts: &[Stmt]) -> usize {
    stmts.iter().take_while(|stmt| is_directive(stmt)).count()
}

/// `var` with the given declarators
pub fn var_decl(decls: Vec<VarDeclarator>) -> Stmt {
    VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Var,
        declare: false,
        decls,
        ..Default::default()
    }
    .into()
}

pub fn make_call_stmt(handle: Ident) -> Stmt {
    ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(make_call_expr(handle)),
    }
    .into()
}

pub fn make_call_expr(handle: Ident) -> Expr {
    CallExpr {
        span: DUMMY_SP,
        callee: handle.as_callee(),
        args: Vec::new(),
        ..Default::default()
    }
    .into()
}

pub fn is_import_or_require(expr: &Expr) -> bool {
    match expr {
        Expr::Call(CallExpr {
            callee: Callee::Expr(expr),
            ..
        }) => {
            if let Expr::Ident(ident) = expr.as_ref() {
                ident.sym.as_ref().starts_with("require")
            } else {
                false
            }
        }
        Expr::Call(CallExpr {
            callee: Callee::Import(_),
            ..
        }) => true,
        _ => false,
    }
}
pub struct UsedInJsx(FxHashSet<Id>);

impl Visit for UsedInJsx {
    noop_visit_type!();

    fn visit_call_expr(&mut self, n: &CallExpr) {
        n.visit_children_with(self);

        if let Callee::Expr(expr) = &n.callee {
            let ident = match expr.as_ref() {
                Expr::Ident(ident) => ident.to_id(),
                Expr::Member(MemberExpr {
                    prop: MemberProp::Ident(ident),
                    ..
                }) => (ident.sym.clone(), SyntaxContext::empty()),
                _ => return,
            };
            if matches!(
                ident.0.as_ref(),
                "createElement" | "jsx" | "jsxDEV" | "jsxs"
            ) && let Some(ExprOrSpread { expr, .. }) = n.args.first()
                && let Expr::Ident(ident) = expr.as_ref()
            {
                self.0.insert(ident.to_id());
            }
        }
    }

    fn visit_jsx_opening_element(&mut self, n: &JSXOpeningElement) {
        if let JSXElementName::Ident(ident) = &n.name {
            self.0.insert(ident.to_id());
        }
        // Components also appear in JSX attribute values
        n.visit_children_with(self);
    }
}

pub fn collect_ident_in_jsx<V: VisitWith<UsedInJsx>>(item: &V) -> FxHashSet<Id> {
    let mut visitor = UsedInJsx(FxHashSet::default());
    item.visit_with(&mut visitor);
    visitor.0
}
