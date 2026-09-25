use self::{
    hook::HookRegister,
    util::{
        collect_ident_in_jsx, is_body_arrow_fn, is_import_or_require, make_assign_stmt,
        top_level_ctxt,
    },
};
use rustc_hash::FxHashSet;
use std::borrow::Cow;
use swc_core::atoms::Atom;
use swc_core::ecma::visit::visit_mut_pass;
use swc_core::{
    common::{
        BytePos, DUMMY_SP, SourceMapper, Span, Spanned, SyntaxContext,
        comments::{Comment, Comments},
        sync::Lrc,
        util::take::Take,
    },
    ecma::ast::*,
    ecma::utils::{ExprFactory, private_ident, quote_ident, quote_str},
    ecma::visit::{Visit, VisitMut, VisitMutWith},
};

pub mod options;
use options::RefreshOptions;
mod hook;
mod util;

#[cfg(test)]
mod tests;

struct Hoc {
    insert: bool,
    reg: Vec<(Ident, Id)>,
    hook: Option<HocHook>,
}
struct HocHook {
    callee: Callee,
    rest_arg: Vec<ExprOrSpread>,
}
enum Persist {
    Hoc(Hoc),
    Component(Ident),
    None,
}
/// Whether a name looks like a component: it starts with a capital letter
fn is_componentish(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_uppercase())
}

fn get_persistent_id(ident: &Ident) -> Persist {
    if is_componentish(&ident.sym) {
        debug_assert!(
            ident.ctxt != SyntaxContext::empty(),
            "`{ident}` should be resolved"
        );
        Persist::Component(ident.clone())
    } else {
        Persist::None
    }
}

/// Fast refresh registrations, like `react-refresh/babel`
/// (<https://github.com/facebook/react/blob/main/packages/react-refresh/src/ReactFreshBabelPlugin.js>).
///
/// It must run after swc's `resolver`. `cm` is used to read the source text of hook calls: pass
/// the program's `SourceMap`, or the plugin metadata's source map proxy.
pub fn refresh<C, S>(options: RefreshOptions, cm: Lrc<S>, comments: Option<C>) -> impl Pass
where
    C: Comments,
    S: SourceMapper,
{
    visit_mut_pass(Refresh {
        refresh_reg: options.refresh_reg.into(),
        refresh_sig: options.refresh_sig.into(),
        emit_full_signatures: options.emit_full_signatures,
        cm,
        comments,
        should_reset: false,
    })
}

/// Whether `get_persistent_id_from_possible_hoc` registers `call`: a call of a name or a member
/// expression whose first argument is a function, a component, or such a call.
///
/// Checking first avoids creating registration handles (a host call each in the wasm plugin) for
/// calls that are not registered.
fn is_possible_hoc(call: &CallExpr) -> bool {
    let (Some(first), Callee::Expr(callee)) = (call.args.first(), &call.callee) else {
        return false;
    };
    if !matches!(&**callee, Expr::Ident(_) | Expr::Member(_)) {
        return false;
    }
    match &*first.expr {
        Expr::Call(inner) => is_possible_hoc(inner),
        Expr::Fn(_) | Expr::Arrow(_) => true,
        Expr::Ident(ident) => is_componentish(&ident.sym),
        _ => false,
    }
}

struct Refresh<C: Comments, S: SourceMapper> {
    refresh_reg: Atom,
    refresh_sig: Atom,
    emit_full_signatures: bool,
    cm: Lrc<S>,
    should_reset: bool,
    comments: Option<C>,
}

