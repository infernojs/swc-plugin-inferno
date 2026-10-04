//! Rejects child flags that the children cannot have, like `checkChildFlagShape` of
//! babel-plugin-inferno. Inferno would render such children wrong, or throw in development.

use super::has_key_attr;
use super::props::emit_error;
use super::text::handle_white_space;
use super::useless_flags::FlagAttr;
use crate::inferno_flags::ChildFlags;
use swc_core::{
    common::Span,
    ecma::{ast::*, utils::number::ToJsString},
};

/// A child as far as the JSX shows it
#[derive(Clone, Copy)]
pub(super) enum Child {
    Text,
    /// `null`, `true` or `false`, which render nothing
    Empty(&'static str),
    /// An array literal, with the index of its first element or fragment item without a key
    Array {
        unkeyed_item: Option<usize>,
    },
    /// An element, a component or a fragment
    VNode {
        keyed: bool,
    },
    /// `{...children}`, which makes an array
    Spread,
    /// An expression whose value is only known at runtime
    Dynamic,
}

/// The expression inside parentheses and TypeScript syntax that does not change the value, e.g.
/// `(<a />)` or `<a /> as any`. swc strips the types before it runs the plugin, but a program that
/// runs the plugin first passes them in.
fn skip_outer_expressions(mut expr: &Expr) -> &Expr {
    loop {
        expr = match expr {
            Expr::Paren(ParenExpr { expr, .. })
            | Expr::TsAs(TsAsExpr { expr, .. })
            | Expr::TsConstAssertion(TsConstAssertion { expr, .. })
            | Expr::TsSatisfies(TsSatisfiesExpr { expr, .. })
            | Expr::TsNonNull(TsNonNullExpr { expr, .. })
            | Expr::TsTypeAssertion(TsTypeAssertion { expr, .. }) => expr,
            _ => return expr,
        };
    }
}

/// `expressionChild` of babel-plugin-inferno: the child that an expression gives
pub(super) fn expression_child(expr: &Expr) -> Child {
    match skip_outer_expressions(expr) {
        Expr::Lit(Lit::Str(_) | Lit::Num(_)) | Expr::Tpl(_) => Child::Text,
        Expr::Lit(Lit::Null(_)) => Child::Empty("null"),
        Expr::Lit(Lit::Bool(Bool { value, .. })) => {
            Child::Empty(if *value { "true" } else { "false" })
        }
        Expr::Array(array) => Child::Array {
            unkeyed_item: array.elems.iter().position(|element| {
                element.as_ref().is_some_and(|element| {
                    element.spread.is_none()
                        && match skip_outer_expressions(&element.expr) {
                            Expr::JSXElement(el) => !has_key_attr(el),
                            Expr::JSXFragment(_) => true,
                            _ => false,
                        }
                })
            }),
        },
        Expr::JSXElement(el) => Child::VNode {
            keyed: has_key_attr(el),
        },
        Expr::JSXFragment(_) => Child::VNode { keyed: false },
        _ => Child::Dynamic,
    }
}

/// The child that a `children` attribute gives, or `None` when it gives none
pub(super) fn prop_child(value: Option<&JSXAttrValue>) -> Option<Child> {
    match value {
        None => Some(Child::Empty("true")),
        Some(JSXAttrValue::Str(s)) => {
            (!handle_white_space(&s.value).is_empty()).then_some(Child::Text)
        }
        Some(JSXAttrValue::JSXExprContainer(container)) => match &container.expr {
            JSXExpr::Expr(expr) => Some(expression_child(expr)),
            _ => None,
        },
        Some(JSXAttrValue::JSXElement(el)) => Some(Child::VNode {
            keyed: has_key_attr(el),
        }),
        Some(_) => Some(Child::VNode { keyed: false }),
    }
}

/// `childrenShape` of babel-plugin-inferno: the children the vNode gets, as far as the JSX shows
/// them. JSX children replace a children prop.
pub(super) fn children_shape(
    children: &[JSXElementChild],
    prop_child: Option<Child>,
) -> Vec<Child> {
    let mut shape = Vec::with_capacity(children.len());

    for child in children {
        match child {
            JSXElementChild::JSXText(text) => {
                if !handle_white_space(&text.value).is_empty() {
                    shape.push(Child::Text);
                }
            }
            JSXElementChild::JSXExprContainer(JSXExprContainer {
                expr: JSXExpr::Expr(expr),
                ..
            }) => shape.push(expression_child(expr)),
            JSXElementChild::JSXExprContainer(_) => {}
            JSXElementChild::JSXSpreadChild(_) => shape.push(Child::Spread),
            JSXElementChild::JSXElement(el) => shape.push(Child::VNode {
                keyed: has_key_attr(el),
            }),
            JSXElementChild::JSXFragment(_) => shape.push(Child::VNode { keyed: false }),
            #[cfg(swc_ast_unknown)]
            _ => panic!("unable to access unknown nodes"),
        }
    }
    if shape.is_empty() {
        shape.extend(prop_child);
    }
    shape
}

fn describe_single_child(child: Child) -> String {
    match child {
        Child::Text => "the child is text".into(),
        Child::Empty(code) => format!("the child is {code}, which renders nothing"),
        Child::Array { .. } => "the child is an array".into(),
        Child::Spread => "the child is a spread, which makes an array".into(),
        Child::VNode { .. } | Child::Dynamic => "the child is an element".into(),
    }
}

/// `childShapeMismatch` of babel-plugin-inferno: why the children cannot have the shape the child
/// flags declare, or `None` when they can or the JSX does not show it
fn child_shape_mismatch(child_flags: ChildFlags, children: &[Child]) -> Option<String> {
    let count = children.len();

    if let ChildFlags::HasInvalidChildren = child_flags {
        let child = children
            .iter()
            .find(|child| !matches!(child, Child::Empty(_) | Child::Dynamic))?;

        return Some(if count == 1 {
            describe_single_child(*child)
        } else {
            format!("there are {count} children")
        });
    }
    let Some(&child) = children.first() else {
        return Some("there are no children".into());
    };

    if let ChildFlags::HasVNodeChildren | ChildFlags::HasTextChildren = child_flags {
        if count > 1 {
            return Some(format!("there are {count} children"));
        }
        return match (child_flags, child) {
            (_, Child::Dynamic)
            | (ChildFlags::HasVNodeChildren, Child::VNode { .. })
            | (ChildFlags::HasTextChildren, Child::Text) => None,
            _ => Some(describe_single_child(child)),
        };
    }
    // Keyed and non keyed children are an array
    let keyed = matches!(child_flags, ChildFlags::HasKeyedChildren);

    if count == 1 {
        return match child {
            Child::Dynamic | Child::Spread => None,
            Child::VNode { .. } => Some("the only child is an element, not an array".into()),
            Child::Array { unkeyed_item } => unkeyed_item
                .filter(|_| keyed)
                .map(|i| format!("the array item at index {i} has no key")),
            _ => Some(describe_single_child(child)),
        };
    }
    children
        .iter()
        .enumerate()
        .find_map(|(i, child)| match *child {
            Child::Empty(code) => Some(format!(
                "the child at index {i} is {code}, which renders nothing"
            )),
            Child::Array { .. } => Some(format!(
                "the child at index {i} is an array, which makes a nested array"
            )),
            Child::Text if keyed => {
                Some(format!("the child at index {i} is text, which has no key"))
            }
            Child::VNode { keyed: false } if keyed => {
                Some(format!("the child at index {i} has no key"))
            }
            _ => None,
        })
}

fn child_flags_name(child_flags: ChildFlags) -> &'static str {
    match child_flags {
        ChildFlags::UnknownChildren => "UnknownChildren",
        ChildFlags::HasInvalidChildren => "HasInvalidChildren",
        ChildFlags::HasVNodeChildren => "HasVNodeChildren",
        ChildFlags::HasNonKeyedChildren => "HasNonKeyedChildren",
        ChildFlags::HasKeyedChildren => "HasKeyedChildren",
        ChildFlags::HasTextChildren => "HasTextChildren",
    }
}

