use std::{fmt::Write, mem};

use base64::prelude::{BASE64_STANDARD, Engine};
use sha1::{Digest, Sha1};
use swc_core::common::util::take::Take;
use swc_core::common::{DUMMY_SP, SourceMapper, Spanned, SyntaxContext};
use swc_core::ecma::ast::*;
use swc_core::ecma::utils::{ExprFactory, private_ident, quote_ident, quote_str};
use swc_core::ecma::visit::{
    Visit, VisitMut, VisitMutWith, VisitWith, noop_visit_mut_type, noop_visit_type,
};

use super::util::{after_directives, is_builtin_hook, make_call_expr, make_call_stmt, var_decl};
use swc_core::atoms::Atom;

/// The name of the variables that hold signature handles
const SIGNATURE_HANDLE: &str = "_s";

// function that use hooks
struct HookSig {
    handle: Ident,
    // need to add an extra register, or already inlined
    hooks: Vec<Hook>,
}

impl HookSig {
    fn new(hooks: Vec<Hook>) -> Self {
        HookSig {
            handle: private_ident!(SIGNATURE_HANDLE),
            hooks,
        }
    }
}

struct Hook {
    callee: HookCall,
    key: String,
}

// we only consider two kinds of callee as hook call
enum HookCall {
    Ident(Ident),
    Member(Box<Expr>, IdentName), // for obj and prop
}

impl HookCall {
    fn name(&self) -> &Atom {
        match self {
            HookCall::Ident(ident) => &ident.sym,
            HookCall::Member(_, prop) => &prop.sym,
        }
    }
}

/// `function() { return [hooks] }`, which gives the refresh runtime the custom hooks to compare
fn hooks_closure(hooks: Vec<HookCall>) -> Function {
    let elems = hooks
        .into_iter()
        .map(|hook| {
            Some(
                match hook {
                    HookCall::Ident(ident) => Expr::from(ident),
                    HookCall::Member(obj, prop) => MemberExpr {
                        span: DUMMY_SP,
                        obj,
                        prop: MemberProp::Ident(prop),
                    }
                    .into(),
                }
                .as_arg(),
            )
        })
        .collect();

    Function {
        body: Some(FunctionBody {
            span: DUMMY_SP,
            stmts: vec![Stmt::Return(ReturnStmt {
                span: DUMMY_SP,
                arg: Some(Box::new(Expr::Array(ArrayLit {
                    span: DUMMY_SP,
                    elems,
                }))),
            })],
        }),
        ..Default::default()
    }
}
pub struct HookRegister<'a, S: SourceMapper> {
    /// The function that creates a signature handle
    pub refresh_sig: &'a Atom,
    pub emit_full_signatures: bool,
    pub ident: Vec<Ident>,
    pub extra_stmt: Vec<Stmt>,
    pub current_scope: Vec<SyntaxContext>,
    pub cm: &'a S,
    pub should_reset: bool,
}

