//! Props of a JSX element, like `getVNodeProps` of babel-plugin-inferno.

use super::text::{collapse_attribute_line_breaks, map_text};
use crate::transformations::attribute_tables::{
    is_lowercase_attribute, react_attribute, svg_attribute,
};
use rustc_hash::FxHashMap;
use swc_core::{
    atoms::{Atom, Wtf8Atom},
    common::{DUMMY_SP, Span, Spanned},
    ecma::{ast::*, utils::is_valid_prop_ident},
    plugin::errors::HANDLER,
};

pub(super) fn emit_error(span: Span, message: &str) {
    HANDLER.with(|handler| handler.struct_span_err(span, message).emit());
}

/// Removes the parentheses around an expression; babel does not keep them in its AST.
pub(super) fn unparen(mut expr: Box<Expr>) -> Box<Expr> {
    while let Expr::Paren(paren) = *expr {
        expr = paren.expr;
    }
    expr
}

/// The value of a JSX attribute, like `getValue` of babel-plugin-inferno
pub(super) fn get_value(value: Option<JSXAttrValue>) -> Box<Expr> {
    match value {
        None => Box::new(Expr::Lit(Lit::Bool(Bool {
            span: DUMMY_SP,
            value: true,
        }))),
        Some(JSXAttrValue::JSXExprContainer(container)) => match container.expr {
            JSXExpr::Expr(expr) => unparen(expr),
            JSXExpr::JSXEmptyExpr(empty) => {
                emit_error(
                    empty.span,
                    "JSX attributes must only be assigned a non-empty expression.",
                );
                Box::new(Expr::Invalid(Invalid { span: empty.span }))
            }
            #[cfg(swc_ast_unknown)]
            _ => panic!("unable to access unknown nodes"),
        },
        // JSX strings have no escape sequences and may span lines. Build a new literal from the
        // decoded value and collapse line breaks like Babel's react-jsx does.
        Some(JSXAttrValue::Str(s)) => Box::new(Expr::Lit(Lit::Str(Str {
            span: s.span,
            value: map_text(s.value, collapse_attribute_line_breaks),
            raw: None,
        }))),
        Some(JSXAttrValue::JSXElement(el)) => Box::new(Expr::JSXElement(el)),
        Some(JSXAttrValue::JSXFragment(frag)) => Box::new(Expr::JSXFragment(frag)),
        #[cfg(swc_ast_unknown)]
        Some(_) => panic!("unable to access unknown nodes"),
    }
}

/// A key of the generated props object
pub(super) fn prop_name(name: &str, span: Span) -> PropName {
    if name == "__proto__" {
        // A non-computed __proto__ key would set the prototype of the props object instead of
        // creating a prop
        PropName::Computed(ComputedPropName {
            span,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
                span,
                value: name.into(),
                raw: None,
            }))),
        })
    } else if is_valid_prop_ident(name) {
        PropName::Ident(IdentName::new(name.into(), span))
    } else {
        PropName::Str(Str {
            span,
            value: name.into(),
            raw: None,
        })
    }
}

pub(super) fn key_value(key: PropName, value: Box<Expr>) -> PropOrSpread {
    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp { key, value })))
}

/// The value of a `children` attribute, as far as the children of an element depend on it
pub(super) enum PropChildren {
    /// `children="text"`, with the decoded text
    Str(Wtf8Atom),
    /// `children={}` or `children={null}`
    Empty,
    /// `children={expression}`, `children=<element />` or `children=<></>`
    Expr,
    /// `children` without a value
    Other,
}

/// A prop, and whether it comes from a `children` attribute
pub(super) struct PropItem {
    pub(super) prop: PropOrSpread,
    pub(super) is_children: bool,
}

#[derive(Default)]
pub(super) struct VNodeProps {
    pub(super) props: Vec<PropItem>,
    pub(super) key: Option<Box<Expr>>,
    pub(super) reference: Option<Box<Expr>>,
    pub(super) class_name: Option<Box<Expr>>,
    pub(super) has_keyed_children: bool,
    pub(super) has_non_keyed_children: bool,
    pub(super) prop_children: Option<PropChildren>,
    pub(super) children_known: bool,
    pub(super) child_flags: Option<Box<Expr>>,
    pub(super) has_re_create_flag: bool,
    pub(super) needs_normalization: bool,
    pub(super) content_editable: bool,
    pub(super) has_text_children: bool,
    pub(super) flags_override: Option<Box<Expr>>,
}

impl VNodeProps {
    /// Removes every children prop and returns the removed values in source order
    pub(super) fn remove_children_props(&mut self) -> Vec<Expr> {
        let mut removed = vec![];

        self.props.retain_mut(|item| {
            if !item.is_children {
                return true;
            }
            if let PropOrSpread::Prop(prop) = &mut item.prop
                && let Prop::KeyValue(KeyValueProp { value, .. }) = &mut **prop
            {
                removed.push(*std::mem::replace(value, Expr::undefined(DUMMY_SP)));
            }
            false
        });

        removed
    }

    pub(super) fn into_object(props: Vec<PropItem>) -> ObjectLit {
        ObjectLit {
            span: DUMMY_SP,
            props: props.into_iter().map(|item| item.prop).collect(),
        }
    }
}

fn attr_name(name: &JSXAttrName) -> Atom {
    match name {
        JSXAttrName::Ident(ident) => ident.sym.clone(),
        JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns, name, .. }) => {
            format!("{}:{}", ns.sym, name.sym).into()
        }
        #[cfg(swc_ast_unknown)]
        _ => panic!("unable to access unknown nodes"),
    }
}

