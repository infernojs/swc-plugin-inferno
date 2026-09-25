//! Turns JSX into Inferno function calls.
//!
//! This is a port of babel-plugin-inferno (`lib/index.js`), and generates the
//! same calls. The comments name the babel-plugin-inferno functions that the
//! code corresponds to.

use crate::inferno_flags::{ChildFlags, VNodeFlags};
use crate::refresh::options::{RefreshOptions, deserialize_refresh};
use crate::transformations::parse_vnode_flag::parse_vnode_flag;
use serde::{Deserialize, Serialize};
use swc_core::atoms::{Atom, Wtf8Atom, atom};
use swc_core::common::comments::Comments;
use swc_core::common::util::take::Take;
use swc_core::common::{DUMMY_SP, Mark, Span, Spanned, SyntaxContext};
use swc_core::ecma::ast::*;
use swc_core::ecma::utils::{ExprFactory, prepend_stmt};
use swc_core::ecma::visit::{VisitMut, VisitMutWith, noop_visit_mut_type, visit_mut_pass};

#[cfg(test)]
mod tests;

mod bindings;
mod props;
mod text;
mod vnode_args;

use self::bindings::{HelperBindings, generate_uid, module_bindings, script_bindings, used_names};
use self::props::{PropChildren, emit_error, get_vnode_props, key_value, unparen};
use self::text::{handle_white_space, map_text};
use self::vnode_args::{
    CreateVNodeArgs, Flag, create_component_vnode_args, create_fragment_vnode_args, is_empty_array,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(default)]
    pub import_source: Option<String>,

    #[serde(default)]
    pub development: Option<bool>,

    /// Emit `/*#__PURE__*/` annotations. Defaults to `true`.
    #[serde(default)]
    pub pure: Option<bool>,

    #[serde(default, deserialize_with = "deserialize_refresh")]
    // default to disabled since this is still considered as experimental by now
    pub refresh: Option<RefreshOptions>,
}

impl Options {
    /// The module the helpers are imported from when `importSource` is not set
    pub(crate) const DEFAULT_IMPORT_SOURCE: &str = "inferno";

    /// Whether generated calls get `/*#__PURE__*/` annotations
    pub fn pure(&self) -> bool {
        self.pure.unwrap_or(true)
    }

    /// Whether the fast refresh pass runs (it also needs `refresh`)
    pub fn development(&self) -> bool {
        self.development.unwrap_or(false)
    }