impl<S: SourceMapper> HookRegister<'_, S> {
    /// `var _s = $RefreshSig$(), ...` for the signature handles used in the scope
    pub fn gen_hook_handle(&mut self) -> Stmt {
        var_decl(
            self.ident
                .take()
                .into_iter()
                .map(|id| VarDeclarator {
                    span: DUMMY_SP,
                    name: id.into(),
                    init: Some(Box::new(make_call_expr(
                        quote_ident!(self.refresh_sig.clone()).into(),
                    ))),
                    definite: false,
                })
                .collect(),
        )
    }

    /// The signature key of the hooks: their names and keys, hashed unless
    /// `emitFullSignatures` is set
    fn signature_key(&self, hooks: &[Hook]) -> String {
        let key = hooks
            .iter()
            .map(|hook| format!("{}{{{}}}", hook.callee.name(), hook.key))
            .collect::<Vec<_>>()
            .join("\n");

        if self.emit_full_signatures {
            key
        } else {
            let mut hasher = Sha1::new();
            hasher.update(key);
            BASE64_STANDARD.encode(hasher.finalize())
        }
    }

    /// The custom hooks that the refresh runtime can compare, and whether the component has to
    /// be remounted because some custom hook is out of scope
    fn custom_hooks_in_scope(&self, hooks: Vec<Hook>) -> (Vec<HookCall>, bool) {
        let mut should_reset = self.should_reset;
        let mut in_scope = Vec::new();

        for Hook { callee, .. } in hooks {
            let (ident, is_custom) = match &callee {
                HookCall::Ident(ident) => (Some(ident), !is_builtin_hook(&ident.sym)),
                HookCall::Member(obj, prop) => (
                    obj.as_ident(),
                    !is_builtin_hook(&prop.sym)
                        && matches!(&**obj, Expr::Ident(obj) if obj.sym != "React"),
                ),
            };
            if !is_custom {
                continue;
            }
            if ident.is_some_and(|ident| self.current_scope.contains(&ident.ctxt)) {
                in_scope.push(callee);
            } else {
                // We don't have anything to put in the array because Hook is out of scope.
                // Since it could potentially have been edited, remount the component.
                should_reset = true;
            }
        }

        (in_scope, should_reset)
    }

    // The second call is around the function itself. This is used to associate a
    // type with a signature.
    // Unlike with $RefreshReg$, this needs to work for nested declarations too.
    fn wrap_with_register(&self, handle: Ident, func: Expr, hooks: Vec<Hook>) -> Expr {
        let key = self.signature_key(&hooks);
        let (custom_hooks, should_reset) = self.custom_hooks_in_scope(hooks);
        let mut args = vec![func.as_arg(), quote_str!(key).as_arg()];

        if should_reset || !custom_hooks.is_empty() {
            args.push(should_reset.as_arg());
        }
        if !custom_hooks.is_empty() {
            args.push(hooks_closure(custom_hooks).as_arg());
        }

        CallExpr {
            span: DUMMY_SP,
            callee: handle.as_callee(),
            args,
            ..Default::default()
        }
        .into()
    }

    // Function bodies no longer carry their own `SyntaxContext`, so the scope is
    // passed in by the caller: it is `Function.ctxt` / `ArrowExpr.ctxt` for a
    // function body, and `BlockStmt.ctxt` for a plain block.
    fn visit_mut_scoped_stmts(&mut self, stmts: &mut Vec<Stmt>, ctxt: SyntaxContext) {
        let old_ident = self.ident.take();
        let old_stmts = self.extra_stmt.take();

        self.current_scope.push(ctxt);

        let stmt_count = stmts.len();
        let old_stmt = mem::replace(stmts, Vec::with_capacity(stmt_count));

        for mut stmt in old_stmt {
            stmt.visit_mut_children_with(self);

            stmts.push(stmt);
            stmts.append(&mut self.extra_stmt);
        }

        if !self.ident.is_empty() {
            stmts.insert(after_directives(stmts), self.gen_hook_handle());
        }

        self.current_scope.pop();
        self.ident = old_ident;
        self.extra_stmt = old_stmts;
    }

    fn visit_mut_arrow_body(&mut self, body: &mut ArrowFunctionBody, ctxt: SyntaxContext) {
        match body {
            ArrowFunctionBody::FunctionBody(b) => self.visit_mut_scoped_stmts(&mut b.stmts, ctxt),
            ArrowFunctionBody::Expr(e) => e.visit_mut_with(self),
        }
    }

    fn gen_hook_register_stmt(&mut self, ident: Ident, sig: HookSig) {
        self.ident.push(sig.handle.clone());
        self.extra_stmt.push(
            ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(self.wrap_with_register(sig.handle, ident.into(), sig.hooks)),
            }
            .into(),
        )
    }
}

