//! Reports compile-time flags that cannot change the compiled vNode, like `checkFlags` of
//! babel-plugin-inferno.

use super::props::PropChildren;
use super::vnode_args::is_empty_array;
use super::{ChildrenResult, VType};
use crate::inferno_flags::VNodeFlags;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error, Visitor},
};
use std::fmt::{self, Display};
use swc_core::{common::Span, plugin::errors::HANDLER};

/// What to do about compile-time flags that cannot change the compiled output, the
/// `uselessFlags` option
#[derive(Debug, Default, Clone, Copy, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UselessFlags {
    /// Report a warning
    #[default]
    Warn,
    /// Report an error, which fails the build
    Error,
    /// Report nothing
    Off,
}

impl<'de> Deserialize<'de> for UselessFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(LevelVisitor)
    }
}

/// Rejects any other value with the message of babel-plugin-inferno, so that a typo does not
/// turn the check off
struct LevelVisitor;

fn invalid_level<E: Error>(value: impl Display) -> E {
    E::custom(format_args!(
        "the uselessFlags option must be \"warn\", \"error\" or \"off\", got {value}."
    ))
}

impl Visitor<'_> for LevelVisitor {
    type Value = UselessFlags;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("\"warn\", \"error\" or \"off\"")
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<UselessFlags, E> {
        match value {
            "warn" => Ok(UselessFlags::Warn),
            "error" => Ok(UselessFlags::Error),
            "off" => Ok(UselessFlags::Off),
            _ => Err(invalid_level(format_args!("{value:?}"))),
        }
    }

    fn visit_bool<E: Error>(self, value: bool) -> Result<UselessFlags, E> {
        Err(invalid_level(value))
    }

    fn visit_i64<E: Error>(self, value: i64) -> Result<UselessFlags, E> {
        Err(invalid_level(value))
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<UselessFlags, E> {
        Err(invalid_level(value))
    }

    fn visit_f64<E: Error>(self, value: f64) -> Result<UselessFlags, E> {
        Err(invalid_level(value))
    }

    fn visit_unit<E: Error>(self) -> Result<UselessFlags, E> {
        Err(invalid_level("null"))
    }
}

/// A compile-time flag attribute. The child flags are in the order they take precedence over
/// each other.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum FlagAttr {
    ChildFlag,
    HasKeyedChildren,
    HasNonKeyedChildren,
    HasTextChildren,
    HasVNodeChildren,
    Flags,
    ReCreate,
}

impl FlagAttr {
    fn name(self) -> &'static str {
        match self {
            FlagAttr::ChildFlag => "$ChildFlag",
            FlagAttr::HasKeyedChildren => "$HasKeyedChildren",
            FlagAttr::HasNonKeyedChildren => "$HasNonKeyedChildren",
            FlagAttr::HasTextChildren => "$HasTextChildren",
            FlagAttr::HasVNodeChildren => "$HasVNodeChildren",
            FlagAttr::Flags => "$Flags",
            FlagAttr::ReCreate => "$ReCreate",
        }
    }

    fn is_child_flag(self) -> bool {
        !matches!(self, FlagAttr::Flags | FlagAttr::ReCreate)
    }
}

/// `isChildShapeKnown` of babel-plugin-inferno: whether the plugin sets the child flags itself
/// because it sees the shape of the children, as it does for static JSX children and for the
/// children props that createVNode compiles without normalization
fn is_child_shape_known(
    children: &ChildrenResult,
    prop_children: Option<&PropChildren>,
    is_fragment: bool,
) -> bool {
    if children.requires_normalization {
        return false;
    }
    // JSX children replace a children prop
    let Some(prop_children) = prop_children.filter(|_| is_empty_array(&children.children)) else {
        return true;
    };

    match prop_children {
        PropChildren::Str(_) | PropChildren::Empty | PropChildren::Other => true,
        // A fragment normalizes a JSX children prop at runtime
        PropChildren::Jsx => !is_fragment,
        PropChildren::Expr => false,
    }
}

fn report(level: UselessFlags, span: Span, message: &str) {
    HANDLER.with(|handler| {
        if level == UselessFlags::Error {
            handler.struct_span_err(span, message).emit();
        } else {
            handler.struct_span_warn(span, message).emit();
        }
    });
}

/// `checkFlags` of babel-plugin-inferno: reports the flags that cannot change the compiled
/// vNode. Each flag is reported once, for its first reason.
pub(super) fn check_flags(
    level: UselessFlags,
    flag_attrs: &[(FlagAttr, Span)],
    vtype: &VType,
    children: &ChildrenResult,
    prop_children: Option<&PropChildren>,
) {
    let is_component = matches!(vtype, VType::Component(_));
    let is_fragment = matches!(vtype, VType::Fragment);
    let has_flags_override = flag_attrs.iter().any(|&(flag, _)| flag == FlagAttr::Flags);
    // The child flag that the compiled vNode uses when the children are dynamic
    let winner = flag_attrs
        .iter()
        .map(|&(flag, _)| flag)
        .filter(|flag| flag.is_child_flag())
        .min();
    let shape_known = winner.is_some()
        && !is_component
        && is_child_shape_known(children, prop_children, is_fragment);

    for &(flag, span) in flag_attrs {
        let name = flag.name();
        let message = if !flag.is_child_flag() {
            if is_fragment {
                format!("{name} has no effect on Fragments.")
            } else if flag == FlagAttr::ReCreate && has_flags_override {
                format!(
                    "$ReCreate is ignored because $Flags replaces the vNode flags. Include \
                     ReCreate ({}) in $Flags instead.",
                    VNodeFlags::ReCreate as u16
                )
            } else {
                continue;
            }
        } else if is_component {
            format!(
                "{name} has no effect on components. Their children are passed in props.children."
            )
        } else if shape_known {
            format!(
                "{name} is not needed: the children are known at compile time, so the plugin sets \
                 their child flags. Child flags only help with dynamic children such as \
                 {{expression}}."
            )
        } else if let Some(winner) = winner.filter(|&winner| winner != flag) {
            format!(
                "{name} is ignored because {} takes precedence. Remove one of them.",
                winner.name()
            )
        } else {
            continue;
        };

        report(level, span, &message);
    }
}