    /// The module the helpers are imported from
    pub fn import_source(&self) -> &str {
        self.import_source
            .as_deref()
            .unwrap_or(Self::DEFAULT_IMPORT_SOURCE)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VNodeType {
    Element,
    Component,
    Fragment,
}

/// The Inferno functions that the generated code calls, in the order they are imported
#[derive(Clone, Copy)]
enum Helper {
    CreateVNode,
    CreateFragment,
    CreateComponentVNode,
    NormalizeProps,
    CreateTextVNode,
}

const HELPERS: [Helper; HELPER_COUNT] = [
    Helper::CreateVNode,
    Helper::CreateFragment,
    Helper::CreateComponentVNode,
    Helper::NormalizeProps,
    Helper::CreateTextVNode,
];

const HELPER_COUNT: usize = 5;

impl Helper {
    /// The helper that a binding of this name declares
    fn from_name(name: &str) -> Option<Self> {
        HELPERS.into_iter().find(|helper| helper.name() == name)
    }

    fn name(self) -> Atom {
        match self {
            Helper::CreateVNode => atom!("createVNode"),
            Helper::CreateFragment => atom!("createFragment"),
            Helper::CreateComponentVNode => atom!("createComponentVNode"),
            Helper::NormalizeProps => atom!("normalizeProps"),
            Helper::CreateTextVNode => atom!("createTextVNode"),
        }
    }
}

/// Turns JSX into Inferno function calls.
///
/// `unresolved_mark` should be the unresolved [Mark] passed to swc's `resolver`.
pub fn jsx<C>(comments: Option<C>, options: Options, unresolved_mark: Mark) -> impl Pass
where
    C: Comments,
{
    visit_mut_pass(Jsx {
        unresolved_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
        import_source: options.import_source().into(),
        pure: options.pure(),
        comments,
        used: [false; HELPER_COUNT],
        bindings: [None; HELPER_COUNT],
    })
}

struct Jsx<C>
where
    C: Comments,
{
    /// The context of unresolved (global) identifiers
    unresolved_ctxt: SyntaxContext,
    import_source: Wtf8Atom,
    pure: bool,
    comments: Option<C>,
    /// Helpers called by the generated code
    used: [bool; HELPER_COUNT],
    /// Helpers the program declares itself, which are called instead of importing them
    bindings: HelperBindings,
}

/// `getVNodeType` of babel-plugin-inferno
struct VType {
    kind: VNodeType,
    tag: Option<Box<Expr>>,
    flags: u16,
}

/// `getVNodeChildren` of babel-plugin-inferno
struct ChildrenResult {
    parent_can_be_keyed: bool,
    children: Box<Expr>,
    found_text: bool,
    parent_can_be_non_keyed: bool,
    requires_normalization: bool,
    has_single_child: bool,
}

/// `isComponent` of babel-plugin-inferno: the first letter is not changed by uppercasing
fn is_component(name: &str) -> bool {
    let Some(first) = name.chars().next() else {
        return false;
    };
    // JavaScript uppercases the first UTF-16 code unit, which leaves a lone surrogate unchanged
    if first as u32 > 0xffff {
        return true;
    }

    let mut upper = first.to_uppercase();
    upper.next() == Some(first) && upper.next().is_none()
}

/// `t.isValidIdentifier(name, false)`: an identifier name, reserved words included
fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();

    chars.next().is_some_and(Ident::is_valid_start) && chars.all(Ident::is_valid_continue)
}

fn has_key_attr(el: &JSXElement) -> bool {
    el.opening.attrs.iter().any(|attr| {
        matches!(
            attr,
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                name: JSXAttrName::Ident(name),
                ..
            }) if name.sym == "key"
        )
    })
}

fn null_expr() -> Box<Expr> {
    Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP })))
}

fn array(elems: Vec<ExprOrSpread>) -> Box<Expr> {
    Box::new(Expr::Array(ArrayLit {
        span: DUMMY_SP,
        elems: elems.into_iter().map(Some).collect(),
    }))
}

/// `mayHaveSideEffects` of babel-plugin-inferno
fn may_have_side_effects(expr: &Expr) -> bool {
    match expr {
        Expr::Paren(paren) => may_have_side_effects(&paren.expr),
        Expr::Ident(_) | Expr::Fn(_) | Expr::Arrow(_) => false,
        Expr::Tpl(tpl) => tpl.exprs.iter().any(|expr| may_have_side_effects(expr)),
        Expr::Lit(_) => false,
        Expr::Array(array) => array
            .elems
            .iter()
            .flatten()
            .any(|element| element.spread.is_some() || may_have_side_effects(&element.expr)),
        Expr::Object(object) => object.props.iter().any(|prop| match prop {
            PropOrSpread::Prop(prop) => match &**prop {
                Prop::Shorthand(_) => false,
                Prop::KeyValue(KeyValueProp { key, value }) => {
                    matches!(key, PropName::Computed(_)) || may_have_side_effects(value)
                }
                _ => true,
            },
            _ => true,
        }),
        Expr::Unary(unary) if unary.op != UnaryOp::Delete => may_have_side_effects(&unary.arg),
        _ => true,
    }
}

