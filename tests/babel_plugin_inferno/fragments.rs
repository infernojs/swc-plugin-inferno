//! Ported from babel-plugin-inferno `tests/fragments.test.js`: Fragments

use crate::helpers::*;

/// keyed fragments
mod keyed_fragments {
    use super::*;

    #[test]
    fn should_create_a_keyed_empty_self_closing_fragment() {
        assert_transform(r#"<Fragment key="k"/>"#, r#"createFragment(null, 1, "k");"#);
    }

    #[test]
    fn should_create_a_keyed_empty_fragment() {
        assert_transform(
            r#"<Fragment key="k"></Fragment>"#,
            r#"createFragment(null, 1, "k");"#,
        );
    }

    #[test]
    fn should_create_a_keyed_fragment_with_text() {
        assert_transform(
            r#"<Fragment key="k">text</Fragment>"#,
            r#"createFragment([createTextVNode("text")], 4, "k");"#,
        );
    }

    #[test]
    fn should_mark_keyed_fragments_as_keyed_children() {
        assert_transform(
            r#"<div><Fragment key="a"><b/></Fragment><Fragment key="c"><d/></Fragment></div>"#,
            r#"createVNode(1, "div", null, [createFragment([createVNode(1, "b")], 4, "a"), createFragment([createVNode(1, "d")], 4, "c")], 8);"#,
        );
    }

    #[test]
    fn should_create_an_empty_fragment_with_has_keyed_children() {
        assert_transform("<Fragment $HasKeyedChildren/>", "createFragment();");
    }
}

/// $HasTextChildren
mod has_text_children {
    use super::*;

    // Not ported:
    // - Should not declare createTextVNode without children when imports are disabled (swc-plugin-inferno has no imports: false option)
    // - Should use pragmaTextVNode for a dynamic child (swc-plugin-inferno has no pragmaTextVNode option)

    #[test]
    fn should_compile_a_dynamic_child_into_a_text_vnode() {
        assert_js_eq(
            &transform_with("{}", r"<Fragment $HasTextChildren>{x}</Fragment>"),
            r#"import { createFragment, createTextVNode } from "inferno";
createFragment([createTextVNode(x)], 4);"#,
        );
    }

    #[test]
    fn should_compile_a_dynamic_child_into_a_text_vnode_in_a_keyed_fragment() {
        assert_transform(
            r#"<Fragment $HasTextChildren key="k">{x}</Fragment>"#,
            r#"createFragment([createTextVNode(x)], 4, "k");"#,
        );
    }

    #[test]
    fn should_compile_static_text_into_a_text_vnode() {
        assert_transform(
            r"<Fragment $HasTextChildren>text</Fragment>",
            r#"createFragment([createTextVNode("text")], 4);"#,
        );
    }

    #[test]
    fn should_put_a_string_expression_child_in_an_array() {
        assert_transform(
            r#"<Fragment $HasTextChildren>{"text"}</Fragment>"#,
            r#"createFragment([createTextVNode("text")], 4);"#,
        );
    }

    #[test]
    fn should_compile_a_children_prop_expression_into_a_text_vnode() {
        assert_transform(
            r"<Fragment $HasTextChildren children={x} />",
            r"createFragment([createTextVNode(x)], 4);",
        );
    }

    #[test]
    fn should_put_a_children_prop_string_in_an_array() {
        assert_transform(
            r#"<Fragment $HasTextChildren children="text" />"#,
            r#"createFragment([createTextVNode("text")], 4);"#,
        );
    }

    #[test]
    fn should_evaluate_an_overridden_children_prop_once() {
        assert_transform(
            r"<Fragment $HasTextChildren children={f()}>{x}</Fragment>",
            r"createFragment([(f(), createTextVNode(x))], 4);",
        );
    }

    #[test]
    fn should_keep_an_element_child_as_a_vnode() {
        assert_js_eq(
            &transform_with("{}", r"<Fragment $HasTextChildren><a/></Fragment>"),
            r#"import { createVNode, createFragment } from "inferno";
createFragment([createVNode(1, "a")], 4);"#,
        );
    }

    #[test]
    fn should_keep_an_element_expression_child_as_a_vnode() {
        assert_transform(
            r"<Fragment $HasTextChildren>{<a/>}</Fragment>",
            r#"createFragment([createVNode(1, "a")], 4);"#,
        );
    }

    #[test]
    fn should_keep_an_element_child_next_to_an_empty_expression_as_a_vnode() {
        assert_transform(
            r"<Fragment $HasTextChildren>{/* c */}<a/></Fragment>",
            r#"createFragment([createVNode(1, "a")], 4);"#,
        );
    }

    #[test]
    fn should_keep_an_element_children_prop_as_a_vnode() {
        assert_transform(
            r"<Fragment $HasTextChildren children=<a/> />",
            r#"createFragment([createVNode(1, "a")], 4);"#,
        );
    }

    #[test]
    fn should_not_import_create_text_vnode_without_children() {
        assert_js_eq(
            &transform_with("{}", r"<Fragment $HasTextChildren />"),
            r#"import { createFragment } from "inferno";
createFragment();"#,
        );
    }

    #[test]
    fn should_not_import_create_text_vnode_for_a_null_children_prop() {
        assert_js_eq(
            &transform_with("{}", r"<Fragment $HasTextChildren children={null} />"),
            r#"import { createFragment } from "inferno";
createFragment();"#,
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
            r"createFragment(x, 2);",
        );
    }

