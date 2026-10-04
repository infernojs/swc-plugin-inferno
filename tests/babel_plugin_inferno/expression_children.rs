//! Ported from babel-plugin-inferno `tests/expression-children.test.js`: Expression children

use crate::helpers::*;

/// empty expressions
mod empty_expressions {
    use super::*;

    #[test]
    fn should_create_no_children_for_a_component_with_only_a_comment() {
        assert_transform("<Foo>{/* c */}</Foo>", "newComponentVNode(0, Foo);");
    }

    #[test]
    fn should_create_an_empty_fragment_for_a_fragment_with_only_a_comment() {
        assert_transform("<>{/* c */}</>", "newFragment(272);");
    }

    #[test]
    fn should_ignore_a_comment_next_to_a_dynamic_child() {
        assert_transform("<div>{/* c */}{a}</div>", r#"newVNode(1, "div", null, a);"#);
    }

    #[test]
    fn should_ignore_a_comment_between_dynamic_children() {
        assert_transform(
            "<div>{a}{/* x */}{b}</div>",
            r#"newVNode(1, "div", null, [a, b]);"#,
        );
    }

    #[test]
    fn should_ignore_a_comment_next_to_component_text() {
        assert_transform(
            "<Foo>{/* c */}text</Foo>",
            r#"newComponentVNode(0, Foo, {
  children: "text"
});"#,
        );
    }
}

/// literal expressions
mod literal_expressions {
    use super::*;

    #[test]
    fn should_pass_a_string_literal_child_as_is() {
        assert_transform(
            r#"<div>{"literal"}</div>"#,
            r#"newVNode(1, "div", null, "literal");"#,
        );
    }

    #[test]
    fn should_pass_several_string_literal_children_as_is() {
        assert_transform(
            r#"<div>{"a"}{"b"}</div>"#,
            r#"newVNode(1, "div", null, ["a", "b"]);"#,
        );
    }

