//! Arguments of the generated calls, like `newVNodeArgs`, `newFragmentArgs` and
//! `createComponentVNodeArgs` of babel-plugin-inferno, and `createVNodeArgs` and
//! `createFragmentVNodeArgs` for the deprecated factories.
//!
//! `None` stands for an argument that was not given. babel-plugin-inferno also
//! treats an empty array literal and a `null` literal as not given.

use crate::inferno_flags::{ChildFlags, VNodeFlags};
use swc_core::{
    common::DUMMY_SP,
    ecma::ast::{ArrayLit, BinExpr, BinaryOp, Expr, ExprOrSpread, Lit, Null, Number, ObjectLit},
    ecma::utils::{ExprFactory, to_int32},
};

/// A flag argument: known at compile time, or given by `$Flags`.
pub(super) enum Flag {
    Known(u32),
    Expr(Box<Expr>),
}

impl Flag {
    pub(super) fn into_expr(self) -> Box<Expr> {
        match self {
            Flag::Known(flag) => num(flag),
            Flag::Expr(expr) => expr,
        }
    }

    /// `packFlags` of babel-plugin-inferno: the flags of newVNode, newComponentVNode and
    /// newFragment hold the child bit. A number is folded into one literal, an expression like
    /// the value of `$Flags` gets the bit ORed in.
    pub(super) fn pack(self, child_bit: u32) -> Box<Expr> {
        match self {
            Flag::Known(flag) => num(flag | child_bit),
            Flag::Expr(expr) => {
                if let Expr::Lit(Lit::Num(Number { value, .. })) = *expr {
                    // JavaScript's | works on 32-bit integers
                    return Box::new(Expr::from(f64::from(to_int32(value) | child_bit as i32)));
                }
                if child_bit == 0 {
                    return expr;
                }
                // The fixer pass adds the parentheses that the expression needs
                Box::new(Expr::Bin(BinExpr {
                    span: DUMMY_SP,
                    op: BinaryOp::BitOr,
                    left: expr,
                    right: num(child_bit),
                }))
            }
        }
    }
}

fn null_arg() -> ExprOrSpread {
    Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))).as_arg()
}

fn num(value: u32) -> Box<Expr> {
    Box::new(Expr::from(f64::from(value)))
}

fn invalid_children_arg() -> ExprOrSpread {
    num(ChildFlags::HasInvalidChildren as u32).as_arg()
}

pub(super) fn is_empty_array(expr: &Expr) -> bool {
    matches!(expr, Expr::Array(ArrayLit { elems, .. }) if elems.is_empty())
}

/// An optional argument, or `None` when it is not given, like `isAstNull` of babel-plugin-inferno.
/// A `null` value is not passed either, like in ts-plugin-inferno: `<div className={null} />`
/// compiles like `<div />`.
fn given(expr: Option<Box<Expr>>) -> Option<ExprOrSpread> {
    expr.filter(|expr| !is_empty_array(expr) && !matches!(**expr, Expr::Lit(Lit::Null(_))))
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

/// The arguments of an element vNode
pub(super) struct VNodeArgs {
    pub(super) flags: Flag,
    pub(super) tag: Box<Expr>,
    pub(super) class_name: Option<Box<Expr>>,
    pub(super) children: Option<Box<Expr>>,
    pub(super) props: ObjectLit,
    pub(super) key: Option<Box<Expr>>,
    pub(super) reference: Option<Box<Expr>>,
}

impl VNodeArgs {
    /// `newVNodeArgs` of babel-plugin-inferno: `newVNode(flags, type, className, children, props,
    /// key, ref)` has no childFlags argument, the flags hold the child bit
    pub(super) fn into_new_vnode_args(self, child_flags: ChildFlags) -> Vec<ExprOrSpread> {
        let props = (!self.props.props.is_empty()).then(|| self.props.as_arg());
        let mut args = Vec::with_capacity(7);

        args.push(self.flags.pack(child_flags.child_bit()).as_arg());
        args.push(self.tag.as_arg());
        push_optional(
            &mut args,
            [
                (given(self.class_name), null_arg),
                (given(self.children), null_arg),
                (props, null_arg),
                (given(self.key), null_arg),
                (given(self.reference), null_arg),
            ],
        );
        args
    }

    /// `createVNodeArgs` of babel-plugin-inferno, for the deprecated createVNode that converts a
    /// `$ChildFlag` known only at runtime
    pub(super) fn into_create_vnode_args(self, child_flags: Box<Expr>) -> Vec<ExprOrSpread> {
        let props = (!self.props.props.is_empty()).then(|| self.props.as_arg());
        let mut args = Vec::with_capacity(8);

        args.push(self.flags.into_expr().as_arg());
        args.push(self.tag.as_arg());
        push_optional(
            &mut args,
            [
                (given(self.class_name), null_arg),
                (given(self.children), null_arg),
                (Some(child_flags.as_arg()), invalid_children_arg),
                (props, null_arg),
                (given(self.key), null_arg),
                (given(self.reference), null_arg),
            ],
        );
        args
    }
}

/// `newFragmentArgs` of babel-plugin-inferno: `newFragment(flags, children, key)`, the flags are
/// `VNodeFlags.Fragment` and the child bit
pub(super) fn new_fragment_args(
    children: Option<Box<Expr>>,
    child_flags: ChildFlags,
    key: Option<Box<Expr>>,
) -> Vec<ExprOrSpread> {
    // Static children are already in an array; a single dynamic child is passed as written, whatever
    // the child flags say, so that a $ChildFlag declares the shape of the value it is given
    let children = given(children);
    // A fragment without children gets an empty text vNode at runtime, like one that has invalid
    // children
    let child_flags = if children.is_some() {
        child_flags
    } else {
        ChildFlags::HasInvalidChildren
    };
    let mut args = Vec::with_capacity(3);

    args.push(num(VNodeFlags::Fragment as u32 | child_flags.child_bit()).as_arg());
    push_optional(&mut args, [(children, null_arg), (given(key), null_arg)]);
    args
}

/// `createFragmentVNodeArgs` of babel-plugin-inferno, for the deprecated createFragment that
/// converts a `$ChildFlag` known only at runtime
pub(super) fn create_fragment_args(
    children: Option<Box<Expr>>,
    child_flags: Box<Expr>,
    key: Option<Box<Expr>>,
) -> Vec<ExprOrSpread> {
    let children = given(children);
    // Child flags only count with children
    let child_flags = children.is_some().then(|| child_flags.as_arg());
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
    flags: Box<Expr>,
    tag: Box<Expr>,
    props: ObjectLit,
    key: Option<Box<Expr>>,
    reference: Option<Box<Expr>>,
) -> Vec<ExprOrSpread> {
    let props = (!props.props.is_empty()).then(|| props.as_arg());
    let mut args = Vec::with_capacity(5);

    args.push(flags.as_arg());
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
