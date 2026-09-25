//! Ported from babel-plugin-inferno `tests/special-flags.test.js`: Special flags

use crate::helpers::*;

/// flag precedence
mod flag_precedence {
    use super::*;

    #[test]
    fn should_use_text_children_when_has_vnode_children_is_set_on_text() {
        assert_transform(
            "<div $HasVNodeChildren>text</div>",
            r#"createVNode(1, "div", null, "text", 16);"#,
        );
    }

    #[test]
    fn should_infer_non_keyed_children_when_has_vnode_children_is_set_on_several_children() {
        assert_transform(
            "<div $HasVNodeChildren><a/><b/></div>",
            r#"createVNode(1, "div", null, [createVNode(1, "a"), createVNode(1, "b")], 4);"#,
        );
    }

    #[test]
    fn should_use_has_keyed_children_for_static_children() {
        assert_transform(
            "<div $HasKeyedChildren><a/><b/></div>",
            r#"createVNode(1, "div", null, [createVNode(1, "a"), createVNode(1, "b")], 8);"#,
        );
    }

    #[test]
    fn should_prefer_has_keyed_children_over_has_non_keyed_children() {
        assert_transform(
            "<div $HasKeyedChildren $HasNonKeyedChildren>{a}</div>",
            r#"createVNode(1, "div", null, a, 8);"#,
        );
    }

    #[test]
    fn should_prefer_child_flag_over_other_child_flags() {
        assert_transform(
            "<div $ChildFlag={1} $HasKeyedChildren>{a}</div>",
            r#"createVNode(1, "div", null, a, 1);"#,
        );
    }

    #[test]
    fn should_prefer_flags_over_re_create_and_content_editable() {
        assert_transform(
            "<div $ReCreate contentEditable $Flags={9}/>",
            r#"createVNode(9, "div", null, null, 1, {
  "contentEditable": true
});"#,
        );
    }
}

/// $ReCreate
mod re_create {
    use super::*;

    #[test]
    fn should_add_the_re_create_flag_to_components() {
        assert_transform("<Foo $ReCreate/>", "createComponentVNode(2050, Foo);");
    }

    #[test]
    fn should_add_the_re_create_flag_to_input_elements() {
        assert_transform("<input $ReCreate/>", r#"createVNode(2112, "input");"#);
    }

    #[test]
    fn should_add_the_re_create_flag_to_svg_elements() {
        assert_transform("<svg $ReCreate/>", r#"createVNode(2080, "svg");"#);
    }

    #[test]
    fn should_combine_re_create_and_content_editable_flags() {
        assert_transform(
            "<div $ReCreate contentEditable/>",
            r#"createVNode(6145, "div", null, null, 1, {
  "contentEditable": true
});"#,
        );
    }
}

/// other combinations
mod other_combinations {
    use super::*;

    #[test]
    fn should_set_text_children_flag_without_children() {
        assert_transform(
            "<div $HasTextChildren />",
            r#"createVNode(1, "div", null, null, 16);"#,
        );
    }

    #[test]
    fn should_ignore_child_flags_on_components() {
        assert_transform(
            "<Foo $HasKeyedChildren>{a}</Foo>",
            r"createComponentVNode(2, Foo, {
  children: a
});",
        );
    }

    #[test]
    fn should_keep_flags_with_a_spread() {
        assert_transform(
            "<div $Flags={1} {...p}/>",
            r#"normalizeProps(createVNode(1, "div", null, null, 1, {
  ...p
}));"#,
        );
    }

    #[test]
    fn should_keep_has_vnode_children_with_a_spread() {
        assert_transform(
            "<div {...p} $HasVNodeChildren>{a}</div>",
            r#"normalizeProps(createVNode(1, "div", null, a, 2, {
  ...p
}));"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    #[test]
    fn should_add_the_content_editable_flag_to_components() {
        assert_transform(
            "<Foo contentEditable/>",
            r#"createComponentVNode(4098, Foo, {
  "contentEditable": true
});"#,
        );
    }

    #[test]
    fn should_let_inferred_non_keyed_children_override_has_text_children() {
        assert_transform(
            "<div $HasTextChildren><a/><b/></div>",
            r#"createVNode(1, "div", null, [createVNode(1, "a"), createVNode(1, "b")], 4);"#,
        );
    }

    #[test]
    fn should_pass_a_string_child_flag_through_as_a_string() {
        assert_transform(
            r#"<div $ChildFlag="1">{a}</div>"#,
            r#"createVNode(1, "div", null, a, "1");"#,
        );
    }
}