    #[test]
    fn should_pass_a_number_child() {
        assert_transform("<div>{1}</div>", r#"newVNode(1, "div", null, 1);"#);
    }

    #[test]
    fn should_omit_a_null_child() {
        assert_transform("<div>{null}</div>", r#"newVNode(1, "div");"#);
    }

    #[test]
    fn should_pass_an_undefined_child() {
        assert_transform(
            "<div>{undefined}</div>",
            r#"newVNode(1, "div", null, undefined);"#,
        );
    }

    #[test]
    fn should_pass_a_boolean_child() {
        assert_transform("<div>{true}</div>", r#"newVNode(1, "div", null, true);"#);
    }

    #[test]
    fn should_pass_a_template_literal_child() {
        assert_transform(
            "<div>{`tpl ${x}`}</div>",
            r#"newVNode(1, "div", null, `tpl ${x}`);"#,
        );
    }

    #[test]
    fn should_pass_string_literals_containing_jsx_text_special_characters() {
        assert_transform(
            r#"<div>{">"}{"}"}</div>"#,
            r#"newVNode(1, "div", null, [">", "}"]);"#,
        );
    }
}

/// dynamic expressions
mod dynamic_expressions {
    use super::*;

    #[test]
    fn should_compile_jsx_inside_a_logical_expression() {
        assert_transform(
            "<div>{cond && <span/>}</div>",
            r#"newVNode(1, "div", null, cond && newVNode(17, "span"));"#,
        );
    }

    #[test]
    fn should_compile_jsx_inside_a_ternary() {
        assert_transform(
            "<div>{cond ? <a/> : <b/>}</div>",
            r#"newVNode(1, "div", null, cond ? newVNode(17, "a") : newVNode(17, "b"));"#,
        );
    }

    #[test]
    fn should_compile_keyed_jsx_returned_from_map() {
        assert_transform(
            "<div>{list.map(i => <li key={i}>{i}</li>)}</div>",
            r#"newVNode(1, "div", null, list.map(i => newVNode(1, "li", null, i, null, i)));"#,
        );
    }

    #[test]
    fn should_compile_an_array_literal_of_keyed_jsx() {
        assert_transform(
            r#"<div>{[<a key="1"/>, <b key="2"/>]}</div>"#,
            r#"newVNode(1, "div", null, [newVNode(17, "a", null, null, null, "1"), newVNode(17, "b", null, null, null, "2")]);"#,
        );
    }

    #[test]
    fn should_compile_an_array_literal_of_unkeyed_components() {
        assert_transform(
            "<div>{[<C/>, <C/>]}</div>",
            r#"newVNode(1, "div", null, [newComponentVNode(0, C), newComponentVNode(0, C)]);"#,
        );
    }

    #[test]
    fn should_pass_a_function_as_component_children() {
        assert_transform(
            "<Foo>{(v) => <div>{v}</div>}</Foo>",
            r#"newComponentVNode(0, Foo, {
  children: v => newVNode(1, "div", null, v)
});"#,
        );
    }

    #[test]
    fn should_pass_an_object_child_as_is() {
        assert_transform(
            "<div>{ {a} }</div>",
            r#"newVNode(1, "div", null, {
  a
});"#,
        );
    }

    #[test]
    fn should_compile_an_expression_container_holding_a_spread_element() {
        assert_transform(
            "<div>{<div {...test} />}</div>",
            r#"newVNode(1, "div", null, normalizeProps(newVNode(17, "div", null, null, {
  ...test
})));"#,
        );
    }

    #[test]
    fn should_keep_a_parenthesized_sequence_expression() {
        assert_transform(
            r#"<div>{(console.log("foo"), JSON.stringify(props))}</div>"#,
            r#"newVNode(1, "div", null, (console.log("foo"), JSON.stringify(props)));"#,
        );
    }

    #[test]
    fn should_keep_optional_chaining_in_a_sequence_expression() {
        assert_transform(
            "<div>{(this?.class, this.class)}</div>",
            r#"newVNode(1, "div", null, (this?.class, this.class));"#,
        );
    }
}

/// spread children
mod spread_children {
    use super::*;

    #[test]
    fn should_spread_children_of_an_element() {
        assert_transform(
            "<div>{...children}</div>",
            r#"newVNode(1, "div", null, [...children]);"#,
        );
    }

    #[test]
    fn should_spread_children_of_a_component() {
        assert_transform(
            "<Foo>{...children}</Foo>",
            r"newComponentVNode(0, Foo, {
  children: [...children]
});",
        );
    }

    #[test]
    fn should_spread_children_of_a_fragment() {
        assert_transform("<>{...children}</>", "newFragment(256, [...children]);");
    }

    #[test]
    fn should_spread_children_of_a_keyed_fragment() {
        assert_transform(
            r#"<Fragment key="k">{...a}</Fragment>"#,
            r#"newFragment(256, [...a], "k");"#,
        );
    }

    #[test]
    fn should_spread_several_children_in_order_oxc_spread_children_multiple_automatic() {
        assert_transform(
            "<div>{...[1, 2]}{...[3, 4]}</div>",
            r#"newVNode(1, "div", null, [...[1, 2], ...[3, 4]]);"#,
        );
    }

    #[test]
    fn should_spread_children_around_a_static_element_oxc_spread_children_mixed_automatic() {
        assert_transform(
            "<div>{...a}<span/>{...b}</div>",
            r#"newVNode(1, "div", null, [...a, newVNode(17, "span"), ...b]);"#,
        );
    }

    #[test]
    fn should_spread_a_jsx_element_child_babel_constant_elements() {
        assert_transform(
            "<div>{...<span/>}</div>",
            r#"newVNode(1, "div", null, [...newVNode(17, "span")]);"#,
        );
    }

    #[test]
    fn should_spread_children_next_to_text() {
        assert_transform(
            "<div>text{...a}</div>",
            r#"newVNode(1, "div", null, [newTextVNode("text"), ...a]);"#,
        );
    }

    #[test]
    fn should_spread_component_children_next_to_text() {
        assert_transform(
            "<Foo>text{...a}</Foo>",
            r#"newComponentVNode(0, Foo, {
  children: ["text", ...a]
});"#,
        );
    }

    #[test]
    fn should_normalize_spread_children_next_to_a_keyed_child() {
        assert_transform(
            r#"<div><span key="k"/>{...a}</div>"#,
            r#"newVNode(1, "div", null, [newVNode(17, "span", null, null, null, "k"), ...a]);"#,
        );
    }

    #[test]
    fn should_use_the_child_flag_given_for_spread_children() {
        assert_transform(
            "<div $HasNonKeyedChildren>{...a}</div>",
            r#"newVNode(5, "div", null, [...a]);"#,
        );
    }

    #[test]
    fn should_compile_spread_children_for_es5_targets() {
        assert_js_eq(
            &transform_with("{}", "<div>{...a}</div>"),
            r#"import { newVNode } from "inferno";
newVNode(1, "div", null, [...a]);"#,
        );
    }
}

