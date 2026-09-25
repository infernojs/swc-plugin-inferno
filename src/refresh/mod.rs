//! Fast refresh registrations, a port of swc's React refresh transform.

use self::{
    cycles::HookCycles,
    hook::HookRegister,
    util::{
        collect_ident_in_jsx, is_body_arrow_fn, is_directive, is_import_or_require,
        make_assign_expr, make_assign_stmt, top_level_ctxt, var_decl,
    },
};
use rustc_hash::FxHashSet;
use std::borrow::Cow;
use swc_core::{
    atoms::{Atom, atom},
    common::{
        BytePos, DUMMY_SP, SourceMapper, Span, SyntaxContext,
        comments::{Comment, Comments},
        sync::Lrc,
        util::take::Take,
    },
    ecma::{
        ast::*,
        utils::{ExprFactory, private_ident, quote_ident, quote_str},
        visit::{Visit, VisitMut, VisitMutWith, noop_visit_mut_type, visit_mut_pass},
    },
};

pub mod options;
use options::RefreshOptions;
mod cycles;
mod hook;
mod util;

#[cfg(test)]
mod tests;

/// The name of the variables that hold registered components
const REGISTRATION_HANDLE: &str = "_c";

/// A `$RefreshReg$(handle, "name")` registration
struct Reg {
    handle: Ident,
    name: Atom,
}

impl Reg {
    fn new(name: Atom) -> Self {
        Reg {
            handle: private_ident!(REGISTRATION_HANDLE),
            name,
        }
    }
}

/// A registered HOC call chain
struct Hoc {
    /// The registrations from the outermost call to the component
    regs: Vec<Reg>,
    /// The signature call of the component, which wraps each HOC call too
    hook: Option<HocHook>,
}

struct HocHook {
    callee: Callee,
    rest_arg: Vec<ExprOrSpread>,
}

impl HocHook {
    /// `callee(expr, ...rest_arg)`. The call has no position: a position shared with `expr` would
    /// take the comments of `expr`, like its pure annotation.
    fn wrap(&self, expr: Expr) -> Expr {
        let mut args = Vec::with_capacity(1 + self.rest_arg.len());
        args.push(expr.as_arg());
        args.extend(self.rest_arg.iter().cloned());

        CallExpr {
            span: DUMMY_SP,
            callee: self.callee.clone(),
            args,
            ..Default::default()
        }
        .into()
    }
}