fn prop_children(value: &Option<JSXAttrValue>) -> PropChildren {
    match value {
        Some(JSXAttrValue::Str(s)) => PropChildren::Str(s.value.clone()),
        Some(JSXAttrValue::JSXExprContainer(container)) => match &container.expr {
            JSXExpr::JSXEmptyExpr(_) => PropChildren::Empty,
            JSXExpr::Expr(expr) if matches!(expr.unwrap_parens(), Expr::Lit(Lit::Null(_))) => {
                PropChildren::Empty
            }
            _ => PropChildren::Expr,
        },
        Some(JSXAttrValue::JSXElement(_) | JSXAttrValue::JSXFragment(_)) => PropChildren::Expr,
        _ => PropChildren::Other,
    }
}

pub(super) fn get_vnode_props(attrs: Vec<JSXAttrOrSpread>, is_component: bool) -> VNodeProps {
    let mut result = VNodeProps::default();
    let mut hooks: Option<Vec<PropOrSpread>> = None;
    // Duplicates need two attributes, which most elements do not have
    let check_duplicates = attrs.len() > 1;
    let mut seen_props: FxHashMap<Atom, ()> = FxHashMap::default();
    let mut output_props: FxHashMap<Atom, Atom> = FxHashMap::default();

    for attr in attrs {
        let attr = match attr {
            JSXAttrOrSpread::SpreadElement(spread) => {
                result.needs_normalization = true;
                result.props.push(PropItem {
                    prop: PropOrSpread::Spread(spread),
                    is_children: false,
                });
                continue;
            }
            JSXAttrOrSpread::JSXAttr(attr) => attr,
            #[cfg(swc_ast_unknown)]
            _ => panic!("unable to access unknown nodes"),
        };
        let span = attr.span();
        let name = attr_name(&attr.name);

        if check_duplicates {
            if seen_props.contains_key(&name) {
                emit_error(
                    span,
                    &format!(
                        "Multiple {name} props are not supported. Remove the duplicate {name} prop."
                    ),
                );
                continue;
            }
            seen_props.insert(name.clone(), ());
        }

        // Adds a prop and rejects attributes that end up as the same prop, e.g. htmlFor and for
        let mut add_prop = |result: &mut VNodeProps,
                            output_name: &str,
                            value: Option<JSXAttrValue>| {
            if check_duplicates {
                if let Some(previous) = output_props.get(&Atom::from(output_name)) {
                    emit_error(
                        span,
                        &format!(
                            "{previous} and {name} both set the {output_name} prop. Remove one of \
                             them."
                        ),
                    );
                    return;
                }
                output_props.insert(output_name.into(), name.clone());
            }
            result.props.push(PropItem {
                prop: key_value(prop_name(output_name, span), get_value(value)),
                is_children: output_name == "children",
            });
        };

        if !is_component && (name == "className" || name == "class") {
            if result.class_name.is_some() {
                emit_error(
                    span,
                    "className and class both set the class name. Remove one of them.",
                );
                continue;
            }
            result.class_name = Some(get_value(attr.value));
        } else if let Some(output_name) = react_attribute(&name).filter(|_| !is_component) {
            add_prop(&mut result, output_name, attr.value);
        } else if !is_component && is_lowercase_attribute(&name) {
            add_prop(&mut result, &name.to_lowercase(), attr.value);
        } else if !is_component && name == "onDoubleClick" {
            add_prop(&mut result, "onDblClick", attr.value);
        } else if is_component && name.starts_with("onComponent") {
            hooks
                .get_or_insert_with(Vec::new)
                .push(key_value(prop_name(&name, span), get_value(attr.value)));
        } else if let Some(output_name) = svg_attribute(&name).filter(|_| !is_component) {
            // React compatibility for SVG attributes
            add_prop(&mut result, output_name, attr.value);
        } else {
            match &*name {
                "$ChildFlag" => {
                    result.children_known = true;
                    result.child_flags = Some(get_value(attr.value));
                }
                "$HasVNodeChildren" => result.children_known = true,
                "$Flags" => result.flags_override = Some(get_value(attr.value)),
                "$HasTextChildren" => {
                    result.children_known = true;
                    result.has_text_children = true;
                }
                "$HasNonKeyedChildren" => {
                    result.children_known = true;
                    result.has_non_keyed_children = true;
                }
                "$HasKeyedChildren" => {
                    result.children_known = true;
                    result.has_keyed_children = true;
                }
                "ref" => result.reference = Some(get_value(attr.value)),
                "key" => {
                    if attr.value.is_none() {
                        emit_error(
                            span,
                            "Please provide an explicit key value. Using \"key\" as a shorthand \
                             for \"key={true}\" is not allowed.",
                        );
                    }
                    result.key = Some(get_value(attr.value));
                }
                "$ReCreate" => result.has_re_create_flag = true,
                _ => {
                    if name == "children" {
                        result.prop_children = Some(prop_children(&attr.value));
                    }
                    if name.len() == 15 && name.eq_ignore_ascii_case("contenteditable") {
                        result.content_editable = true;
                    }
                    add_prop(&mut result, &name, attr.value);
                }
            }
        }
    }

    // Component hooks are passed in the ref argument; a ref attribute is merged in first so that
    // the hook attributes win regardless of their position
    if let Some(hooks) = hooks {
        let mut props = vec![];

        if let Some(reference) = result.reference.take() {
            props.push(PropOrSpread::Spread(SpreadElement {
                dot3_token: DUMMY_SP,
                expr: reference,
            }));
        }
        props.extend(hooks);
        result.reference = Some(Box::new(Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props,
        })));
    }

    result
}