impl<S: SourceMapper> VisitMut for HookRegister<'_, S> {
    noop_visit_mut_type!();

    fn visit_mut_block_stmt(&mut self, b: &mut BlockStmt) {
        let ctxt = b.ctxt;
        self.visit_mut_scoped_stmts(&mut b.stmts, ctxt);
    }

    fn visit_mut_function(&mut self, f: &mut Function) {
        f.decorators.visit_mut_with(self);
        f.params.visit_mut_with(self);

        let ctxt = f.ctxt;
        if let Some(body) = &mut f.body {
            self.visit_mut_scoped_stmts(&mut body.stmts, ctxt);
        }
    }

    fn visit_mut_constructor(&mut self, c: &mut Constructor) {
        c.params.visit_mut_with(self);

        if let Some(body) = &mut c.body {
            self.visit_mut_scoped_stmts(&mut body.stmts, c.ctxt);
        }
    }

    fn visit_mut_arrow_expr(&mut self, a: &mut ArrowExpr) {
        a.params.visit_mut_with(self);

        let ctxt = a.ctxt;
        self.visit_mut_arrow_body(&mut a.body, ctxt);
    }

    fn visit_mut_expr(&mut self, e: &mut Expr) {
        e.visit_mut_children_with(self);

        match e {
            Expr::Fn(FnExpr {
                ident: fn_ident,
                function: f,
            }) => {
                if let Some(body) = &mut f.body {
                    let self_ids: Vec<Id> = fn_ident.iter().map(Ident::to_id).collect();
                    let sig = collect_hooks(&mut body.stmts, self.cm, &self_ids);

                    if let Some(HookSig { handle, hooks }) = sig {
                        self.ident.push(handle.clone());
                        *e = self.wrap_with_register(handle, e.take(), hooks);
                    }
                }
            }
            Expr::Arrow(ArrowExpr { body, .. }) => {
                let sig = collect_hooks_arrow(body, self.cm, &[]);

                if let Some(HookSig { handle, hooks }) = sig {
                    self.ident.push(handle.clone());
                    *e = self.wrap_with_register(handle, e.take(), hooks);
                }
            }
            _ => (),
        }
    }

    fn visit_mut_var_decl(&mut self, n: &mut VarDecl) {
        // we don't want visit_mut_expr to mess up with function name inference
        // so intercept it here

        for decl in n.decls.iter_mut() {
            if let VarDeclarator {
                // it doesn't quite make sense for other Pat to appear here
                name: Pat::Ident(id),
                init: Some(init),
                ..
            } = decl
            {
                match init.as_mut() {
                    Expr::Fn(FnExpr {
                        ident: fn_ident,
                        function: f,
                    }) => {
                        let ctxt = f.ctxt;
                        if let Some(body) = &mut f.body {
                            self.visit_mut_scoped_stmts(&mut body.stmts, ctxt);
                            let mut self_ids = vec![id.to_id()];
                            self_ids.extend(fn_ident.iter().map(Ident::to_id));
                            if let Some(sig) = collect_hooks(&mut body.stmts, self.cm, &self_ids) {
                                self.gen_hook_register_stmt(Ident::from(&*id), sig);
                            }
                        } else {
                            self.visit_mut_expr(init);
                        }
                    }
                    Expr::Arrow(ArrowExpr { body, ctxt, .. }) => {
                        let ctxt = *ctxt;
                        self.visit_mut_arrow_body(body, ctxt);
                        if let Some(sig) = collect_hooks_arrow(body, self.cm, &[id.to_id()]) {
                            self.gen_hook_register_stmt(Ident::from(&*id), sig);
                        }
                    }
                    _ => self.visit_mut_expr(init),
                }
            } else {
                decl.visit_mut_children_with(self)
            }
        }
    }

    fn visit_mut_default_decl(&mut self, d: &mut DefaultDecl) {
        d.visit_mut_children_with(self);

        // only when expr has ident
        if let DefaultDecl::Fn(FnExpr {
            ident: Some(ident),
            function: f,
        }) = d
            && let Some(body) = &mut f.body
            && let Some(sig) = collect_hooks(&mut body.stmts, self.cm, &[ident.to_id()])
        {
            self.gen_hook_register_stmt(ident.clone(), sig);
        }
    }

    fn visit_mut_fn_decl(&mut self, f: &mut FnDecl) {
        f.visit_mut_children_with(self);

        if let Some(body) = &mut f.function.body
            && let Some(sig) = collect_hooks(&mut body.stmts, self.cm, &[f.ident.to_id()])
        {
            self.gen_hook_register_stmt(f.ident.clone(), sig);
        }
    }
}

/// Collects the hooks called in a function body. `self_ids` are the names of the function, whose
/// recursive calls are not hooks of their own.
fn collect_hooks<S: SourceMapper>(
    stmts: &mut Vec<Stmt>,
    cm: &S,
    self_ids: &[Id],
) -> Option<HookSig> {
    let mut hook = HookCollector {
        state: Vec::new(),
        cm,
        self_ids,
    };

    stmts.visit_with(&mut hook);

    if !hook.state.is_empty() {
        let sig = HookSig::new(hook.state);
        stmts.insert(after_directives(stmts), make_call_stmt(sig.handle.clone()));

        Some(sig)
    } else {
        None
    }
}