/// `withOverridden` of babel-plugin-inferno: a children prop replaced by JSX children is still
/// evaluated before them, like in React's JSX transform. `None` stands for no children argument,
/// and stays so when the prop has no side effects.
fn with_overridden(overridden: Expr, value: Option<Box<Expr>>) -> Option<Box<Expr>> {
    // The prop value has no parentheses, see `get_value`
    let mut exprs: Vec<_> = match overridden {
        Expr::Seq(seq) => seq
            .exprs
            .into_iter()
            .filter(|expr| may_have_side_effects(expr))
            .collect(),
        node if may_have_side_effects(&node) => vec![Box::new(node)],
        _ => vec![],
    };

    if exprs.is_empty() {
        return value;
    }
    exprs.push(value.unwrap_or_else(null_expr));
    Some(Box::new(Expr::Seq(SeqExpr {
        span: DUMMY_SP,
        exprs,
    })))
}

/// `jsxMemberExpressionReference` of babel-plugin-inferno
fn member_object(object: JSXObject) -> Option<Box<Expr>> {
    match object {
        JSXObject::Ident(ident) => {
            // The object of a member expression tag must be a variable, which a-b in
            // <a-b.c /> cannot be
            if !is_valid_identifier(&ident.sym) {
                emit_error(
                    ident.span,
                    &format!(
                        "{} is not a valid variable name for a member expression tag.",
                        ident.sym
                    ),
                );
                return None;
            }
            if ident.sym == "this" {
                return Some(Box::new(Expr::This(ThisExpr { span: ident.span })));
            }
            Some(Box::new(Expr::Ident(ident)))
        }
        JSXObject::JSXMemberExpr(member) => {
            let object = member_object(member.obj)?;

            Some(member_expr(member.span, object, member.prop))
        }
        #[cfg(swc_ast_unknown)]
        _ => panic!("unable to access unknown nodes"),
    }
}

fn member_expr(span: Span, object: Box<Expr>, property: IdentName) -> Box<Expr> {
    // A property that is not an identifier, like bar-baz in <Foo.bar-baz />, needs a
    // computed member access
    let prop = if is_valid_identifier(&property.sym) {
        MemberProp::Ident(property)
    } else {
        MemberProp::Computed(ComputedPropName {
            span: property.span,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: property.span,
                value: property.sym.into(),
                raw: None,
            }))),
        })
    };

    Box::new(Expr::Member(MemberExpr {
        span,
        obj: object,
        prop,
    }))
}

/// `getVNodeType` of babel-plugin-inferno
fn get_vnode_type(name: JSXElementName) -> Option<VType> {
    match name {
        JSXElementName::Ident(ident) => {
            if ident.sym == "Fragment" {
                Some(VType {
                    kind: VNodeType::Fragment,
                    tag: None,
                    flags: VNodeFlags::ComponentUnknown as u16,
                })
            } else if is_component(&ident.sym) && is_valid_identifier(&ident.sym) {
                // Names that are not identifiers, like Foo-bar, can only be elements
                Some(VType {
                    kind: VNodeType::Component,
                    tag: Some(Box::new(Expr::Ident(ident))),
                    flags: VNodeFlags::ComponentUnknown as u16,
                })
            } else {
                Some(VType {
                    kind: VNodeType::Element,
                    flags: parse_vnode_flag(&ident.sym),
                    tag: Some(Box::new(Expr::Lit(Lit::Str(Str {
                        span: ident.span,
                        value: ident.sym.into(),
                        raw: None,
                    })))),
                })
            }
        }
        JSXElementName::JSXMemberExpr(member) => {
            if member.prop.sym == "Fragment" {
                return Some(VType {
                    kind: VNodeType::Fragment,
                    tag: None,
                    flags: VNodeFlags::ComponentUnknown as u16,
                });
            }
            let object = member_object(member.obj)?;

            Some(VType {
                kind: VNodeType::Component,
                tag: Some(member_expr(member.span, object, member.prop)),
                flags: VNodeFlags::ComponentUnknown as u16,
            })
        }
        JSXElementName::JSXNamespacedName(name) => {
            emit_error(
                name.span,
                &format!(
                    "Namespace tags like <{}:{}> are not supported.",
                    name.ns.sym, name.name.sym
                ),
            );
            None
        }
        #[cfg(swc_ast_unknown)]
        _ => panic!("unable to access unknown nodes"),
    }
}

