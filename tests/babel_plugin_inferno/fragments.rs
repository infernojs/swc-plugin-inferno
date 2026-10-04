//! Ported from babel-plugin-inferno `tests/fragments.test.js`: Fragments

use crate::helpers::*;

/// keyed fragments
mod keyed_fragments {
    use super::*;

    #[test]
    fn should_create_a_keyed_empty_self_closing_fragment() {
        assert_transform(r#"<Fragment key="k"/>"#, r#"newFragment(272, null, "k");"#);
    }

    #[test]
    fn should_create_a_keyed_empty_fragment() {
        assert_transform(
            r#"<Fragment key="k"></Fragment>"#,
            r#"newFragment(272, null, "k");"#,
        );
    }

    #[test]
    fn should_create_a_keyed_fragment_with_text() {
        assert_transform(
            r#"<Fragment key="k">text</Fragment>"#,
            r#"newFragment(260, [newTextVNode("text")], "k");"#,
        );
    }

    #[test]
    fn should_mark_keyed_fragments_as_keyed_children() {
        assert_transform(
            r#"<div><Fragment key="a"><b/></Fragment><Fragment key="c"><d/></Fragment></div>"#,
            r#"newVNode(33, "div", null, [newFragment(260, [newVNode(17, "b")], "a"), newFragment(260, [newVNode(17, "d")], "c")]);"#,
        );
    }

    #[test]
    fn should_throw_for_an_empty_fragment_with_has_keyed_children() {
        assert_transform_error(
            r"<Fragment $HasKeyedChildren/>",
            "$HasKeyedChildren needs an array of elements or components that all have a key, but there are no children.",
        );
    }
}

/// $HasTextChildren
mod has_text_children {
    use super::*;

    // Not ported:
    // - Should use pragmaTextVNode for a dynamic child (swc-plugin-inferno has no pragmaTextVNode option)

    #[test]
    fn should_compile_a_dynamic_child_into_a_text_vnode() {
        assert_js_eq(
            &transform_with("{}", r"<Fragment $HasTextChildren>{x}</Fragment>"),
            r#"import { newFragment, newTextVNode } from "inferno";
newFragment(260, [newTextVNode(x)]);"#,
        );
    }

    #[test]
    fn should_compile_a_dynamic_child_into_a_text_vnode_in_a_keyed_fragment() {
        assert_transform(
            r#"<Fragment $HasTextChildren key="k">{x}</Fragment>"#,
            r#"newFragment(260, [newTextVNode(x)], "k");"#,
        );
    }

    #[test]
    fn should_compile_static_text_into_a_text_vnode() {
        assert_transform(
            r"<Fragment $HasTextChildren>text</Fragment>",
            r#"newFragment(260, [newTextVNode("text")]);"#,
        );
    }

    #[test]
    fn should_put_a_string_expression_child_in_an_array() {
        assert_transform(
            r#"<Fragment $HasTextChildren>{"text"}</Fragment>"#,
            r#"newFragment(260, [newTextVNode("text")]);"#,
        );
    }

    #[test]
    fn should_compile_a_children_prop_expression_into_a_text_vnode() {
        assert_transform(
            r"<Fragment $HasTextChildren children={x} />",
            r"newFragment(260, [newTextVNode(x)]);",
        );
    }

    #[test]
    fn should_put_a_children_prop_string_in_an_array() {
        assert_transform(
            r#"<Fragment $HasTextChildren children="text" />"#,
            r#"newFragment(260, [newTextVNode("text")]);"#,
        );
    }

    #[test]
    fn should_evaluate_an_overridden_children_prop_once() {
        assert_transform(
            r"<Fragment $HasTextChildren children={f()}>{x}</Fragment>",
            r"newFragment(260, [(f(), newTextVNode(x))]);",
        );
    }

    #[test]
    fn should_throw_for_has_text_children_on_an_element_child() {
        assert_transform_error(
            r"<Fragment $HasTextChildren><a/></Fragment>",
            "$HasTextChildren needs one text child, but the child is an element.",
        );
    }

    #[test]
    fn should_throw_for_has_text_children_on_an_element_expression_child() {
        assert_transform_error(
            r"<Fragment $HasTextChildren>{<a/>}</Fragment>",
            "$HasTextChildren needs one text child, but the child is an element.",
        );
    }

    #[test]
    fn should_throw_for_has_text_children_on_an_element_child_next_to_an_empty_expression() {
        assert_transform_error(
            r"<Fragment $HasTextChildren>{/* c */}<a/></Fragment>",
            "$HasTextChildren needs one text child, but the child is an element.",
        );
    }