impl<C: Comments, S: SourceMapper> Refresh<C, S> {
    fn get_persistent_id_from_var_decl(
        &self,
        var_decl: &mut VarDecl,
        used_in_jsx: &FxHashSet<Id>,
        hook_reg: &mut HookRegister,
    ) -> Persist {
        // We only handle the case when a single variable is declared
        if let [
            VarDeclarator {
                name: Pat::Ident(binding),
                init: Some(init_expr),
                ..
            },
        ] = var_decl.decls.as_mut_slice()
        {
            if used_in_jsx.contains(&binding.to_id()) && !is_import_or_require(init_expr) {
                match init_expr.as_ref() {
                    // TaggedTpl is for something like styled.div`...`
                    Expr::Arrow(_) | Expr::Fn(_) | Expr::TaggedTpl(_) | Expr::Call(_) => {
                        return Persist::Component(Ident::from(&*binding));
                    }
                    _ => (),
                }
            }

            if let Persist::Component(persistent_id) = get_persistent_id(&Ident::from(&*binding)) {
                return match init_expr.as_mut() {
                    Expr::Fn(_) => Persist::Component(persistent_id),
                    Expr::Arrow(ArrowExpr { body, .. }) => {
                        // Ignore complex function expressions like
                        // let Foo = () => () => {}
                        if is_body_arrow_fn(body) {
                            Persist::None
                        } else {
                            Persist::Component(persistent_id)
                        }
                    }
                    // Maybe a HOC.
                    Expr::Call(call_expr) if is_possible_hoc(call_expr) => {
                        let res = self.get_persistent_id_from_possible_hoc(
                            call_expr,
                            vec![(private_ident!("_c"), persistent_id.to_id())],
                            hook_reg,
                        );
                        if let Persist::Hoc(Hoc {
                            insert,
                            reg,
                            hook: Some(hook),
                        }) = res
                        {
                            make_hook_reg(init_expr.as_mut(), hook);
                            Persist::Hoc(Hoc {
                                insert,
                                reg,
                                hook: None,
                            })
                        } else {
                            res
                        }
                    }
                    _ => Persist::None,
                };
            }
        }
        Persist::None
    }

    fn get_persistent_id_from_possible_hoc(
        &self,
        call_expr: &mut CallExpr,
        mut reg: Vec<(Ident, Id)>,
        hook_reg: &mut HookRegister,
    ) -> Persist {
        let [first, ..] = call_expr.args.as_mut_slice() else {
            return Persist::None;
        };
        let first_arg = &mut first.expr;
        let Callee::Expr(callee) = &call_expr.callee else {
            return Persist::None;
        };
        let hoc_name: Cow<'_, str> = match callee.as_ref() {
            Expr::Ident(fn_name) => Cow::Borrowed(fn_name.sym.as_ref()),
            // original react implement use `getSource` so we just follow them
            Expr::Member(member) => {
                Cow::Owned(self.cm.span_to_snippet(member.span).unwrap_or_default())
            }
            _ => return Persist::None,
        };
        let Some((_, (reg_name, _))) = reg.last() else {
            return Persist::None;
        };
        let reg_str = (
            format!("{reg_name}${hoc_name}").into(),
            SyntaxContext::empty(),
        );
        match first_arg.as_mut() {
            Expr::Call(expr) => {
                let reg_ident = private_ident!("_c");
                reg.push((reg_ident.clone(), reg_str));
                if let Persist::Hoc(hoc) =
                    self.get_persistent_id_from_possible_hoc(expr, reg, hook_reg)
                {
                    let mut first = first_arg.take();
                    if let Some(HocHook { callee, rest_arg }) = &hoc.hook {
                        let span = first.span();
                        let mut args = Vec::with_capacity(1 + rest_arg.len());
                        args.push(first.as_arg());
                        args.extend(rest_arg.iter().cloned());
                        first = CallExpr {
                            span,
                            callee: callee.clone(),
                            args,
                            ..Default::default()
                        }
                        .into()
                    }
                    **first_arg = make_assign_stmt(reg_ident, first);

                    Persist::Hoc(hoc)
                } else {
                    Persist::None
                }
            }
            Expr::Fn(_) | Expr::Arrow(_) => {
                let reg_ident = private_ident!("_c");
                let mut first = first_arg.take();
                first.visit_mut_with(hook_reg);
                let hook = if let Expr::Call(call) = first.as_ref() {
                    let res = Some(HocHook {
                        callee: call.callee.clone(),
                        rest_arg: call.args.get(1..).unwrap_or_default().to_vec(),
                    });
                    **first_arg = make_assign_stmt(reg_ident.clone(), first);
                    res
                } else {
                    **first_arg = make_assign_stmt(reg_ident.clone(), first);
                    None
                };
                reg.push((reg_ident, reg_str));
                Persist::Hoc(Hoc {
                    reg,
                    insert: true,
                    hook,
                })
            }
            // export default hoc(Foo)
            // const X = hoc(Foo)
            Expr::Ident(ident) => {
                if let Persist::Component(_) = get_persistent_id(ident) {
                    Persist::Hoc(Hoc {
                        reg,
                        insert: true,
                        hook: None,
                    })
                } else {
                    Persist::None
                }
            }
            _ => Persist::None,
        }
    }
}

