use rustc_hash::FxHashMap;
use swc_core::atoms::Atom;
use swc_core::common::Span;
use swc_core::common::comments::Comments;
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{VisitMut, VisitMutWith, noop_visit_mut_type, visit_mut_pass};

#[cfg(test)]
mod tests;

/// This pass adds a /*#__PURE__*/ annotation to calls to known pure top-level
/// Inferno methods, so that terser and other minifiers can safely remove them
/// during dead code elimination.
///
/// The methods are recognized when they are imported from `inferno` or from `import_source`, the
/// module the JSX transform imports its helpers from.
pub fn pure_annotations<C>(comments: Option<C>, import_source: Atom) -> impl Pass
where
    C: Comments,
{
    visit_mut_pass(PureAnnotations {
        imports: FxHashMap::default(),
        import_source,
        comments,
    })
}

/// What an import from the Inferno module binds
enum InfernoImport {
    /// A named export
    Named(Atom),
    /// The module object or the default export, whose properties are the exports
    Object,
}

struct PureAnnotations<C>
where
    C: Comments,
{
    imports: FxHashMap<Id, InfernoImport>,
    import_source: Atom,
    comments: Option<C>,
}

impl<C> VisitMut for PureAnnotations<C>
where
    C: Comments,
{
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        // Pass 1: collect imports. A pass may be applied to several modules.
        self.imports.clear();
        for item in &module.body {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
                let Some(src) = import.src.value.as_str() else {
                    continue;
                };
                if src != "inferno" && src != &*self.import_source {
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
                            // `import { default as Inferno }` is a default import
                            let import = if imported == "default" {
                                InfernoImport::Object
                            } else {
                                InfernoImport::Named(imported)
                            };
                            self.imports.insert(named.local.to_id(), import);
                        }
                        ImportSpecifier::Default(ImportDefaultSpecifier { local, .. })
                        | ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
                            self.imports.insert(local.to_id(), InfernoImport::Object);
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

    /// Scripts cannot import Inferno
    fn visit_mut_script(&mut self, _: &mut Script) {}

    fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
        if self.should_annotate(call)
            && let Some(comments) = &self.comments
        {
            // The comment needs a position; generated calls may have none
            if call.span.lo.is_dummy() {
                call.span = Span::dummy_with_cmt();
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
            Expr::Ident(ident) => match self.imports.get(&ident.to_id())? {
                InfernoImport::Named(name) => Some(name),
                InfernoImport::Object => None,
            },
            Expr::Member(MemberExpr {
                obj,
                prop: MemberProp::Ident(prop),
                ..
            }) => match &**obj {
                Expr::Ident(obj)
                    if matches!(self.imports.get(&obj.to_id()), Some(InfernoImport::Object)) =>
                {
                    Some(&prop.sym)
                }
                _ => None,
            },
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
        let call = match expr.unwrap_parens() {
            Expr::Call(call) => call,
            // This pass runs before the JSX transform, which turns JSX into fresh vNodes
            Expr::JSXElement(_) | Expr::JSXFragment(_) => return true,
            _ => return false,
        };

        match self.inferno_export(&call.callee).map(|name| &**name) {
            Some("normalizeProps") => self.should_annotate(call),
            Some(name) => creates_vnode(name),
            None => false,
        }
    }
}

/// Inferno exports that return a new vNode
fn creates_vnode(name: &str) -> bool {
    matches!(
        name,
        "createVNode"
            | "createComponentVNode"
            | "createFragment"
            | "createTextVNode"
            | "createPortal"
            | "directClone"
    )
}

fn is_pure(name: &str) -> bool {
    // Only imports from the Inferno module are collected, see `visit_mut_module`.
    // createElement and createClass are exported by inferno-compat.
    creates_vnode(name)
        || matches!(
            name,
            "createRef"
                | "createRenderer"
                | "forwardRef"
                | "findDOMFromVNode"
                | "getFlagsForElementVnode"
                | "linkEvent"
                | "createElement"
                | "createClass"
        )
}
