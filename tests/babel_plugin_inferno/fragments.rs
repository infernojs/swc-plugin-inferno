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
}