/// A comment containing this resets the state of every component in the file on each edit
const REFRESH_RESET: &str = "@refresh reset";

fn has_refresh_reset(comments: &[Comment]) -> bool {
    comments
        .iter()
        .any(|comment| comment.text.contains(REFRESH_RESET))
}

/// We let user do /* @refresh reset */ to reset state in the whole file.
impl<C, S> Visit for Refresh<C, S>
where
    C: Comments,
    S: SourceMapper,
{
    fn visit_span(&mut self, n: &Span) {
        if self.should_reset {
            return;
        }
        let Some(comments) = &self.comments else {
            return;
        };
        // `has_*` is a cheap check; `with_*` takes the comments and puts them back
        let leading =
            |pos| comments.has_leading(pos) && comments.with_leading(pos, has_refresh_reset);
        let trailing =
            |pos| comments.has_trailing(pos) && comments.with_trailing(pos, has_refresh_reset);

        self.should_reset = (!n.hi.is_dummy() && leading(n.hi - BytePos(1)))
            || leading(n.lo)
            || trailing(n.lo)
            // A comment after code on the same line
            || trailing(n.hi);
    }
}

impl<C: Comments, S: SourceMapper> VisitMut for Refresh<C, S> {
    fn visit_mut_module(&mut self, n: &mut Module) {
        // A reset comment only applies to its own file
        self.should_reset = false;
        // to collect comments
        self.visit_module(n);

        self.visit_mut_module_items(&mut n.body);
    }

    fn visit_mut_module_items(&mut self, module_items: &mut Vec<ModuleItem>) {
        let used_in_jsx = collect_ident_in_jsx(module_items);

        let mut items = Vec::with_capacity(module_items.len());
        let mut refresh_regs = Vec::<(Ident, Id)>::new();

        let mut hook_visitor = HookRegister {
            refresh_sig: &self.refresh_sig,
            emit_full_signatures: self.emit_full_signatures,
            ident: Vec::new(),
            extra_stmt: Vec::new(),
            current_scope: vec![top_level_ctxt(module_items)],
            cm: &*self.cm,
            should_reset: self.should_reset,
        };

        for mut item in module_items.take() {
            let persistent_id = match &mut item {
                // function Foo() {}
                ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl { ident, .. }))) => {
                    get_persistent_id(ident)
                }

