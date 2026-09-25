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
enum VType {
    Element { tag: Box<Expr>, flags: u16 },
    Component(Box<Expr>),
    Fragment,
}

/// `getVNodeChildren` of babel-plugin-inferno
struct ChildrenResult {
    parent_can_be_keyed: bool,
    children: Box<Expr>,
    found_text: bool,
    /// A child is an element or a fragment
    found_vnode: bool,
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
        JSXElementName::Ident(ident) => Some(if ident.sym == "Fragment" {
            VType::Fragment
        } else if is_component(&ident.sym) && is_valid_identifier(&ident.sym) {
            // Names that are not identifiers, like Foo-bar, can only be elements
            VType::Component(Box::new(Expr::Ident(ident)))
        } else {
            VType::Element {
                flags: parse_vnode_flag(&ident.sym),
                tag: Box::new(Expr::Lit(Lit::Str(Str {
                    span: ident.span,
                    value: ident.sym.into(),
                    raw: None,
                }))),
            }
        }),
        JSXElementName::JSXMemberExpr(member) => {
            if member.prop.sym == "Fragment" {
                return Some(VType::Fragment);
            }
            let object = member_object(member.obj)?;

            Some(VType::Component(member_expr(
                member.span,
                object,
                member.prop,
            )))
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

    /// Whether generated calls get pure annotations
    fn annotates(&self) -> bool {
        self.pure && self.comments.is_some()
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
    /// the children are not normalized at runtime. `createTextVNode` is imported with the first
    /// text vNode.
    fn transform_text_nodes(&mut self, mut children: Box<Expr>, found_vnode: bool) -> Box<Expr> {
        if let Expr::Array(array) = &mut *children {
            for element in array.elems.iter_mut().flatten() {
                if element.spread.is_none() && matches!(*element.expr, Expr::Lit(Lit::Str(_))) {
                    element.expr = self.text_vnode(element.expr.take());
                }
            }
            return children;
        }
        // A single child is a string, or any expression that $HasTextChildren declares as text on
        // a fragment. JSX stays a vNode whatever the flag says.
        if found_vnode || matches!(*children, Expr::JSXElement(_) | Expr::JSXFragment(_)) {
            return children;
        }
        self.text_vnode(children)
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
        let mut found_vnode = false;
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
                JSXElementChild::JSXElement(_) | JSXElementChild::JSXFragment(_) => {
                    found_vnode = true;
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
            found_vnode,
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
        let ChildrenResult {
            parent_can_be_keyed,
            children,
            found_text,
            found_vnode,
            requires_normalization,
            has_single_child,
            ..
        } = self.get_vnode_children(frag.children, false);
        let (children, child_flags) = if requires_normalization {
            (children, ChildFlags::UnknownChildren)
        } else {
            (
                if has_single_child {
                    array(vec![children.as_arg()])
                } else {
                    children
                },
                if parent_can_be_keyed {
                    ChildFlags::HasKeyedChildren
                } else {
                    ChildFlags::HasNonKeyedChildren
                },
            )
        };
        let children = if found_text {
            self.transform_text_nodes(children, found_vnode)
        } else {
            children
        };

        self.call(
            span,
            Helper::CreateFragment,
            create_fragment_vnode_args(
                Some(children),
                child_flags.into(),
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
        let is_component = matches!(vtype, VType::Component(_));
        let is_fragment = matches!(vtype, VType::Fragment);
        let mut vprops = get_vnode_props(el.opening.attrs, is_component);
        let ChildrenResult {
            parent_can_be_keyed,
            children,
            mut found_text,
            found_vnode,
            parent_can_be_non_keyed,
            requires_normalization,
            mut has_single_child,
        } = self.get_vnode_children(el.children, vprops.children_known || is_component);
        let mut children = Some(children);
        let mut child_flags = ChildFlags::HasInvalidChildren;
        let mut flags = match &vtype {
            VType::Element { flags, .. } => *flags,
            VType::Component(_) | VType::Fragment => VNodeFlags::ComponentUnknown as u16,
        };
        let mut overridden = None;
        // A single fragment child that $HasTextChildren declares as text, which goes in an array
        // like a static one
        let mut single_text_child = false;

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
                            found_text = true;
                            has_single_child = true;
                            children = Some(Box::new(Expr::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: text,
                                raw: None,
                            }))));
                        } else {
                            children = None;
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
                        child_flags = if !is_fragment && (is_jsx || vprops.children_known) {
                            ChildFlags::HasVNodeChildren
                        } else {
                            ChildFlags::UnknownChildren
                        };
                        children = value;
                    }
                    PropChildren::Empty | PropChildren::Other => children = None,
                }
            }

