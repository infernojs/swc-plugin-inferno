//! Arguments of the generated calls, like `createVNodeArgs`, `createFragmentVNodeArgs`
//! and `createComponentVNodeArgs` of babel-plugin-inferno.
//!
//! `None` stands for an argument that was not given. babel-plugin-inferno also
//! treats an empty array literal as not given.

use crate::inferno_flags::ChildFlags;
use swc_core::{
    common::DUMMY_SP,
    ecma::ast::{ArrayLit, Expr, ExprOrSpread, Lit, Null, ObjectLit},
    ecma::utils::ExprFactory,
};

/// A flag argument: known at compile time, or given by `$Flags` or `$ChildFlag`.
pub(super) enum Flag {
    Known(u16),
    Expr(Box<Expr>),
}

impl From<ChildFlags> for Flag {
    fn from(flags: ChildFlags) -> Self {
        Flag::Known(flags as u16)
    }
}

impl Flag {
    fn is(&self, value: ChildFlags) -> bool {
        matches!(self, Flag::Known(flag) if *flag == value as u16)
    }

    fn into_arg(self) -> ExprOrSpread {
        match self {
            Flag::Known(flag) => num_arg(flag),
            Flag::Expr(expr) => expr.as_arg(),
        }
    }
}

fn null_arg() -> ExprOrSpread {
    Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))).as_arg()
}

fn num_arg(value: u16) -> ExprOrSpread {
    Box::new(Expr::from(f64::from(value))).as_arg()
}

fn invalid_children_arg() -> ExprOrSpread {
    num_arg(ChildFlags::HasInvalidChildren as u16)
}

pub(super) fn is_empty_array(expr: &Expr) -> bool {
    matches!(expr, Expr::Array(ArrayLit { elems, .. }) if elems.is_empty())
}

/// An optional argument, or `None` when it is not given, like `isAstNull` of babel-plugin-inferno
fn given(expr: Option<Box<Expr>>) -> Option<ExprOrSpread> {
    expr.filter(|expr| !is_empty_array(expr))
        .map(ExprFactory::as_arg)
}

/// An optional argument, and the default passed in its place when a later argument is given
type Optional = (Option<ExprOrSpread>, fn() -> ExprOrSpread);

/// Appends the optional arguments up to the last one given
fn push_optional<const N: usize>(args: &mut Vec<ExprOrSpread>, optional: [Optional; N]) {
    let count = optional
        .iter()
        .rposition(|(value, _)| value.is_some())
        .map_or(0, |last| last + 1);

    args.extend(
        optional
            .into_iter()
            .take(count)
            .map(|(value, default)| value.unwrap_or_else(default)),
    );
}

pub(super) struct CreateVNodeArgs {
    pub(super) flags: Flag,
    pub(super) tag: Box<Expr>,
    pub(super) class_name: Option<Box<Expr>>,
    pub(super) children: Option<Box<Expr>>,
    pub(super) child_flags: Flag,
    pub(super) props: ObjectLit,
    pub(super) key: Option<Box<Expr>>,
    pub(super) reference: Option<Box<Expr>>,
}

impl CreateVNodeArgs {
    pub(super) fn into_args(self) -> Vec<ExprOrSpread> {
        let child_flags = (!self.child_flags.is(ChildFlags::HasInvalidChildren))
            .then(|| self.child_flags.into_arg());
        let props = (!self.props.props.is_empty()).then(|| self.props.as_arg());
        let mut args = Vec::with_capacity(8);

        args.push(self.flags.into_arg());
        args.push(self.tag.as_arg());
        push_optional(
            &mut args,
            [
                (given(self.class_name), null_arg),
                (given(self.children), null_arg),
                (child_flags, invalid_children_arg),
                (props, null_arg),
                (given(self.key), null_arg),
                (given(self.reference), null_arg),
            ],
        );
        args
    }
}

pub(super) fn create_fragment_vnode_args(
    children: Option<Box<Expr>>,
    child_flags: Flag,
    key: Option<Box<Expr>>,
) -> Vec<ExprOrSpread> {
    let children = children
        .filter(|children| !is_empty_array(children))
        .map(|children| {
            if child_flags.is(ChildFlags::HasVNodeChildren)
                || child_flags.is(ChildFlags::HasNonKeyedChildren)
                || child_flags.is(ChildFlags::HasKeyedChildren)
                || child_flags.is(ChildFlags::UnknownChildren)
                || matches!(*children, Expr::Array(_))
            {
                children.as_arg()
            } else {
                Expr::Array(ArrayLit {
                    span: DUMMY_SP,
                    elems: vec![Some(children.as_arg())],
                })
                .as_arg()
            }
        });
    // Child flags only count with children
    let child_flags = (children.is_some() && !child_flags.is(ChildFlags::HasInvalidChildren))
        .then(|| child_flags.into_arg());
    let mut args = Vec::with_capacity(3);

    push_optional(
        &mut args,
        [
            (children, null_arg),
            (child_flags, invalid_children_arg),
            (given(key), null_arg),
        ],
    );
    args
}

pub(super) fn create_component_vnode_args(
    flags: Flag,
    tag: Box<Expr>,
    props: ObjectLit,
    key: Option<Box<Expr>>,
    reference: Option<Box<Expr>>,
) -> Vec<ExprOrSpread> {
    let props = (!props.props.is_empty()).then(|| props.as_arg());
    let mut args = Vec::with_capacity(5);

    args.push(flags.into_arg());
    args.push(tag.as_arg());
    push_optional(
        &mut args,
        [
            (props, null_arg),
            (given(key), null_arg),
            (given(reference), null_arg),
        ],
    );
    args
}