/// What the children of each child flag need to be
fn child_flags_needs(child_flags: ChildFlags) -> &'static str {
    match child_flags {
        ChildFlags::UnknownChildren => unreachable!("UnknownChildren children are not checked"),
        ChildFlags::HasInvalidChildren => "no children",
        ChildFlags::HasVNodeChildren => "one element or component child",
        ChildFlags::HasNonKeyedChildren => "an array of elements or components",
        ChildFlags::HasKeyedChildren => "an array of elements or components that all have a key",
        ChildFlags::HasTextChildren => "one text child",
    }
}

/// `checkChildFlagShape` of babel-plugin-inferno: reports an error at the child flag that the
/// vNode uses when the JSX shows that its children cannot have the declared shape. `child_flag`
/// is the value of `$ChildFlag`. Returns whether it reported an error.
pub(super) fn check_child_flag_shape(
    flag_attrs: &[(FlagAttr, Span)],
    child_flag: Option<&Expr>,
    children: &[Child],
) -> bool {
    let Some(&(winner, span)) = flag_attrs
        .iter()
        .filter(|(flag, _)| flag.is_child_flag())
        .min_by_key(|(flag, _)| *flag)
    else {
        return false;
    };
    let (child_flags, label) = match winner {
        FlagAttr::ChildFlag => {
            // An expression is only known at runtime
            let Some(Expr::Lit(Lit::Num(Number { value, .. }))) = child_flag else {
                return false;
            };
            let child_flags = ChildFlags::from_value(*value);
            let value = value.to_js_string();
            let Some(child_flags) = child_flags else {
                emit_error(
                    span,
                    &format!(
                        "$ChildFlag={{{value}}} is not a ChildFlags value. Use 0 (UnknownChildren), \
                         1 (HasInvalidChildren), 2 (HasVNodeChildren), 4 (HasNonKeyedChildren), \
                         8 (HasKeyedChildren) or 16 (HasTextChildren)."
                    ),
                );
                return true;
            };
            if let ChildFlags::UnknownChildren = child_flags {
                return false;
            }
            (
                child_flags,
                format!("$ChildFlag={{{value}}} ({})", child_flags_name(child_flags)),
            )
        }
        FlagAttr::HasKeyedChildren => (ChildFlags::HasKeyedChildren, winner.name().into()),
        FlagAttr::HasNonKeyedChildren => (ChildFlags::HasNonKeyedChildren, winner.name().into()),
        FlagAttr::HasTextChildren => (ChildFlags::HasTextChildren, winner.name().into()),
        FlagAttr::HasVNodeChildren => (ChildFlags::HasVNodeChildren, winner.name().into()),
        FlagAttr::Flags => return false,
    };
    let Some(mismatch) = child_shape_mismatch(child_flags, children) else {
        return false;
    };

    emit_error(
        span,
        &format!(
            "{label} needs {}, but {mismatch}.",
            child_flags_needs(child_flags)
        ),
    );
    true
}