            if !requires_normalization || vprops.children_known {
                if vprops.has_keyed_children || parent_can_be_keyed {
                    child_flags = ChildFlags::HasKeyedChildren;
                } else if vprops.has_non_keyed_children || parent_can_be_non_keyed {
                    child_flags = ChildFlags::HasNonKeyedChildren;
                } else if vprops.has_text_children || (found_text && has_single_child) {
                    found_text = is_fragment;
                    child_flags = if is_fragment {
                        ChildFlags::HasNonKeyedChildren
                    } else {
                        ChildFlags::HasTextChildren
                    };
                    single_text_child = is_fragment
                        && children
                            .as_deref()
                            .is_some_and(|children| !matches!(children, Expr::Array(_)));
                } else if has_single_child {
                    // A static fragment child is put in an array below, a dynamic one declared by
                    // $HasVNodeChildren is passed as is
                    child_flags = if is_fragment && !requires_normalization {
                        ChildFlags::HasNonKeyedChildren
                    } else {
                        ChildFlags::HasVNodeChildren
                    };
                }
            } else if vprops.has_keyed_children {
                child_flags = ChildFlags::HasKeyedChildren;
            } else if vprops.has_non_keyed_children {
                child_flags = ChildFlags::HasNonKeyedChildren;
            }

            // A children prop is passed as the children argument, or replaced by JSX children
            overridden = vprops.take_children_prop();
        }

        if found_text {
            children = children.map(|children| self.transform_text_nodes(children, found_vnode));
        }

        let child_flags = match vprops.child_flags {
            // If $ChildFlag is provided it is runtime dependant
            Some(expr) => Flag::Expr(expr),
            None if !is_component && requires_normalization && !vprops.children_known => {
                ChildFlags::UnknownChildren.into()
            }
            None => child_flags.into(),
        };

        if let Some(overridden) = overridden {
            children = with_overridden(*overridden, children);
        }

        let span = self.annotate(span);
        // normalizeProps wraps the call and keeps the position of the tag, like in
        // babel-plugin-inferno. A pure annotation belongs to a position, so the wrapped call gets
        // a position of its own to be annotated too; otherwise a minifier keeps it when the
        // vNode is unused.
        let call_span = if vprops.needs_normalization && self.annotates() {
            self.annotate(DUMMY_SP)
        } else {
            span
        };
        let flags = match vprops.flags_override {
            Some(expr) => Flag::Expr(expr),
            None => Flag::Known(flags),
        };
        let props = ObjectLit {
            span: DUMMY_SP,
            props: vprops.props,
        };

        let call = match vtype {
            VType::Component(tag) => {
                let args =
                    create_component_vnode_args(flags, tag, props, vprops.key, vprops.reference);
                self.call(call_span, Helper::CreateComponentVNode, args)
            }
            VType::Element { tag, .. } => {
                let args = CreateVNodeArgs {
                    flags,
                    tag,
                    class_name: vprops.class_name,
                    children,
                    child_flags,
                    props,
                    key: vprops.key,
                    reference: vprops.reference,
                }
                .into_args();
                self.call(call_span, Helper::CreateVNode, args)
            }
            VType::Fragment => {
                if single_text_child || (!requires_normalization && has_single_child) {
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