/// children prop
mod children_prop {
    use super::*;

    #[test]
    fn should_normalize_a_string_children_prop() {
        assert_transform(
            r#"<div children={"txt"} />"#,
            r#"newVNode(1, "div", null, "txt");"#,
        );
    }

    #[test]
    fn should_normalize_an_array_children_prop() {
        assert_transform(
            "<div children={[a, b]} />",
            r#"newVNode(1, "div", null, [a, b]);"#,
        );
    }

    #[test]
    fn should_normalize_an_unknown_children_prop_expression() {
        assert_transform("<div children={a} />", r#"newVNode(1, "div", null, a);"#);
    }

    #[test]
    fn should_use_a_jsx_element_children_prop_given_without_braces() {
        assert_transform(
            "<div children=<span/> />",
            r#"newVNode(9, "div", null, newVNode(17, "span"));"#,
        );
    }

    #[test]
    fn should_use_a_jsx_fragment_children_prop_given_without_braces() {
        assert_transform(
            "<div children=<>{a}</> />",
            r#"newVNode(9, "div", null, newFragment(256, a));"#,
        );
    }

    #[test]
    fn should_trust_has_vnode_children_for_a_children_prop_expression() {
        assert_transform(
            "<div $HasVNodeChildren children={a} />",
            r#"newVNode(9, "div", null, a);"#,
        );
    }

    #[test]
    fn should_normalize_a_fragment_children_prop_like_fragment_children() {
        assert_transform("<Fragment children={a} />", "newFragment(256, a);");
    }

    #[test]
    fn should_normalize_a_jsx_fragment_children_prop() {
        assert_transform(
            "<Fragment children={<span/>} />",
            r#"newFragment(256, newVNode(17, "span"));"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    #[test]
    fn should_mark_an_element_with_only_a_comment_child_as_unknown_children() {
        assert_transform("<div>{/* comment */}</div>", r#"newVNode(1, "div");"#);
    }

    #[test]
    fn should_mark_an_element_with_an_empty_expression_as_unknown_children() {
        assert_transform("<div>{}</div>", r#"newVNode(1, "div");"#);
    }

    #[test]
    fn should_wrap_text_next_to_a_comment_in_create_text_vnode_with_unknown_children() {
        assert_transform(
            "<div>{/* c */}text</div>",
            r#"newVNode(1, "div", null, newTextVNode("text"));"#,
        );
    }

    #[test]
    fn should_mark_static_siblings_around_a_comment_as_unknown_children() {
        assert_transform(
            "<div><span/>{/* c */}<span/></div>",
            r#"newVNode(1, "div", null, [newVNode(17, "span"), newVNode(17, "span")]);"#,
        );
    }
}
