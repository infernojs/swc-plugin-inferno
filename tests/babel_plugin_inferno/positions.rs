//! Ported from babel-plugin-inferno `tests/positions.test.js`: JSX positions

use crate::helpers::*;

/// expression positions
mod expression_positions {
    use super::*;

    #[test]
    fn should_compile_jsx_in_default_parameters() {
        assert_transform(
            "function F({x = <b/>}) { return <div>{x}</div>; }",
            r#"function F({
  x = newVNode(17, "b")
}) {
  return newVNode(1, "div", null, x);
}"#,
        );
    }

    #[test]
    fn should_compile_jsx_in_static_class_fields() {
        assert_transform(
            "class C { static el = <i/>; }",
            r#"class C {
  static el = newVNode(17, "i");
}"#,
        );
    }

    #[test]
    fn should_compile_jsx_in_a_class_field_arrow_function() {
        assert_transform(
            "class A { render = () => <this.subComponent />; }",
            r"class A {
  render = () => newComponentVNode(0, this.subComponent);
}",
        );
    }

    #[test]
    fn should_compile_jsx_in_a_default_export() {
        assert_transform(
            "export default () => <div/>;",
            r#"export default () => newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_compile_jsx_in_a_named_export() {
        assert_transform(
            "export const A = () => <div/>;",
            r#"export const A = () => newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_compile_jsx_inside_higher_order_component_calls() {
        assert_transform(
            "const C = memo(forwardRef((props, ref) => <div ref={ref}/>));",
            r#"const C = memo(forwardRef((props, ref) => newVNode(17, "div", null, null, null, null, ref)));"#,
        );
    }

    #[test]
    fn should_compile_several_top_level_jsx_expression_statements() {
        assert_transform(
            r"<div>{a}</div>;
<span>{b}</span>",
            r#"newVNode(1, "div", null, a);
newVNode(1, "span", null, b);"#,
        );
    }

    #[test]
    fn should_compile_jsx_used_as_a_component_variable() {
        assert_transform(
            r"let Foo = <div />;
<Foo />;",
            r#"let Foo = newVNode(17, "div");
newComponentVNode(0, Foo);"#,
        );
    }
}

/// JSX created by other plugins
mod jsx_created_by_other_plugins {
    use super::*;

    use swc_core::{
        common::DUMMY_SP,
        ecma::{
            ast::*,
            visit::{VisitMut, VisitMutWith, visit_mut_pass},
        },
    };

    /// Replaces `makeJSX()` with `<div title="generated">text</div>` built without spans.
    struct MakeJsx;

    impl VisitMut for MakeJsx {
        fn visit_mut_expr(&mut self, expr: &mut Expr) {
            expr.visit_mut_children_with(self);

            let Expr::Call(CallExpr {
                callee: Callee::Expr(callee),
                ..
            }) = expr
            else {
                return;
            };
            if !matches!(&**callee, Expr::Ident(i) if i.sym == "makeJSX") {
                return;
            }

            let name = |sym: &str| JSXElementName::Ident(Ident::new_no_ctxt(sym.into(), DUMMY_SP));

            *expr = Expr::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: name("div"),
                    attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName::new("title".into(), DUMMY_SP)),
                        value: Some(JSXAttrValue::Str(Str {
                            span: DUMMY_SP,
                            value: "generated".into(),
                            raw: None,
                        })),
                    })],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: "text".into(),
                    raw: "text".into(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: name("div"),
                }),
            }));
        }
    }

    #[test]
    fn should_compile_jsx_nodes_built_without_source_locations() {
        let compiled = compile_with(
            Setup::default(),
            "const el = makeJSX();",
            visit_mut_pass(MakeJsx),
        )
        .unwrap();

        assert_js_eq(
            &strip_inferno_import(&compiled.code),
            "const el = newVNode(3, \"div\", null, \"text\", {\n  \"title\": \"generated\"\n});",
        );
    }
}