    #[test]
    fn should_throw_for_has_text_children_on_an_element_children_prop() {
        assert_transform_error(
            r"<Fragment $HasTextChildren children=<a/> />",
            "$HasTextChildren needs one text child, but the child is an element.",
        );
    }

    #[test]
    fn should_throw_for_has_text_children_without_children() {
        assert_transform_error(
            r"<Fragment $HasTextChildren />",
            "$HasTextChildren needs one text child, but there are no children.",
        );
    }

    #[test]
    fn should_throw_for_has_text_children_on_a_null_children_prop() {
        assert_transform_error(
            r"<Fragment $HasTextChildren children={null} />",
            "$HasTextChildren needs one text child, but the child is null, which renders nothing.",
        );
    }

    #[test]
    fn should_throw_for_has_text_children_without_children_when_imports_are_disabled() {
        assert_transform_error(
            r"<Fragment $HasTextChildren />",
            "$HasTextChildren needs one text child, but there are no children.",
        );
    }
}

/// $HasVNodeChildren
mod has_vnode_children {
    use super::*;

    #[test]
    fn should_pass_a_dynamic_child_as_a_single_vnode() {
        assert_transform(
            r"<Fragment $HasVNodeChildren>{x}</Fragment>",
            r"newFragment(264, x);",
        );
    }

    #[test]
    fn should_pass_a_dynamic_child_as_a_single_vnode_in_a_keyed_fragment() {
        assert_transform(
            r#"<Fragment $HasVNodeChildren key="k">{x}</Fragment>"#,
            r#"newFragment(264, x, "k");"#,
        );
    }

    #[test]
    fn should_pass_an_element_expression_child_as_a_single_vnode() {
        assert_transform(
            r"<Fragment $HasVNodeChildren>{<a/>}</Fragment>",
            r#"newFragment(264, newVNode(17, "a"));"#,
        );
    }

    #[test]
    fn should_pass_an_element_child_next_to_an_empty_expression_as_a_single_vnode() {
        assert_transform(
            r"<Fragment $HasVNodeChildren>{/* c */}<a/></Fragment>",
            r#"newFragment(264, newVNode(17, "a"));"#,
        );
    }

    #[test]
    fn should_evaluate_an_overridden_children_prop_before_a_dynamic_child() {
        assert_transform(
            r"<Fragment $HasVNodeChildren children={f()}>{x}</Fragment>",
            r"newFragment(264, (f(), x));",
        );
    }

    #[test]
    fn should_put_a_static_element_child_in_an_array() {
        assert_transform(
            r"<Fragment $HasVNodeChildren><a/></Fragment>",
            r#"newFragment(260, [newVNode(17, "a")]);"#,
        );
    }

    #[test]
    fn should_throw_for_has_vnode_children_on_static_text() {
        assert_transform_error(
            r"<Fragment $HasVNodeChildren>text</Fragment>",
            "$HasVNodeChildren needs one element or component child, but the child is text.",
        );
    }
}

/// string children prop
mod string_children_prop {
    use super::*;