impl<C> Jsx<C>
where
    C: Comments,
{
    fn helper(&mut self, helper: Helper) -> Ident {
        self.used[helper as usize] = true;

        Ident::new(
            helper.name(),
            DUMMY_SP,
            self.bindings[helper as usize].unwrap_or_default(),
        )
    }

    /// Adds a pure annotation in front of a generated call
    fn annotate(&self, span: Span) -> Span {
        let Some(comments) = self.comments.as_ref().filter(|_| self.pure) else {
            return span;
        };
        // JSX created by other transforms may have no position to attach the comment to
        let span = if span.lo.is_dummy() {
            Span::dummy_with_cmt()
        } else {
            span
        };

        comments.add_pure_comment(span.lo);
        span
    }

    fn call(&mut self, span: Span, helper: Helper, args: Vec<ExprOrSpread>) -> Expr {
        Expr::Call(CallExpr {
            span,
            ctxt: self.unresolved_ctxt,
            callee: self.helper(helper).as_callee(),
            args,
            type_args: None,
        })
    }

    fn text_vnode(&mut self, text: Box<Expr>) -> Box<Expr> {
        let span = self.annotate(text.span());

        Box::new(self.call(span, Helper::CreateTextVNode, vec![text.as_arg()]))
    }

    /// `transformTextNodes` of babel-plugin-inferno: text children become text vNodes when
    /// the children are not normalized at runtime
    fn transform_text_nodes(&mut self, children: Expr) -> Option<Box<Expr>> {
        self.used[Helper::CreateTextVNode as usize] = true;

        match children {
            Expr::Array(mut array) => {
                for element in array.elems.iter_mut().flatten() {
                    if element.spread.is_none() && matches!(*element.expr, Expr::Lit(Lit::Str(_))) {
                        element.expr = self.text_vnode(element.expr.take());
                    }
                }
                Some(Box::new(Expr::Array(array)))
            }
            text @ Expr::Lit(Lit::Str(_)) => Some(self.text_vnode(Box::new(text))),
            _ => None,
        }
    }

    /// `createVNode` of babel-plugin-inferno for a child of an element or a fragment
    fn child_to_vnode(&mut self, child: JSXElementChild) -> Option<ExprOrSpread> {
        match child {
            JSXElementChild::JSXText(text) => {
                let value = map_text(text.value, handle_white_space);

                if value.is_empty() {
                    return None;
                }
                Some(
                    Expr::Lit(Lit::Str(Str {
                        span: text.span,
                        value,
                        raw: None,
                    }))
                    .as_arg(),
                )
            }
            JSXElementChild::JSXExprContainer(JSXExprContainer {
                expr: JSXExpr::Expr(expr),
                ..
            }) => Some(unparen(expr).as_arg()),
            JSXElementChild::JSXExprContainer(_) => None,
            // {...children} spreads an iterable into the children array, like esbuild and
            // TypeScript compile it
            JSXElementChild::JSXSpreadChild(JSXSpreadChild { span, expr }) => Some(ExprOrSpread {
                spread: Some(span),
                expr: unparen(expr),
            }),
            JSXElementChild::JSXElement(el) => Some(self.element_to_expr(*el).as_arg()),
            JSXElementChild::JSXFragment(frag) => Some(self.fragment_to_expr(frag).as_arg()),
            #[cfg(swc_ast_unknown)]
            _ => panic!("unable to access unknown nodes"),
        }
    }

    /// `getVNodeChildren` of babel-plugin-inferno
    fn get_vnode_children(
        &mut self,
        ast_children: Vec<JSXElementChild>,
        is_children_known: bool,
    ) -> ChildrenResult {
        let ast_children_len = ast_children.len();
        let mut children = vec![];
        let mut parent_can_be_keyed = false;
        let mut requires_normalization = false;
        let mut found_text = false;
        let mut has_spread_child = false;

        for child in ast_children {
            // When a key is found on one of the children, they must all be keyed
            let is_keyed = !is_children_known
                && !parent_can_be_keyed
                && matches!(&child, JSXElementChild::JSXElement(el) if has_key_attr(el));
            let is_text = matches!(child, JSXElementChild::JSXText(_));

            match child {
                JSXElementChild::JSXExprContainer(_) => requires_normalization = true,
                JSXElementChild::JSXSpreadChild(_) => {
                    requires_normalization = true;
                    has_spread_child = true;
                }
                _ => {}
            }

            if let Some(vnode) = self.child_to_vnode(child) {
                // createVNode drops text that collapses to nothing
                found_text |= is_text;
                parent_can_be_keyed |= is_keyed;
                children.push(vnode);
            }
        }

        // A spread child is only valid inside the children array
        let has_single_child = children.len() == 1 && !has_spread_child;

        ChildrenResult {
            parent_can_be_keyed: !has_single_child && parent_can_be_keyed,
            children: if has_single_child {
                children.pop().unwrap().expr
            } else {
                array(children)
            },
            found_text,
            parent_can_be_non_keyed: !has_single_child
                && !parent_can_be_keyed
                && !requires_normalization
                && ast_children_len > 1,
            requires_normalization,
            has_single_child,
        }
    }

    /// `createVNode` of babel-plugin-inferno for a JSX fragment
    fn fragment_to_expr(&mut self, frag: JSXFragment) -> Expr {
        let span = self.annotate(frag.span);
        let result = self.get_vnode_children(frag.children, false);
        let mut children = Some(result.children);
        let child_flags = if !result.requires_normalization {
            if result.has_single_child {
                children = Some(array(vec![children.unwrap().as_arg()]));
            }
            if result.parent_can_be_keyed {
                ChildFlags::HasKeyedChildren
            } else {
                ChildFlags::HasNonKeyedChildren
            }
        } else {
            ChildFlags::UnknownChildren
        };

        if result.found_text {
            children = children.and_then(|children| self.transform_text_nodes(*children));
        }

        self.call(
            span,
            Helper::CreateFragment,
            create_fragment_vnode_args(
                children,
                Flag::Known(child_flags as u16),
                // short syntax fragments cannot have a key
                None,
            ),
        )
    }

    /// `createVNode` of babel-plugin-inferno for a JSX element
    fn element_to_expr(&mut self, el: JSXElement) -> Expr {
        let span = el.span;
        let Some(vtype) = get_vnode_type(el.opening.name) else {
            return Expr::Invalid(Invalid { span });
        };
        let kind = vtype.kind;
        let is_component = kind == VNodeType::Component;
        let mut vprops = get_vnode_props(el.opening.attrs, is_component);
        let mut result =
            self.get_vnode_children(el.children, vprops.children_known || is_component);
        let mut children = Some(result.children.take());
        let mut child_flags = Flag::Known(ChildFlags::HasInvalidChildren as u16);
        let mut flags = vtype.flags;
        let mut overridden = None;

        if vprops.has_re_create_flag {
            flags |= VNodeFlags::ReCreate as u16;
        }
        if vprops.content_editable {
            flags |= VNodeFlags::ContentEditable as u16;
        }

        if is_component {
            // JSX children replace a children prop
            let children = children
                .take()
                .filter(|children| !is_empty_array(children))
                .map(|children| match vprops.take_children_prop() {
                    Some(overridden) => with_overridden(*overridden, Some(children)),
                    None => Some(children),
                });

            if let Some(Some(value)) = children {
                vprops.props.push(key_value(
                    PropName::Ident(IdentName::new(atom!("children"), DUMMY_SP)),
                    value,
                ));
            }
        } else {
            if children.as_deref().is_some_and(is_empty_array)
                && let Some(prop_children) = vprops.prop_children.take()
            {
                match prop_children {
                    PropChildren::Str(value) => {
                        let text = map_text(value, handle_white_space);

                        if !text.is_empty() {
                            if kind != VNodeType::Fragment {
                                result.found_text = true;
                                result.has_single_child = true;
                            }
                            children = Some(Box::new(Expr::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: text,
                                raw: None,
                            }))));
                        } else {
                            children = None;
                            child_flags = Flag::Known(ChildFlags::HasInvalidChildren as u16);
                        }
                    }
                    PropChildren::Expr => {
                        // children={expression}, or children=<element /> without braces. It is
                        // passed as the children argument instead of as a prop.
                        let value = vprops.take_children_prop();
                        let is_jsx = value.as_deref().is_some_and(|value| {
                            matches!(value, Expr::JSXElement(_) | Expr::JSXFragment(_))
                        });

                        // Only JSX is known to be a single vNode; other values are normalized at
                        // runtime like {expression} children
                        child_flags = Flag::Known(if kind != VNodeType::Fragment
                            && (is_jsx || vprops.children_known)
                        {
                            ChildFlags::HasVNodeChildren
                        } else {
                            ChildFlags::UnknownChildren
                        } as u16);
                        children = value;
                    }
                    PropChildren::Empty | PropChildren::Other => {
                        children = None;
                        child_flags = Flag::Known(ChildFlags::HasInvalidChildren as u16);
                    }
                }
            }

            if !result.requires_normalization || vprops.children_known {
                if vprops.has_keyed_children || result.parent_can_be_keyed {
                    child_flags = Flag::Known(ChildFlags::HasKeyedChildren as u16);
                } else if vprops.has_non_keyed_children || result.parent_can_be_non_keyed {
                    child_flags = Flag::Known(ChildFlags::HasNonKeyedChildren as u16);
                } else if vprops.has_text_children || (result.found_text && result.has_single_child)
                {
                    result.found_text = kind == VNodeType::Fragment;
                    child_flags = Flag::Known(if kind == VNodeType::Fragment {
                        ChildFlags::HasNonKeyedChildren as u16
                    } else {
                        ChildFlags::HasTextChildren as u16
                    });
                } else if result.has_single_child {
                    child_flags = Flag::Known(if kind == VNodeType::Fragment {
                        ChildFlags::HasNonKeyedChildren as u16
                    } else {
                        ChildFlags::HasVNodeChildren as u16
                    });
                }
            } else if vprops.has_keyed_children {
                child_flags = Flag::Known(ChildFlags::HasKeyedChildren as u16);
            } else if vprops.has_non_keyed_children {
                child_flags = Flag::Known(ChildFlags::HasNonKeyedChildren as u16);
            }

            // A children prop is passed as the children argument, or replaced by JSX children
            overridden = vprops.take_children_prop();
        }

        if result.found_text {
            children = children.and_then(|children| self.transform_text_nodes(*children));
        }

        if let Some(expr) = vprops.child_flags.take() {
            // If $ChildFlag is provided it is runtime dependant
            child_flags = Flag::Expr(expr);
        } else if !is_component && result.requires_normalization && !vprops.children_known {
            child_flags = Flag::Known(ChildFlags::UnknownChildren as u16);
        }

        if let Some(overridden) = overridden {
            children = with_overridden(*overridden, children);
        }

        let span = self.annotate(span);
        let flags = match vprops.flags_override.take() {
            Some(expr) => Flag::Expr(expr),
            None => Flag::Known(flags),
        };
        let props = ObjectLit {
            span: DUMMY_SP,
            props: vprops.props,
        };

        let call = match kind {
            VNodeType::Component => {
                let args = create_component_vnode_args(
                    flags,
                    vtype.tag.unwrap(),
                    props,
                    vprops.key,
                    vprops.reference,
                );
                self.call(span, Helper::CreateComponentVNode, args)
            }
            VNodeType::Element => {
                let args = CreateVNodeArgs {
                    flags,
                    tag: vtype.tag.unwrap(),
                    class_name: vprops.class_name,
                    children,
                    child_flags,
                    props,
                    key: vprops.key,
                    reference: vprops.reference,
                }
                .into_args();
                self.call(span, Helper::CreateVNode, args)
            }
            VNodeType::Fragment => {
                if !result.requires_normalization && result.has_single_child {
                    children = children.map(|children| array(vec![children.as_arg()]));
                }
                let args = create_fragment_vnode_args(children, child_flags, vprops.key);

                return self.call(span, Helper::CreateFragment, args);
            }
        };

        // normalizeProps normalizes the children too
        if vprops.needs_normalization {
            return self.call(span, Helper::NormalizeProps, vec![call.as_arg()]);
        }

        call
    }

    fn set_bindings(&mut self, bindings: HelperBindings) {
        self.used = [false; HELPER_COUNT];
        self.bindings = bindings;
    }

    /// Helpers to import: the used ones that the program does not declare itself
    fn helpers_to_import(&self) -> Vec<Atom> {
        HELPERS
            .into_iter()
            .filter(|&helper| {
                self.used[helper as usize] && self.bindings[helper as usize].is_none()
            })
            .map(Helper::name)
            .collect()
    }
}