/// What a module item declares
enum Persist {
    /// A HOC call chain, with its registrations from the outermost call. The outermost handle is
    /// assigned from `target` after the declaration, or in place when there is no target.
    Hoc {
        regs: Vec<Reg>,
        target: Option<Ident>,
    },
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
    /// The component or HOC that a module item declares
    fn persistent_id(
        &self,
        item: &mut ModuleItem,
        used_in_jsx: &FxHashSet<Id>,
        hook_reg: &mut HookRegister<S>,
    ) -> Persist {
        match item {
            // function Foo() {}
            ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl { ident, .. })))
            // export function Foo() {}
            | ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                decl: Decl::Fn(FnDecl { ident, .. }),
                ..
            }))
            // export default function Foo() {}
            | ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(ExportDefaultDecl {
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
            })) => self.get_persistent_id_from_var_decl(var_decl, used_in_jsx, hook_reg),

            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export)) => {
                self.get_persistent_id_from_default_export(export, hook_reg)
            }

            _ => Persist::None,
        }
    }

    fn get_persistent_id_from_var_decl(
        &self,
        var_decl: &mut VarDecl,
        used_in_jsx: &FxHashSet<Id>,
        hook_reg: &mut HookRegister<S>,
    ) -> Persist {
        // We only handle the case when a single variable is declared
        let [
            VarDeclarator {
                name: Pat::Ident(binding),
                init: Some(init_expr),
                ..
            },
        ] = var_decl.decls.as_mut_slice()
        else {
            return Persist::None;
        };

        if used_in_jsx.contains(&binding.to_id())
            && !is_import_or_require(init_expr)
            // TaggedTpl is for something like styled.div`...`
            && matches!(
                **init_expr,
                Expr::Arrow(_) | Expr::Fn(_) | Expr::TaggedTpl(_) | Expr::Call(_)
            )
        {
            return Persist::Component(Ident::from(&*binding));
        }

        let Persist::Component(persistent_id) = get_persistent_id(&Ident::from(&*binding)) else {
            return Persist::None;
        };
        match &mut **init_expr {
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
                let root = Reg::new(persistent_id.sym.clone());
                let Some(Hoc { regs, hook }) =
                    self.get_persistent_id_from_possible_hoc(call_expr, vec![root], hook_reg)
                else {
                    return Persist::None;
                };
                if let Some(hook) = hook {
                    **init_expr = hook.wrap(init_expr.as_mut().take());
                }
                Persist::Hoc {
                    regs,
                    target: Some(Ident::new(persistent_id.sym, DUMMY_SP, persistent_id.ctxt)),
                }
            }
            _ => Persist::None,
        }
    }

    /// Handles nested cases like `export default memo(() => {})`. In those cases it is more
    /// plausible people will omit names so they're worth handling despite possible false
    /// positives.
    fn get_persistent_id_from_default_export(
        &self,
        export: &mut ExportDefaultExpr,
        hook_reg: &mut HookRegister<S>,
    ) -> Persist {
        let Expr::Call(call) = &mut *export.expr else {
            return Persist::None;
        };
        if !is_possible_hoc(call) {
            return Persist::None;
        }
        let root = Reg::new(atom!("%default%"));
        let Some(Hoc { regs, hook }) =
            self.get_persistent_id_from_possible_hoc(call, vec![root], hook_reg)
        else {
            return Persist::None;
        };

        if let Some(hook) = hook {
            *export.expr = hook.wrap(export.expr.as_mut().take());
        }
        *export.expr = make_assign_expr(regs[0].handle.clone(), export.expr.take());

        Persist::Hoc { regs, target: None }
    }

    fn get_persistent_id_from_possible_hoc(
        &self,
        call_expr: &mut CallExpr,
        mut regs: Vec<Reg>,
        hook_reg: &mut HookRegister<S>,
    ) -> Option<Hoc> {
        let [first, ..] = call_expr.args.as_mut_slice() else {
            return None;
        };
        let first_arg = &mut first.expr;
        let Callee::Expr(callee) = &call_expr.callee else {
            return None;
        };
        let hoc_name: Cow<'_, str> = match callee.as_ref() {
            Expr::Ident(fn_name) => Cow::Borrowed(fn_name.sym.as_ref()),
            // original react implement use `getSource` so we just follow them
            Expr::Member(member) => {
                Cow::Owned(self.cm.span_to_snippet(member.span).unwrap_or_default())
            }
            _ => return None,
        };
        let name: Atom = format!("{}${hoc_name}", regs.last()?.name).into();

        match first_arg.as_mut() {
            Expr::Call(expr) => {
                let handle = private_ident!(REGISTRATION_HANDLE);
                regs.push(Reg {
                    handle: handle.clone(),
                    name,
                });
                let hoc = self.get_persistent_id_from_possible_hoc(expr, regs, hook_reg)?;
                let mut first = first_arg.take();
                if let Some(hook) = &hoc.hook {
                    first = Box::new(hook.wrap(*first));
                }
                **first_arg = make_assign_expr(handle, first);

                Some(hoc)
            }
            Expr::Fn(_) | Expr::Arrow(_) => {
                let handle = private_ident!(REGISTRATION_HANDLE);
                let mut first = first_arg.take();
                first.visit_mut_with(hook_reg);
                let hook = match &*first {
                    Expr::Call(call) => Some(HocHook {
                        callee: call.callee.clone(),
                        rest_arg: call.args.get(1..).unwrap_or_default().to_vec(),
                    }),
                    _ => None,
                };
                **first_arg = make_assign_expr(handle.clone(), first);
                regs.push(Reg { handle, name });

                Some(Hoc { regs, hook })
            }
            // export default hoc(Foo)
            // const X = hoc(Foo)
            Expr::Ident(ident) => matches!(get_persistent_id(ident), Persist::Component(_))
                .then_some(Hoc { regs, hook: None }),
            _ => None,
        }
    }

    /// Registers the components of a module and the signatures of its hooks
    fn transform_items(&self, module_items: &mut Vec<ModuleItem>) {
        let used_in_jsx = collect_ident_in_jsx(module_items);

        let mut items = Vec::with_capacity(module_items.len());
        let mut refresh_regs = Vec::<Reg>::new();

        let cycles = HookCycles::find(module_items);
        let mut hook_visitor = HookRegister {
            cycles: &cycles,
            refresh_sig: &self.refresh_sig,
            emit_full_signatures: self.emit_full_signatures,
            ident: Vec::new(),
            extra_stmt: Vec::new(),
            current_scope: vec![top_level_ctxt(module_items)],
            cm: &*self.cm,
            should_reset: self.should_reset,
        };

        for mut item in module_items.take() {
            let persistent_id = self.persistent_id(&mut item, &used_in_jsx, &mut hook_visitor);

            if let Persist::Hoc { .. } = persistent_id {
                // we need to make hook transform happens after component for
                // HOC
                items.push(item);
            } else {
                item.visit_mut_children_with(&mut hook_visitor);

                items.push(item);
                items.extend(hook_visitor.extra_stmt.drain(..).map(ModuleItem::Stmt));
            }

            match persistent_id {
                Persist::None => (),
                Persist::Component(persistent_id) => {
                    let reg = Reg::new(persistent_id.sym.clone());

                    items.push(make_assign_stmt(reg.handle.clone(), persistent_id.into()).into());
                    refresh_regs.push(reg);
                }
                Persist::Hoc { mut regs, target } => {
                    if let Some(target) = target {
                        items.push(make_assign_stmt(regs[0].handle.clone(), target.into()).into());
                    }
                    // The component is registered first
                    regs.reverse();
                    refresh_regs.append(&mut regs);
                }
            }
        }

        if !hook_visitor.ident.is_empty() {
            let prologue = items
                .iter()
                .take_while(|item| matches!(item, ModuleItem::Stmt(stmt) if is_directive(stmt)))
                .count();
            items.insert(prologue, hook_visitor.gen_hook_handle().into());
        }

        // Insert
        // ```
        // var _c, _c1;
        // ```
        if !refresh_regs.is_empty() {
            items.push(
                var_decl(
                    refresh_regs
                        .iter()
                        .map(|reg| VarDeclarator {
                            span: DUMMY_SP,
                            name: reg.handle.clone().into(),
                            init: None,
                            definite: false,
                        })
                        .collect(),
                )
                .into(),
            );
        }

        // Insert
        // ```
        // $RefreshReg$(_c, "Hello");
        // $RefreshReg$(_c1, "Foo");
        // ```
        for Reg { handle, name } in refresh_regs {
            items.push(
                ExprStmt {
                    span: DUMMY_SP,
                    expr: CallExpr {
                        callee: quote_ident!(self.refresh_reg.clone()).as_callee(),
                        args: vec![handle.as_arg(), quote_str!(name).as_arg()],
                        ..Default::default()
                    }
                    .into(),
                }
                .into(),
            );
        }

        *module_items = items;
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
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, n: &mut Module) {
        // A reset comment only applies to its own file
        self.should_reset = false;
        // to collect comments
        self.visit_module(n);

        self.transform_items(&mut n.body);
    }

    /// Components are only registered in ES modules
    fn visit_mut_script(&mut self, _: &mut Script) {}
}