fn collect_hooks_arrow<S: SourceMapper>(
    body: &mut ArrowFunctionBody,
    cm: &S,
    self_ids: &[Id],
) -> Option<HookSig> {
    match body {
        ArrowFunctionBody::FunctionBody(block) => collect_hooks(&mut block.stmts, cm, self_ids),
        ArrowFunctionBody::Expr(expr) => {
            let mut hook = HookCollector {
                state: Vec::new(),
                cm,
                self_ids,
            };

            expr.visit_with(&mut hook);

            if !hook.state.is_empty() {
                let sig = HookSig::new(hook.state);
                *body = ArrowFunctionBody::FunctionBody(FunctionBody {
                    span: expr.span(),
                    stmts: vec![
                        make_call_stmt(sig.handle.clone()),
                        Stmt::Return(ReturnStmt {
                            span: expr.span(),
                            arg: Some(Box::new(expr.as_mut().take())),
                        }),
                    ],
                });
                Some(sig)
            } else {
                None
            }
        }
    }
}

struct HookCollector<'a, S: SourceMapper> {
    state: Vec<Hook>,
    cm: &'a S,
    self_ids: &'a [Id],
}

fn is_hook_like(s: &str) -> bool {
    s.strip_prefix("use")
        .and_then(|rest| rest.chars().next())
        .is_some_and(char::is_uppercase)
}

impl<S: SourceMapper> HookCollector<'_, S> {
    fn is_self_call(&self, ident: &Ident) -> bool {
        self.self_ids
            .iter()
            .any(|(sym, ctxt)| *sym == ident.sym && *ctxt == ident.ctxt)
    }

    fn get_hook_from_call_expr(&self, expr: &CallExpr, lhs: Option<&Pat>) -> Option<Hook> {
        let Callee::Expr(callee) = &expr.callee else {
            return None;
        };
        let (name, callee) = match &**callee {
            // A recursive call is not listed: a hook in its own hooks array makes the refresh
            // runtime recurse forever while computing the signature key
            Expr::Ident(ident) if is_hook_like(&ident.sym) && !self.is_self_call(ident) => {
                (&ident.sym, HookCall::Ident(ident.clone()))
            }
            // hook cannot be used in class, so we're fine without SuperProp
            Expr::Member(MemberExpr {
                obj,
                prop: MemberProp::Ident(prop),
                ..
            }) if is_hook_like(&prop.sym) => {
                (&prop.sym, HookCall::Member(obj.clone(), prop.clone()))
            }
            _ => return None,
        };
        let mut key = lhs
            .map(|lhs| self.cm.span_to_snippet(lhs.span()).unwrap_or_default())
            .unwrap_or_default();
        // Some built-in Hooks reset on edits to their initial state argument
        let initial_state = match &**name {
            "useState" => expr.args.first(),
            "useReducer" => expr.args.get(1),
            _ => None,
        };
        if let Some(initial_state) = initial_state {
            let _ = write!(
                key,
                "({})",
                self.cm
                    .span_to_snippet(initial_state.span())
                    .unwrap_or_default()
            );
        }

        Some(Hook { callee, key })
    }

    fn get_hook_from_expr(&self, expr: &Expr, lhs: Option<&Pat>) -> Option<Hook> {
        if let Expr::Call(call) = expr {
            self.get_hook_from_call_expr(call, lhs)
        } else {
            None
        }
    }
}

impl<S: SourceMapper> Visit for HookCollector<'_, S> {
    noop_visit_type!();

    fn visit_arrow_function_body(&mut self, _: &ArrowFunctionBody) {}

    fn visit_block_stmt(&mut self, _: &BlockStmt) {}

    fn visit_function_body(&mut self, _: &FunctionBody) {}

    fn visit_expr(&mut self, expr: &Expr) {
        expr.visit_children_with(self);

        if let Expr::Call(call) = expr
            && let Some(hook) = self.get_hook_from_call_expr(call, None)
        {
            self.state.push(hook)
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(ExprStmt { expr, .. }) => {
                if let Some(hook) = self.get_hook_from_expr(expr, None) {
                    self.state.push(hook)
                } else {
                    stmt.visit_children_with(self)
                }
            }
            Stmt::Decl(Decl::Var(var_decl)) => {
                for decl in &var_decl.decls {
                    match decl
                        .init
                        .as_deref()
                        .and_then(|init| self.get_hook_from_expr(init, Some(&decl.name)))
                    {
                        Some(hook) => self.state.push(hook),
                        None => decl.visit_children_with(self),
                    }
                }
            }
            Stmt::Return(ReturnStmt { arg: Some(arg), .. }) => {
                if let Some(hook) = self.get_hook_from_expr(arg.as_ref(), None) {
                    self.state.push(hook)
                } else {
                    stmt.visit_children_with(self)
                }
            }
            _ => stmt.visit_children_with(self),
        }
    }
}
