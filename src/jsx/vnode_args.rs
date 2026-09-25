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
    Box::new(Expr::from(value as f64)).as_arg()
}

/// `isAstNull` of babel-plugin-inferno
pub(super) fn is_ast_null(expr: &Option<Box<Expr>>) -> bool {
    match expr {
        None => true,
        Some(expr) => is_empty_array(expr),
    }
}

pub(super) fn is_empty_array(expr: &Expr) -> bool {
    matches!(expr, Expr::Array(ArrayLit { elems, .. }) if elems.is_empty())
}

fn push_or_null(args: &mut Vec<ExprOrSpread>, expr: Option<Box<Expr>>) {
    args.push(match expr {
        Some(expr) => expr.as_arg(),
        None => null_arg(),
    })
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
        let has_class_name = !is_ast_null(&self.class_name);
        let has_children = !is_ast_null(&self.children);
        let has_child_flags = !self.child_flags.is(ChildFlags::HasInvalidChildren);
        let has_props = !self.props.props.is_empty();
        let has_key = !is_ast_null(&self.key);
        let has_ref = !is_ast_null(&self.reference);
        let mut args = vec![self.flags.into_arg(), self.tag.as_arg()];

        if has_class_name {
            push_or_null(&mut args, self.class_name);
        } else if has_children || has_child_flags || has_props || has_key || has_ref {
            args.push(null_arg());
        }

        if has_children {
            push_or_null(&mut args, self.children);
        } else if has_child_flags || has_props || has_key || has_ref {
            args.push(null_arg());
        }

        if has_child_flags {
            args.push(self.child_flags.into_arg());
        } else if has_props || has_key || has_ref {
            args.push(num_arg(ChildFlags::HasInvalidChildren as u16));
        }

        if has_props {
            args.push(self.props.as_arg());
        } else if has_key || has_ref {
            args.push(null_arg());
        }

        if has_key {
            push_or_null(&mut args, self.key);
        } else if has_ref {
            args.push(null_arg());
        }

        if has_ref {
            push_or_null(&mut args, self.reference);
        }

        args
    }
}

pub(super) fn create_fragment_vnode_args(
    children: Option<Box<Expr>>,
    child_flags: Flag,
    key: Option<Box<Expr>>,
) -> Vec<ExprOrSpread> {
    let mut args = vec![];
    let has_children = !is_ast_null(&children);
    let has_child_flags = has_children && !child_flags.is(ChildFlags::HasInvalidChildren);
    let has_key = !is_ast_null(&key);

    if let Some(children) = children.filter(|_| has_children) {
        if child_flags.is(ChildFlags::HasNonKeyedChildren)
            || child_flags.is(ChildFlags::HasKeyedChildren)
            || child_flags.is(ChildFlags::UnknownChildren)
            || matches!(*children, Expr::Array(_))
        {
            args.push(children.as_arg());
        } else {
            args.push(
                Expr::Array(ArrayLit {
                    span: DUMMY_SP,
                    elems: vec![Some(children.as_arg())],
                })
                .as_arg(),
            );
        }
    } else if has_child_flags || has_key {
        args.push(null_arg());
    }

    if has_child_flags {
        args.push(child_flags.into_arg());
    } else if has_key {
        args.push(num_arg(ChildFlags::HasInvalidChildren as u16));
    }

    if has_key {
        push_or_null(&mut args, key);
    }

    args
}

pub(super) fn create_component_vnode_args(
    flags: Flag,
    tag: Box<Expr>,
    props: ObjectLit,
    key: Option<Box<Expr>>,
    reference: Option<Box<Expr>>,
) -> Vec<ExprOrSpread> {
    let has_props = !props.props.is_empty();
    let has_key = !is_ast_null(&key);
    let has_ref = !is_ast_null(&reference);
    let mut args = vec![flags.into_arg(), tag.as_arg()];

    if has_props {
        args.push(props.as_arg());
    } else if has_key || has_ref {
        args.push(null_arg());
    }

    if has_key {
        push_or_null(&mut args, key);
    } else if has_ref {
        args.push(null_arg());
    }

    if has_ref {
        push_or_null(&mut args, reference);
    }

    args
}