impl<C> VisitMut for Jsx<C>
where
    C: Comments,
{
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        let jsx = match expr {
            Expr::Paren(ParenExpr { expr: inner, .. })
                if matches!(**inner, Expr::JSXElement(_) | Expr::JSXFragment(_)) =>
            {
                &mut **inner
            }
            _ => &mut *expr,
        };

        match jsx {
            Expr::JSXElement(el) => *expr = self.element_to_expr(*el.take()),
            Expr::JSXFragment(frag) => *expr = self.fragment_to_expr(frag.take()),
            _ => {}
        }

        // JSX in attribute values and expression containers
        expr.visit_mut_children_with(self);
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        self.set_bindings(module_bindings(module));

        module.visit_mut_children_with(self);

        let helpers = self.helpers_to_import();
        if helpers.is_empty() {
            return;
        }

        prepend_stmt(
            &mut module.body,
            ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                span: DUMMY_SP,
                specifiers: helpers
                    .into_iter()
                    .map(|name| {
                        ImportSpecifier::Named(ImportNamedSpecifier {
                            span: DUMMY_SP,
                            local: Ident::new_no_ctxt(name, DUMMY_SP),
                            imported: None,
                            is_type_only: false,
                        })
                    })
                    .collect(),
                src: Box::new(Str {
                    span: DUMMY_SP,
                    value: self.import_source.clone(),
                    raw: None,
                }),
                type_only: false,
                with: None,
                phase: Default::default(),
            })),
        );
    }

    fn visit_mut_script(&mut self, script: &mut Script) {
        self.set_bindings(script_bindings(script));

        script.visit_mut_children_with(self);

        let helpers = self.helpers_to_import();
        if helpers.is_empty() {
            return;
        }

        // Scripts cannot contain import declarations, so the helpers are read from a require()
        // call instead: var _inferno = require("inferno"), createVNode = _inferno.createVNode;
        let module_id = Ident::new_no_ctxt(
            generate_uid(&self.import_source.to_string_lossy(), &used_names(script)),
            DUMMY_SP,
        );
        let require = Ident::new(atom!("require"), DUMMY_SP, self.unresolved_ctxt);
        let mut decls = vec![VarDeclarator {
            span: DUMMY_SP,
            name: module_id.clone().into(),
            init: Some(Box::new(Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: require.as_callee(),
                args: vec![
                    Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: self.import_source.clone(),
                        raw: None,
                    }))
                    .as_arg(),
                ],
                ..Default::default()
            }))),
            definite: false,
        }];

        for name in helpers {
            decls.push(VarDeclarator {
                span: DUMMY_SP,
                name: Ident::new_no_ctxt(name.clone(), DUMMY_SP).into(),
                init: Some(Box::new(Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::Ident(module_id.clone())),
                    prop: MemberProp::Ident(IdentName::new(name, DUMMY_SP)),
                }))),
                definite: false,
            });
        }

        prepend_stmt(
            &mut script.body,
            Stmt::Decl(Decl::Var(Box::new(VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Var,
                decls,
                ..Default::default()
            }))),
        );
    }
}
