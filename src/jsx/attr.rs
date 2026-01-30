use swc_core::{
    atoms::wtf8::{Wtf8, Wtf8Buf},
    common::{Span, DUMMY_SP},
    ecma::ast::*,
};

use swc_core::plugin::errors::HANDLER;

pub(super) fn jsx_attr_value_to_expr(v: JSXAttrValue) -> Option<Box<Expr>> {
    Some(match v {
        JSXAttrValue::Str(s) => {
            let value = transform_jsx_attr_str(&s.value);

            Lit::Str(Str {
                span: s.span,
                raw: None,
                value: value.into(),
            })
            .into()
        }
        JSXAttrValue::JSXExprContainer(e) => match e.expr {
            JSXExpr::JSXEmptyExpr(_) => None?,
            JSXExpr::Expr(e) => e,
            #[cfg(swc_ast_unknown)]
            _ => panic!("unable to access unknown nodes"),
        },
        JSXAttrValue::JSXElement(e) => e.into(),
        JSXAttrValue::JSXFragment(f) => f.into(),
        #[cfg(swc_ast_unknown)]
        _ => panic!("unable to access unknown nodes"),
    })
}

pub(super) fn jsx_attr_value_to_expr_or_invalid(v: JSXAttrValue, err_span: Span) -> Box<Expr> {
    jsx_attr_value_to_expr(v).unwrap_or_else(|| {
        HANDLER.with(|handler| {
            handler
                .struct_span_err(err_span, "The value of JSX attribute should not be empty")
                .emit()
        });

        Box::new(Expr::Invalid(Invalid { span: DUMMY_SP }))
    })
}

fn transform_jsx_attr_str(v: &Wtf8) -> Wtf8Buf {
    let mut buf = Wtf8Buf::with_capacity(v.len());
    let mut iter = v.code_points().peekable();

    while let Some(code_point) = iter.next() {
        if let Some(c) = code_point.to_char() {
            match c {
                '\u{0008}' => buf.push_str("\\b"),
                '\u{000c}' => buf.push_str("\\f"),
                ' ' => buf.push_char(' '),

                '\n' | '\r' | '\t' => {
                    buf.push_char(' ');

                    while let Some(next) = iter.peek() {
                        if next.to_char() == Some(' ') {
                            iter.next();
                        } else {
                            break;
                        }
                    }
                }
                '\u{000b}' => buf.push_str("\\v"),
                '\0' => buf.push_str("\\x00"),

                '"' => buf.push_char('"'),

                '\x01'..='\x0f' | '\x10'..='\x1f' => {
                    buf.push_char(c);
                }

                '\x20'..='\x7e' => {
                    buf.push_char(c);
                }
                '\u{7f}'..='\u{ff}' => {
                    buf.push_char(c);
                }

                _ => {
                    buf.push_char(c);
                }
            }
        } else {
            buf.push(code_point);
        }
    }

    buf
}
