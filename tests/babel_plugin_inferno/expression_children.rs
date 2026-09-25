//! Ported from babel-plugin-inferno `tests/expression-children.test.js`: Expression children

use crate::helpers::*;

/// empty expressions
mod empty_expressions {
    use super::*;

    #[test]
    fn should_create_no_children_for_a_component_with_only_a_comment() {
        assert_transform("<Foo>{/* c */}</Foo>", "createComponentVNode(2, Foo);");
    }

    #[test]
    fn should_create_an_empty_fragment_for_a_fragment_with_only_a_comment() {
        assert_transform("<>{/* c */}</>", "createFragment();");
    }

    #[test]
    fn should_ignore_a_comment_next_to_a_dynamic_child() {
        assert_transform(
            "<div>{/* c */}{a}</div>",
            r#"createVNode(1, "div", null, a, 0);"#,
        );
    }

    #[test]
    fn should_ignore_a_comment_between_dynamic_children() {
        assert_transform(
            "<div>{a}{/* x */}{b}</div>",
            r#"createVNode(1, "div", null, [a, b], 0);"#,
        );
    }

    #[test]
    fn should_ignore_a_comment_next_to_component_text() {
        assert_transform(
            "<Foo>{/* c */}text</Foo>",
            r#"createComponentVNode(2, Foo, {
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
            r#"createVNode(1, "div", null, "literal", 0);"#,
        );
    }

    #[test]
    fn should_pass_several_string_literal_children_as_is() {
        assert_transform(
            r#"<div>{"a"}{"b"}</div>"#,
            r#"createVNode(1, "div", null, ["a", "b"], 0);"#,
        );
    }

    #[test]
    fn should_pass_a_number_child() {
        assert_transform("<div>{1}</div>", r#"createVNode(1, "div", null, 1, 0);"#);
    }

    #[test]
    fn should_pass_a_null_child() {
        assert_transform(
            "<div>{null}</div>",
            r#"createVNode(1, "div", null, null, 0);"#,
        );
    }

    #[test]
    fn should_pass_an_undefined_child() {
        assert_transform(
            "<div>{undefined}</div>",
            r#"createVNode(1, "div", null, undefined, 0);"#,
        );
    }

    #[test]
    fn should_pass_a_boolean_child() {
        assert_transform(
            "<div>{true}</div>",
            r#"createVNode(1, "div", null, true, 0);"#,
        );
    }

    #[test]
    fn should_pass_a_template_literal_child() {
        assert_transform(
            "<div>{`tpl ${x}`}</div>",
            r#"createVNode(1, "div", null, `tpl ${x}`, 0);"#,
        );
    }

    #[test]
    fn should_pass_string_literals_containing_jsx_text_special_characters() {
        assert_transform(
            r#"<div>{">"}{"}"}</div>"#,
            r#"createVNode(1, "div", null, [">", "}"], 0);"#,
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
            r#"createVNode(1, "div", null, cond && createVNode(1, "span"), 0);"#,
        );
    }

    #[test]
    fn should_compile_jsx_inside_a_ternary() {
        assert_transform(
            "<div>{cond ? <a/> : <b/>}</div>",
            r#"createVNode(1, "div", null, cond ? createVNode(1, "a") : createVNode(1, "b"), 0);"#,
        );
    }

    #[test]
    fn should_compile_keyed_jsx_returned_from_map() {
        assert_transform(
            "<div>{list.map(i => <li key={i}>{i}</li>)}</div>",
            r#"createVNode(1, "div", null, list.map(i => createVNode(1, "li", null, i, 0, null, i)), 0);"#,
        );
    }

    #[test]
    fn should_compile_an_array_literal_of_keyed_jsx() {
        assert_transform(
            r#"<div>{[<a key="1"/>, <b key="2"/>]}</div>"#,
            r#"createVNode(1, "div", null, [createVNode(1, "a", null, null, 1, null, "1"), createVNode(1, "b", null, null, 1, null, "2")], 0);"#,
        );
    }

    #[test]
    fn should_compile_an_array_literal_of_unkeyed_components() {
        assert_transform(
            "<div>{[<C/>, <C/>]}</div>",
            r#"createVNode(1, "div", null, [createComponentVNode(2, C), createComponentVNode(2, C)], 0);"#,
        );
    }

    #[test]
    fn should_pass_a_function_as_component_children() {
        assert_transform(
            "<Foo>{(v) => <div>{v}</div>}</Foo>",
            r#"createComponentVNode(2, Foo, {
  children: v => createVNode(1, "div", null, v, 0)
});"#,
        );
    }

    #[test]
    fn should_pass_an_object_child_as_is() {
        assert_transform(
            "<div>{ {a} }</div>",
            r#"createVNode(1, "div", null, {
  a
}, 0);"#,
        );
    }

    #[test]
    fn should_compile_an_expression_container_holding_a_spread_element() {
        assert_transform(
            "<div>{<div {...test} />}</div>",
            r#"createVNode(1, "div", null, normalizeProps(createVNode(1, "div", null, null, 1, {
  ...test
})), 0);"#,
        );
    }

    #[test]
    fn should_keep_a_parenthesized_sequence_expression() {
        assert_transform(
            r#"<div>{(console.log("foo"), JSON.stringify(props))}</div>"#,
            r#"createVNode(1, "div", null, (console.log("foo"), JSON.stringify(props)), 0);"#,
        );
    }

    #[test]
    fn should_keep_optional_chaining_in_a_sequence_expression() {
        assert_transform(
            "<div>{(this?.class, this.class)}</div>",
            r#"createVNode(1, "div", null, (this?.class, this.class), 0);"#,
        );
    }
}

