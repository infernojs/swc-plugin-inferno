use rustc_hash::FxHashMap;
use swc_core::atoms::{Atom, atom};
use swc_core::common::Span;
use swc_core::common::comments::Comments;
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{VisitMut, VisitMutWith, noop_visit_mut_type, visit_mut_pass};

#[cfg(test)]
mod tests;

/// This pass adds a /*#__PURE__#/ annotation to calls to known pure top-level
/// Inferno methods, so that terser and other minifiers can safely remove them
/// during dead code elimination.
pub fn pure_annotations<C>(comments: Option<C>) -> impl Pass
where
    C: Comments,
{
    visit_mut_pass(PureAnnotations {
        imports: Default::default(),
        comments,
    })
}

struct PureAnnotations<C>
where
    C: Comments,
{
    imports: FxHashMap<Id, Atom>,
    comments: Option<C>,
}

impl<C> VisitMut for PureAnnotations<C>
where
    C: Comments,
{
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        // Pass 1: collect imports
        for item in &module.body {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
                let Some(src_str) = import.src.value.as_str() else {
                    continue;
                };
                if src_str != "inferno" {
                    continue;
                }

                for specifier in &import.specifiers {
                    match specifier {
                        ImportSpecifier::Named(named) => {
                            let imported: Atom = match &named.imported {
                                Some(ModuleExportName::Ident(imported)) => imported.sym.clone(),
                                Some(ModuleExportName::Str(s)) => {
                                    s.value.to_atom_lossy().into_owned()
                                }
                                None => named.local.sym.clone(),
                                #[cfg(swc_ast_unknown)]
                                Some(_) => continue,
                            };
                            self.imports.insert(named.local.to_id(), imported);
                        }
                        ImportSpecifier::Default(default) => {
                            self.imports.insert(default.local.to_id(), atom!("default"));
                        }
                        ImportSpecifier::Namespace(ns) => {
                            self.imports.insert(ns.local.to_id(), atom!("*"));
                        }
                        #[cfg(swc_ast_unknown)]
                        _ => (),
                    }
                }
            }
        }

        if self.imports.is_empty() {
            return;
        }

        // Pass 2: add pure annotations.
        module.visit_mut_children_with(self);
    }

    fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
        if self.should_annotate(call)
            && let Some(comments) = &self.comments
        {
            if call.span.lo.is_dummy() {
                call.span.lo = Span::dummy_with_cmt().lo;
            }

            comments.add_pure_comment(call.span.lo);
        }

        call.visit_mut_children_with(self);
    }
}

impl<C> PureAnnotations<C>
where
    C: Comments,
{
    /// Returns the name of the Inferno export called by `callee`, for both
    /// `foo()` (named import) and `Inferno.foo()` (default / namespace import).
    fn inferno_export<'a>(&'a self, callee: &'a Callee) -> Option<&'a Atom> {
        let Callee::Expr(expr) = callee else {
            return None;
        };

        match &**expr {
            Expr::Ident(ident) => self.imports.get(&ident.to_id()),
            Expr::Member(member) => {
                let Expr::Ident(obj) = &*member.obj else {
                    return None;
                };
                let specifier = self.imports.get(&obj.to_id())?;
                if &**specifier != "default" && &**specifier != "*" {
                    return None;
                }
                match &member.prop {
                    MemberProp::Ident(prop) => Some(&prop.sym),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn should_annotate(&self, call: &CallExpr) -> bool {
        match self.inferno_export(&call.callee).map(|name| &**name) {
            // normalizeProps mutates its argument in place, so it can only be
            // dropped when that argument is a fresh vNode nothing else refers to.
            Some("normalizeProps") => call
                .args
                .first()
                .is_some_and(|arg| arg.spread.is_none() && self.is_fresh_vnode(&arg.expr)),
            Some(name) => is_pure(name),
            None => false,
        }
    }

    fn is_fresh_vnode(&self, expr: &Expr) -> bool {
        let Expr::Call(call) = expr.unwrap_parens() else {
            return false;
        };

        match self.inferno_export(&call.callee).map(|name| &**name) {
            Some(
                "createVNode"
                | "createComponentVNode"
                | "createFragment"
                | "createTextVNode"
                | "createPortal"
                | "directClone",
            ) => true,
            Some("normalizeProps") => self.should_annotate(call),
            _ => false,
        }
    }
}

fn is_pure(specifier: &str) -> bool {
    // Only imports from "inferno" are collected, see `visit_mut_module`.
    matches!(
        specifier,
        "createComponentVNode"
            | "createFragment"
            | "createPortal"
            | "createRef"
            | "createRenderer"
            | "createTextVNode"
            | "createVNode"
            | "forwardRef"
            | "directClone"
            | "findDOMFromVNode"
            | "getFlagsForElementVnode"
            | "linkEvent"
            | "createElement"
            | "createClass"
    )
}