    #[test]
    fn should_compile_a_children_prop_string_into_a_text_vnode() {
        assert_js_eq(
            &transform_with("{}", r#"<Fragment children="text" />"#),
            r#"import { newFragment, newTextVNode } from "inferno";
newFragment(260, [newTextVNode("text")]);"#,
        );
    }

    #[test]
    fn should_compile_a_children_prop_string_into_a_text_vnode_in_a_keyed_fragment() {
        assert_transform(
            r#"<Fragment children="text" key="k" />"#,
            r#"newFragment(260, [newTextVNode("text")], "k");"#,
        );
    }

    #[test]
    fn should_compile_a_children_prop_string_into_a_text_vnode_in_a_react_fragment() {
        assert_transform(
            r#"<React.Fragment children="text" />"#,
            r#"newFragment(260, [newTextVNode("text")]);"#,
        );
    }

    #[test]
    fn should_keep_single_line_whitespace_in_a_children_prop_string() {
        assert_transform(
            r#"<Fragment children="  " />"#,
            r#"newFragment(260, [newTextVNode("  ")]);"#,
        );
    }

    #[test]
    fn should_create_an_empty_fragment_for_an_empty_children_prop_string() {
        assert_js_eq(
            &transform_with("{}", r#"<Fragment children="" />"#),
            r#"import { newFragment } from "inferno";
newFragment(272);"#,
        );
    }

    #[test]
    fn should_throw_for_a_children_prop_string_with_has_non_keyed_children() {
        assert_transform_error(
            r#"<Fragment $HasNonKeyedChildren children="text" />"#,
            "$HasNonKeyedChildren needs an array of elements or components, but the child is text.",
        );
    }

    #[test]
    fn should_throw_for_a_children_prop_string_with_has_keyed_children() {
        assert_transform_error(
            r#"<Fragment $HasKeyedChildren children="text" />"#,
            "$HasKeyedChildren needs an array of elements or components that all have a key, but the child is text.",
        );
    }

    #[test]
    fn should_throw_for_a_children_prop_string_with_has_vnode_children() {
        assert_transform_error(
            r#"<Fragment $HasVNodeChildren children="text" />"#,
            "$HasVNodeChildren needs one element or component child, but the child is text.",
        );
    }
}

/// fragment placement
mod fragment_placement {
    use super::*;

    #[test]
    fn should_compile_an_empty_react_fragment_inside_an_element() {
        assert_transform(
            "<div><React.Fragment /></div>",
            r#"newVNode(9, "div", null, newFragment(272));"#,
        );
    }

    #[test]
    fn should_compile_a_fragment_as_component_children() {
        assert_transform(
            "<Foo><>{a}</></Foo>",
            r"newComponentVNode(0, Foo, {
  children: newFragment(256, a)
});",
        );
    }

    #[test]
    fn should_compile_a_fragment_inside_an_element_next_to_text() {
        assert_transform(
            "<div>text<>{a}</></div>",
            r#"newVNode(5, "div", null, [newTextVNode("text"), newFragment(256, a)]);"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    #[test]
    fn should_drop_a_spread_on_fragment() {
        assert_transform(
            "<Fragment {...p}>x</Fragment>",
            r#"newFragment(260, [newTextVNode("x")]);"#,
        );
    }

    #[test]
    fn should_drop_ref_on_fragment() {
        assert_transform(
            "<Fragment ref={r}>x</Fragment>",
            r#"newFragment(260, [newTextVNode("x")]);"#,
        );
    }

    #[test]
    fn should_drop_other_props_on_react_fragment() {
        assert_transform(
            "<React.Fragment a={1}>x</React.Fragment>",
            r#"newFragment(260, [newTextVNode("x")]);"#,
        );
    }

    // The $ChildFlag declares the shape of the child as it is written, so the child is not put in
    // an array
    #[test]
    fn should_pass_a_dynamic_child_as_written_when_child_flag_is_an_expression() {
        assert_transform(
            "<Fragment $ChildFlag={x}>{a}</Fragment>",
            "createFragment(a, x);",
        );
        assert_transform(
            r#"<Fragment $ChildFlag={x} key="k">{a}</Fragment>"#,
            r#"createFragment(a, x, "k");"#,
        );
        assert_transform(
            "<Fragment $ChildFlag={x} children={a} />",
            "createFragment(a, x);",
        );
    }

    #[test]
    fn should_pass_a_dynamic_child_as_written_when_child_flag_is_a_number() {
        assert_transform(
            "<Fragment $ChildFlag={16}>{a}</Fragment>",
            "newFragment(258, a);",
        );
        assert_transform(
            "<Fragment $ChildFlag={1}>{a}</Fragment>",
            "newFragment(272, a);",
        );
    }

    #[test]
    fn should_compile_a_fragment_with_only_a_null_child_like_an_empty_one() {
        assert_transform("<Fragment>{null}</Fragment>", "newFragment(272);");
        assert_transform(
            "<Fragment key={null}>{a}</Fragment>",
            "newFragment(256, a);",
        );
    }

    #[test]
    fn should_still_put_a_static_child_in_an_array_when_child_flag_is_an_expression() {
        assert_transform(
            "<Fragment $ChildFlag={x}><a/></Fragment>",
            r#"createFragment([newVNode(17, "a")], x);"#,
        );
        assert_transform(
            "<Fragment $ChildFlag={x}>text</Fragment>",
            r#"createFragment([newTextVNode("text")], x);"#,
        );
    }

    #[test]
    fn should_throw_for_several_dynamic_children_declared_as_text() {
        assert_transform_error(
            r"<Fragment $HasTextChildren>{x}{y}</Fragment>",
            "$HasTextChildren needs one text child, but there are 2 children.",
        );
    }
}