/// spread children
mod spread_children {
    use super::*;

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_spread_children_of_an_element() {
        assert_transform(
            "<div>{...children}</div>",
            r#"createVNode(1, "div", null, [...children], 0);"#,
        );
    }

    #[test]
    fn should_spread_children_of_a_component() {
        assert_transform(
            "<Foo>{...children}</Foo>",
            r"createComponentVNode(2, Foo, {
  children: [...children]
});",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_spread_children_of_a_fragment() {
        assert_transform("<>{...children}</>", "createFragment([...children], 0);");
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_spread_children_of_a_keyed_fragment() {
        assert_transform(
            r#"<Fragment key="k">{...a}</Fragment>"#,
            r#"createFragment([...a], 0, "k");"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_spread_several_children_in_order_oxc_spread_children_multiple_automatic() {
        assert_transform(
            "<div>{...[1, 2]}{...[3, 4]}</div>",
            r#"createVNode(1, "div", null, [...[1, 2], ...[3, 4]], 0);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_spread_children_around_a_static_element_oxc_spread_children_mixed_automatic() {
        assert_transform(
            "<div>{...a}<span/>{...b}</div>",
            r#"createVNode(1, "div", null, [...a, createVNode(1, "span"), ...b], 0);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_spread_a_jsx_element_child_babel_constant_elements() {
        assert_transform(
            "<div>{...<span/>}</div>",
            r#"createVNode(1, "div", null, [...createVNode(1, "span")], 0);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_spread_children_next_to_text() {
        assert_transform(
            "<div>text{...a}</div>",
            r#"createVNode(1, "div", null, [createTextVNode("text"), ...a], 0);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno wraps text in createTextVNode inside component children"]
    fn should_spread_component_children_next_to_text() {
        assert_transform(
            "<Foo>text{...a}</Foo>",
            r#"createComponentVNode(2, Foo, {
  children: ["text", ...a]
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_normalize_spread_children_next_to_a_keyed_child() {
        assert_transform(
            r#"<div><span key="k"/>{...a}</div>"#,
            r#"createVNode(1, "div", null, [createVNode(1, "span", null, null, 1, null, "k"), ...a], 0);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_use_the_child_flag_given_for_spread_children() {
        assert_transform(
            "<div $HasNonKeyedChildren>{...a}</div>",
            r#"createVNode(1, "div", null, [...a], 4);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not compile spread children as unknown children"]
    fn should_compile_spread_children_for_es5_targets() {
        assert_js_eq(
            &transform_with("{}", "<div>{...a}</div>"),
            r#"import { createVNode } from "inferno";
createVNode(1, "div", null, [...a], 0);"#,
        );
    }
}

/// children prop
mod children_prop {
    use super::*;

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_normalize_a_string_children_prop() {
        assert_transform(
            r#"<div children={"txt"} />"#,
            r#"createVNode(1, "div", null, "txt", 0);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_normalize_an_array_children_prop() {
        assert_transform(
            "<div children={[a, b]} />",
            r#"createVNode(1, "div", null, [a, b], 0);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_normalize_an_unknown_children_prop_expression() {
        assert_transform(
            "<div children={a} />",
            r#"createVNode(1, "div", null, a, 0);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_use_a_jsx_element_children_prop_given_without_braces() {
        assert_transform(
            "<div children=<span/> />",
            r#"createVNode(1, "div", null, createVNode(1, "span"), 2);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_use_a_jsx_fragment_children_prop_given_without_braces() {
        assert_transform(
            "<div children=<>{a}</> />",
            r#"createVNode(1, "div", null, createFragment(a, 0), 2);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_trust_has_vnode_children_for_a_children_prop_expression() {
        assert_transform(
            "<div $HasVNodeChildren children={a} />",
            r#"createVNode(1, "div", null, a, 2);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_normalize_a_fragment_children_prop_like_fragment_children() {
        assert_transform("<Fragment children={a} />", "createFragment(a, 0);");
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_normalize_a_jsx_fragment_children_prop() {
        assert_transform(
            "<Fragment children={<span/>} />",
            r#"createFragment(createVNode(1, "span"), 0);"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    // babel-plugin-inferno passes null children with unknown childFlags; both render nothing.
    #[test]
    fn should_drop_a_comment_child() {
        assert_transform("<div>{/* comment */}</div>", r#"createVNode(1, "div");"#);
    }

    // babel-plugin-inferno passes null children with unknown childFlags; both render nothing.
    #[test]
    fn should_drop_an_empty_expression_child() {
        assert_transform("<div>{}</div>", r#"createVNode(1, "div");"#);
    }

    // babel-plugin-inferno wraps the text in createTextVNode and marks it as unknown children.
    #[test]
    fn should_compile_text_next_to_a_comment_as_text_children() {
        assert_transform(
            "<div>{/* c */}text</div>",
            r#"createVNode(1, "div", null, "text", 16);"#,
        );
    }

    // babel-plugin-inferno marks the children as unknown, which Inferno normalizes at runtime.
    #[test]
    fn should_mark_static_siblings_around_a_comment_as_non_keyed_children() {
        assert_transform(
            "<div><span/>{/* c */}<span/></div>",
            r#"createVNode(1, "div", null, [
    createVNode(1, "span"),
    createVNode(1, "span")
], 4);"#,
        );
    }
}