                // export function Foo() {}
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    decl: Decl::Fn(FnDecl { ident, .. }),
                    ..
                })) => get_persistent_id(ident),

                // export default function Foo() {}
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(ExportDefaultDecl {
                    decl:
                        DefaultDecl::Fn(FnExpr {
                            // We don't currently handle anonymous default exports.
                            ident: Some(ident),
                            ..
                        }),
                    ..
                })) => get_persistent_id(ident),

                // const Foo = () => {}
                // export const Foo = () => {}
                ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl)))
                | ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    decl: Decl::Var(var_decl),
                    ..
                })) => {
                    self.get_persistent_id_from_var_decl(var_decl, &used_in_jsx, &mut hook_visitor)
                }

                // This code path handles nested cases like:
                // export default memo(() => {})
                // In those cases it is more plausible people will omit names
                // so they're worth handling despite possible false positives.
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                    expr,
                    span,
                })) => {
                    if let Expr::Call(call) = expr.as_mut()
                        && is_possible_hoc(call)
                        && let Persist::Hoc(Hoc { reg, hook, .. }) = self
                            .get_persistent_id_from_possible_hoc(
                                call,
                                vec![(
                                    private_ident!("_c"),
                                    ("%default%".into(), SyntaxContext::empty()),
                                )],
                                &mut hook_visitor,
                            )
                    {
                        if let Some(hook) = hook {
                            make_hook_reg(expr.as_mut(), hook);
                        }
                        item = ExportDefaultExpr {
                            expr: Box::new(make_assign_stmt(reg[0].0.clone(), expr.take())),
                            span: *span,
                        }
                        .into();
                        Persist::Hoc(Hoc {
                            insert: false,
                            reg,
                            hook: None,
                        })
                    } else {
                        Persist::None
                    }
                }

                _ => Persist::None,
            };

            if let Persist::Hoc(_) = persistent_id {
                // we need to make hook transform happens after component for
                // HOC
                items.push(item);
            } else {
                item.visit_mut_children_with(&mut hook_visitor);

                items.push(item);
                items.extend(
                    hook_visitor
                        .extra_stmt
                        .take()
                        .into_iter()
                        .map(ModuleItem::Stmt),
                );
            }

            match persistent_id {
                Persist::None => (),
                Persist::Component(persistent_id) => {
                    let registration_handle = private_ident!("_c");

                    refresh_regs.push((registration_handle.clone(), persistent_id.to_id()));

                    items.push(
                        ExprStmt {
                            span: DUMMY_SP,
                            expr: Box::new(make_assign_stmt(
                                registration_handle,
                                persistent_id.into(),
                            )),
                        }
                        .into(),
                    );
                }

                Persist::Hoc(mut hoc) => {
                    hoc.reg.reverse();
                    if hoc.insert
                        && let Some((ident, name)) = hoc.reg.last()
                    {
                        items.push(
                            ExprStmt {
                                span: DUMMY_SP,
                                expr: Box::new(make_assign_stmt(
                                    ident.clone(),
                                    Ident::new(name.0.clone(), DUMMY_SP, name.1).into(),
                                )),
                            }
                            .into(),
                        )
                    }
                    refresh_regs.append(&mut hoc.reg);
                }
            }
        }

        if !hook_visitor.ident.is_empty() {
            items.insert(0, hook_visitor.gen_hook_handle().into());
        }

        // Insert
        // ```
        // var _c, _c1;
        // ```
        if !refresh_regs.is_empty() {
            items.push(
                VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Var,
                    declare: false,
                    decls: refresh_regs
                        .iter()
                        .map(|(handle, _)| VarDeclarator {
                            span: DUMMY_SP,
                            name: handle.clone().into(),
                            init: None,
                            definite: false,
                        })
                        .collect(),
                    ..Default::default()
                }
                .into(),
            );
        }

        // Insert
        // ```
        // $RefreshReg$(_c, "Hello");
        // $RefreshReg$(_c1, "Foo");
        // ```
        for (handle, persistent_id) in refresh_regs {
            items.push(
                ExprStmt {
                    span: DUMMY_SP,
                    expr: CallExpr {
                        callee: quote_ident!(self.refresh_reg.clone()).as_callee(),
                        args: vec![handle.as_arg(), quote_str!(persistent_id.0).as_arg()],
                        ..Default::default()
                    }
                    .into(),
                }
                .into(),
            );
        }

        *module_items = items
    }

    fn visit_mut_ts_module_decl(&mut self, _: &mut TsModuleDecl) {}
}

fn make_hook_reg(expr: &mut Expr, mut hook: HocHook) {
    let span = expr.span();
    let mut args = vec![expr.take().as_arg()];
    args.append(&mut hook.rest_arg);
    *expr = CallExpr {
        span,
        callee: hook.callee,
        args,
        ..Default::default()
    }
    .into();
}