    #[test]
    fn should_pass_a_dynamic_child_as_a_single_vnode_in_a_keyed_fragment() {
        assert_transform(
            r#"<Fragment $HasVNodeChildren key="k">{x}</Fragment>"#,
            r#"createFragment(x, 2, "k");"#,
        );
    }

    #[test]
    fn should_pass_an_element_expression_child_as_a_single_vnode() {
        assert_transform(
            r"<Fragment $HasVNodeChildren>{<a/>}</Fragment>",
            r#"createFragment(createVNode(1, "a"), 2);"#,
        );
    }

    #[test]
    fn should_pass_an_element_child_next_to_an_empty_expression_as_a_single_vnode() {
        assert_transform(
            r"<Fragment $HasVNodeChildren>{/* c */}<a/></Fragment>",
            r#"createFragment(createVNode(1, "a"), 2);"#,
        );
    }

    #[test]
    fn should_evaluate_an_overridden_children_prop_before_a_dynamic_child() {
        assert_transform(
            r"<Fragment $HasVNodeChildren children={f()}>{x}</Fragment>",
            r"createFragment((f(), x), 2);",
        );
    }

    #[test]
    fn should_put_a_static_element_child_in_an_array() {
        assert_transform(
            r"<Fragment $HasVNodeChildren><a/></Fragment>",
            r#"createFragment([createVNode(1, "a")], 4);"#,
        );
    }

    #[test]
    fn should_compile_static_text_into_a_text_vnode() {
        assert_transform(
            r"<Fragment $HasVNodeChildren>text</Fragment>",
            r#"createFragment([createTextVNode("text")], 4);"#,
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
            r#"import { createFragment, createTextVNode } from "inferno";
createFragment([createTextVNode("text")], 4);"#,
        );
    }

    #[test]
    fn should_compile_a_children_prop_string_into_a_text_vnode_in_a_keyed_fragment() {
        assert_transform(
            r#"<Fragment children="text" key="k" />"#,
            r#"createFragment([createTextVNode("text")], 4, "k");"#,
        );
    }

    #[test]
    fn should_compile_a_children_prop_string_into_a_text_vnode_in_a_react_fragment() {
        assert_transform(
            r#"<React.Fragment children="text" />"#,
            r#"createFragment([createTextVNode("text")], 4);"#,
        );
    }

    #[test]
    fn should_keep_single_line_whitespace_in_a_children_prop_string() {
        assert_transform(
            r#"<Fragment children="  " />"#,
            r#"createFragment([createTextVNode("  ")], 4);"#,
        );
    }

    #[test]
    fn should_create_an_empty_fragment_for_an_empty_children_prop_string() {
        assert_js_eq(
            &transform_with("{}", r#"<Fragment children="" />"#),
            r#"import { createFragment } from "inferno";
createFragment();"#,
        );
    }

    #[test]
    fn should_put_a_children_prop_string_in_an_array_with_has_non_keyed_children() {
        assert_transform(
            r#"<Fragment $HasNonKeyedChildren children="text" />"#,
            r#"createFragment([createTextVNode("text")], 4);"#,
        );
    }

    #[test]
    fn should_put_a_children_prop_string_in_an_array_with_has_keyed_children() {
        assert_transform(
            r#"<Fragment $HasKeyedChildren children="text" />"#,
            r#"createFragment([createTextVNode("text")], 8);"#,
        );
    }

    #[test]
    fn should_compile_a_children_prop_string_into_a_text_vnode_with_has_vnode_children() {
        assert_transform(
            r#"<Fragment $HasVNodeChildren children="text" />"#,
            r#"createFragment([createTextVNode("text")], 4);"#,
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
            r#"createVNode(1, "div", null, createFragment(), 2);"#,
        );
    }

    #[test]
    fn should_compile_a_fragment_as_component_children() {
        assert_transform(
            "<Foo><>{a}</></Foo>",
            r"createComponentVNode(2, Foo, {
  children: createFragment(a, 0)
});",
        );
    }

    #[test]
    fn should_compile_a_fragment_inside_an_element_next_to_text() {
        assert_transform(
            "<div>text<>{a}</></div>",
            r#"createVNode(1, "div", null, [createTextVNode("text"), createFragment(a, 0)], 4);"#,
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
            r#"createFragment([createTextVNode("x")], 4);"#,
        );
    }

    #[test]
    fn should_drop_ref_on_fragment() {
        assert_transform(
            "<Fragment ref={r}>x</Fragment>",
            r#"createFragment([createTextVNode("x")], 4);"#,
        );
    }

    #[test]
    fn should_drop_other_props_on_react_fragment() {
        assert_transform(
            "<React.Fragment a={1}>x</React.Fragment>",
            r#"createFragment([createTextVNode("x")], 4);"#,
        );
    }

    #[test]
    fn should_wrap_a_dynamic_child_in_an_array_when_child_flag_is_an_expression() {
        assert_transform(
            "<Fragment $ChildFlag={x}>{a}</Fragment>",
            "createFragment([a], x);",
        );
    }

    #[test]
    fn should_leave_several_dynamic_children_declared_as_text_unwrapped_and_not_import_create_text_vnode()
     {
        assert_js_eq(
            &transform_with("{}", r"<Fragment $HasTextChildren>{x}{y}</Fragment>"),
            r#"import { createFragment } from "inferno";
createFragment([x, y], 4);"#,
        );
    }
}
