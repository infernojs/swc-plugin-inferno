use crate::inferno_flags::ChildFlags;
use swc_core::{
    common::{util::take::Take, DUMMY_SP},
    ecma::ast::{ArrayLit, Expr, ExprOrSpread, Lit, Null, Number, ObjectLit},
    ecma::utils::ExprFactory,
};

#[inline(always)]
fn null_arg() -> ExprOrSpread {
    Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))).as_arg()
}

#[inline(always)]
fn u16_as_num_arg(value: u16) -> ExprOrSpread {
    Box::new(Expr::Lit(Lit::Num(Number {
        span: DUMMY_SP,
        raw: None,
        value: value as f64,
    })))
    .as_arg()
}

pub(super) struct CreateVNodeArgs {
    pub(super) flags: ExprOrSpread,
    pub(super) name: Expr,
    pub(super) class_name: Option<Box<Expr>>,
    pub(super) children: Vec<Option<ExprOrSpread>>,
    pub(super) child_flags: u16,
    pub(super) child_flags_override_param: Option<ExprOrSpread>,
    pub(super) props: ObjectLit,
    pub(super) key: Option<ExprOrSpread>,
    pub(super) refs: Option<ExprOrSpread>,
}

impl CreateVNodeArgs {
    #[inline(always)]
    pub(super) fn into_args(self) -> Vec<ExprOrSpread> {
        let CreateVNodeArgs {
            flags,
            name,
            class_name,
            mut children,
            child_flags,
            child_flags_override_param,
            props,
            key,
            refs,
        } = self;

        let mut args: Vec<ExprOrSpread> = Vec::with_capacity(8);
        args.push(flags);
        args.push(name.as_arg());

        let has_children = !children.is_empty();
        let has_child_flags = child_flags_override_param.is_some()
            || child_flags != (ChildFlags::HasInvalidChildren as u16);
        let has_props = !props.props.is_empty();
        let has_key = key.is_some();
        let has_ref = refs.is_some();

        match class_name {
            None => {
                if has_children || has_child_flags || has_props || has_key || has_ref {
                    args.push(null_arg());
                }
            }
            Some(some_class_name) => {
                args.push(some_class_name.as_arg());
            }
        }

        match children.len() {
            0 => {
                if has_child_flags || has_props || has_key || has_ref {
                    args.push(null_arg());
                }
            }
            1 => {
                let only_child = children.take().into_iter().next().flatten();
                match only_child {
                    Some(child) => args.push(child.expr.as_arg()),
                    None => args.push(
                        Box::new(Expr::Array(ArrayLit {
                            span: DUMMY_SP,
                            elems: vec![None],
                        }))
                        .as_arg(),
                    ),
                }
            }
            _ => args.push(
                Box::new(Expr::Array(ArrayLit {
                    span: DUMMY_SP,
                    elems: children.take(),
                }))
                .as_arg(),
            ),
        }

        if has_child_flags {
            match child_flags_override_param {
                Some(some_child_flags_override_param) => {
                    args.push(some_child_flags_override_param);
                }
                None => args.push(u16_as_num_arg(child_flags)),
            }
        } else if has_props || has_key || has_ref {
            args.push(u16_as_num_arg(ChildFlags::HasInvalidChildren as u16));
        }

        if has_props {
            args.push(props.as_arg());
        } else if has_key || has_ref {
            args.push(null_arg());
        }

        match key {
            None => {
                if has_ref {
                    args.push(null_arg());
                }
            }
            Some(some_key) => {
                args.push(some_key);
            }
        }

        if let Some(some_refs) = refs {
            args.push(some_refs);
        }

        args
    }
}

#[inline(always)]
pub(super) fn create_component_vnode_args(
    flags: ExprOrSpread,
    name: Expr,
    props_literal: ObjectLit,
    key: Option<ExprOrSpread>,
    refs: Option<ExprOrSpread>,
) -> Vec<ExprOrSpread> {
    let mut args: Vec<ExprOrSpread> = Vec::with_capacity(5);
    args.push(flags);
    args.push(name.as_arg());

    if props_literal.props.is_empty() {
        if key.is_some() || refs.is_some() {
            args.push(null_arg());
        }
    } else {
        args.push(props_literal.as_arg());
    }

    match key {
        None => {
            if refs.is_some() {
                args.push(null_arg());
            }
        }
        Some(some_key) => {
            args.push(some_key);
        }
    }

    if let Some(some_ref) = refs {
        args.push(some_ref);
    }

    args
}

#[inline(always)]
pub(super) fn create_fragment_vnode_args(
    mut children: Vec<Option<ExprOrSpread>>,
    children_shape_is_user_defined: bool,
    child_flags: u16,
    child_flags_override_param: Option<ExprOrSpread>,
    key: Option<ExprOrSpread>,
) -> Vec<ExprOrSpread> {
    let mut args: Vec<ExprOrSpread> = Vec::with_capacity(3);
    let has_child_flags = child_flags_override_param.is_some()
        || child_flags != (ChildFlags::HasInvalidChildren as u16);
    let has_key = key.is_some();

    match children.len() {
        0 => {
            if has_child_flags || has_key {
                args.push(null_arg());
            }
        }
        1 => {
            if children_shape_is_user_defined || child_flags == ChildFlags::UnknownChildren as u16 {
                let only_child = children.take().into_iter().next().flatten();
                match only_child {
                    Some(child) => args.push(child.expr.as_arg()),
                    None => args.push(
                        Box::new(Expr::Array(ArrayLit {
                            span: DUMMY_SP,
                            elems: vec![None],
                        }))
                        .as_arg(),
                    ),
                }
            } else {
                args.push(
                    Box::new(Expr::Array(ArrayLit {
                        span: DUMMY_SP,
                        elems: children.take(),
                    }))
                    .as_arg(),
                );
            }
        }
        _ => args.push(
            Box::new(Expr::Array(ArrayLit {
                span: DUMMY_SP,
                elems: children.take(),
            }))
            .as_arg(),
        ),
    }

    if has_child_flags {
        match child_flags_override_param {
            Some(some_child_flags_override_param) => {
                args.push(some_child_flags_override_param);
            }
            None => args.push(u16_as_num_arg(child_flags)),
        }
    } else if has_key {
        args.push(u16_as_num_arg(ChildFlags::HasInvalidChildren as u16));
    }

    if let Some(some_key) = key {
        args.push(some_key);
    }

    args
}
