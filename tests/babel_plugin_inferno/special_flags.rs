//! Ported from babel-plugin-inferno `tests/special-flags.test.js`: Special flags

use crate::helpers::*;

/// flag precedence
mod flag_precedence {
    use super::*;

    #[test]
    fn should_throw_for_has_vnode_children_on_text() {
        assert_transform_error(
            "<div $HasVNodeChildren>text</div>",
            "$HasVNodeChildren needs one element or component child, but the child is text.",
        );
    }

    #[test]
    fn should_throw_for_has_vnode_children_on_several_children() {
        assert_transform_error(
            "<div $HasVNodeChildren><a/><b/></div>",
            "$HasVNodeChildren needs one element or component child, but there are 2 children.",
        );
    }

    #[test]
    fn should_use_has_keyed_children_for_static_children() {
        assert_transform(
            r#"<div $HasKeyedChildren><a key="1"/><b key="2"/></div>"#,
            r#"newVNode(33, "div", null, [newVNode(17, "a", null, null, null, "1"), newVNode(17, "b", null, null, null, "2")]);"#,
        );
    }

    #[test]
    fn should_throw_for_has_keyed_children_on_static_children_without_keys() {
        assert_transform_error(
            "<div $HasKeyedChildren><a/><b/></div>",
            "$HasKeyedChildren needs an array of elements or components that all have a key, but the child at index 0 has no key.",
        );
    }

    #[test]
    fn should_prefer_has_keyed_children_over_has_non_keyed_children() {
        assert_transform(
            "<div $HasKeyedChildren $HasNonKeyedChildren>{a}</div>",
            r#"newVNode(33, "div", null, a);"#,
        );
    }

    #[test]
    fn should_prefer_child_flag_over_other_child_flags() {
        assert_transform(
            "<div $ChildFlag={1} $HasKeyedChildren>{a}</div>",
            r#"newVNode(17, "div", null, a);"#,
        );
    }

    #[test]
    fn should_prefer_flags_over_content_editable() {
        assert_transform(
            "<div contentEditable $Flags={9}/>",
            r#"newVNode(25, "div", null, null, {
  "contentEditable": true
});"#,
        );
    }
}

/// child flags that the children cannot have
mod child_flags_that_the_children_cannot_have {
    use super::*;

    const OFF: &str = r#"{"uselessFlags": "off"}"#;

    /// The errors of compiling `input`, or `None` when it compiles
    fn error_with(options: &str, input: &str) -> Option<String> {
        compile(
            Setup {
                options,
                ..Default::default()
            },
            input,
        )
        .err()
    }

    fn error(input: &str) -> Option<String> {
        error_with(OFF, input)
    }

    #[track_caller]
    fn assert_error(input: &str, message: &str) {
        assert_compile_error(
            Setup {
                options: OFF,
                ..Default::default()
            },
            input,
            message,
        );
    }

    // babel-plugin-inferno prints its code frame. swc prints its own, which points at the same
    // code.
    #[test]
    fn should_point_at_the_flag_with_a_code_frame() {
        let error = error(
            "function App() {\n  return (\n    <ul $HasKeyedChildren>\n      <li/>\n    </ul>\n  );\n}",
        )
        .unwrap();

        assert!(error.contains("$HasKeyedChildren needs an array of elements or components that all have a key, but the only child is an element, not an array."), "{error}");
        assert!(
            error.contains(" 3 |     <ul $HasKeyedChildren>\n   :         ^^^^^^^^^^^^^^^^^\n"),
            "{error}"
        );
    }

    #[test]
    fn should_throw_whatever_the_useless_flags_option_is() {
        let input = "<div $HasVNodeChildren>{[a, b]}</div>";
        let message =
            "$HasVNodeChildren needs one element or component child, but the child is an array.";

        for options in [
            r#"{"uselessFlags": "off"}"#,
            r#"{"uselessFlags": "warn"}"#,
            r#"{"uselessFlags": "error"}"#,
        ] {
            let error = error_with(options, input).unwrap();

            assert!(error.contains(message), "{options}: {error}");
        }
    }

    #[test]
    fn should_throw_for_an_array_children_prop_with_has_vnode_children() {
        assert_error(
            "<div $HasVNodeChildren children={[a, b]} />",
            "$HasVNodeChildren needs one element or component child, but the child is an array.",
        );
    }

    #[test]
    fn should_throw_for_literal_text_with_has_vnode_children() {
        assert_error(
            "<div $HasVNodeChildren>{`a${b}`}</div>",
            "$HasVNodeChildren needs one element or component child, but the child is text.",
        );
        assert_error(
            "<div $HasVNodeChildren>{1}</div>",
            "$HasVNodeChildren needs one element or component child, but the child is text.",
        );
    }

    #[test]
    fn should_throw_for_a_boolean_child_with_has_text_children() {
        assert_error(
            "<div $HasTextChildren>{true}</div>",
            "$HasTextChildren needs one text child, but the child is true, which renders nothing.",
        );
    }

    #[test]
    fn should_throw_for_an_array_item_without_a_key_with_has_keyed_children() {
        assert_error(
            r#"<div $HasKeyedChildren>{[<a key="1"/>, <b/>]}</div>"#,
            "$HasKeyedChildren needs an array of elements or components that all have a key, but the array item at index 1 has no key.",
        );
    }

    #[test]
    fn should_not_throw_for_an_array_of_keyed_items_with_has_keyed_children() {
        assert_eq!(
            error(r#"<div $HasKeyedChildren>{[<a key="1"/>, <b key="2"/>]}</div>"#),
            None
        );
    }

    #[test]
    fn should_throw_for_a_child_that_renders_nothing_in_non_keyed_children() {
        assert_error(
            "<div $HasNonKeyedChildren><a/>{null}</div>",
            "$HasNonKeyedChildren needs an array of elements or components, but the child at index 1 is null, which renders nothing.",
        );
    }

    #[test]
    fn should_throw_for_a_nested_array_in_non_keyed_children() {
        assert_error(
            "<div $HasNonKeyedChildren><a/>{[b]}</div>",
            "$HasNonKeyedChildren needs an array of elements or components, but the child at index 1 is an array, which makes a nested array.",
        );
    }

    #[test]
    fn should_not_throw_for_text_next_to_elements_in_non_keyed_children() {
        assert_eq!(error("<div $HasNonKeyedChildren><a/>text</div>"), None);
    }

    #[test]
    fn should_only_check_the_child_flag_that_takes_precedence() {
        assert_eq!(
            error("<div $HasNonKeyedChildren $HasVNodeChildren><a/><b/></div>"),
            None
        );
        assert_error(
            "<div $HasVNodeChildren $HasNonKeyedChildren><a/></div>",
            "$HasNonKeyedChildren needs an array of elements or components, but the only child is an element, not an array.",
        );
    }

    #[test]
    fn should_not_check_child_flags_on_components() {
        assert_eq!(error("<Foo $HasVNodeChildren>text</Foo>"), None);
        assert_eq!(error("<Foo $ChildFlag={3}>text</Foo>"), None);
    }

    #[test]
    fn should_not_check_child_flag_expressions_and_unknown_children() {
        assert_eq!(error("<div $ChildFlag={flag}>text</div>"), None);
        assert_eq!(error("<div $ChildFlag={0}>text</div>"), None);
    }

    #[test]
    fn should_check_a_numeric_child_flag_like_the_flag_of_its_value() {
        assert_error(
            "<div $ChildFlag={2}>text</div>",
            "$ChildFlag={2} (HasVNodeChildren) needs one element or component child, but the child is text.",
        );
        assert_error(
            "<div $ChildFlag={16}><a/></div>",
            "$ChildFlag={16} (HasTextChildren) needs one text child, but the child is an element.",
        );
    }

    // From ts-plugin-inferno. Babel's parser drops the parentheses.
    #[test]
    fn should_look_through_parentheses() {
        assert_error(
            "<div $HasTextChildren>{(<a/>)}</div>",
            "$HasTextChildren needs one text child, but the child is an element.",
        );
        assert_error(
            "<div $HasKeyedChildren>{[(<a/>)]}</div>",
            "$HasKeyedChildren needs an array of elements or components that all have a key, but the array item at index 0 has no key.",
        );
        assert_error(
            "<div $ChildFlag={(2)}>text</div>",
            "$ChildFlag={2} (HasVNodeChildren) needs one element or component child, but the child is text.",
        );
    }

    // From ts-plugin-inferno
    #[test]
    fn should_throw_for_a_children_attribute_without_a_value() {
        assert_error(
            "<div $HasVNodeChildren children />",
            "$HasVNodeChildren needs one element or component child, but the child is true, which renders nothing.",
        );
    }

    #[test]
    fn should_not_throw_for_dynamic_children() {
        assert_eq!(
            error("<div $HasVNodeChildren>{cond ? <a/> : null}</div>"),
            None
        );
        assert_eq!(error("<div $HasTextChildren>{name}</div>"), None);
        assert_eq!(
            error("<div $HasKeyedChildren>{items.map(render)}</div>"),
            None
        );
    }
}

/// packed flags of the v10 factories
mod packed_flags_of_the_v10_factories {
    use super::*;

    #[test]
    fn should_put_the_child_bit_of_each_children_shape_into_the_flags() {
        assert_transform("<div/>", r#"newVNode(17, "div");"#);
        assert_transform("<div>text</div>", r#"newVNode(3, "div", null, "text");"#);
        assert_transform(
            "<div><a/><b/></div>",
            r#"newVNode(5, "div", null, [newVNode(17, "a"), newVNode(17, "b")]);"#,
        );
        assert_transform(
            "<div><a/></div>",
            r#"newVNode(9, "div", null, newVNode(17, "a"));"#,
        );
        assert_transform(
            "<div $HasKeyedChildren>{a}</div>",
            r#"newVNode(33, "div", null, a);"#,
        );
    }

    #[test]
    fn should_leave_the_child_bit_out_for_children_that_are_normalized() {
        assert_transform("<div>{a}</div>", r#"newVNode(1, "div", null, a);"#);
    }

    #[test]
    fn should_keep_the_element_flags_of_svg_and_form_elements() {
        assert_transform(
            "<svg><path/></svg>",
            r#"newVNode(72, "svg", null, newVNode(80, "path"));"#,
        );
        assert_transform("<input/>", r#"newVNode(528, "input");"#);
    }

    #[test]
    fn should_give_components_the_component_unknown_flags_which_are_0() {
        assert_transform("<Foo/>", "newComponentVNode(0, Foo);");
    }

    #[test]
    fn should_add_the_child_bit_to_a_flags_expression() {
        assert_transform(
            "<div $Flags={f}>text</div>",
            r#"newVNode(f | 2, "div", null, "text");"#,
        );
        assert_transform(
            "<div $Flags={a ? b : c}/>",
            r#"newVNode((a ? b : c) | 16, "div");"#,
        );
    }

    #[test]
    fn should_not_add_a_child_bit_to_a_flags_expression_of_normalized_children() {
        assert_transform(
            "<div $Flags={f}>{a}</div>",
            r#"newVNode(f, "div", null, a);"#,
        );
    }

    #[test]
    fn should_fold_a_numeric_child_flag_into_the_flags() {
        assert_transform(
            "<div $ChildFlag={16}>{a}</div>",
            r#"newVNode(3, "div", null, a);"#,
        );
    }

    #[test]
    fn should_throw_for_a_child_flag_number_that_is_not_a_child_flags_value() {
        assert_transform_error(
            "<div $ChildFlag={3}>{a}</div>",
            "$ChildFlag={3} is not a ChildFlags value. Use 0 (UnknownChildren), 1 (HasInvalidChildren), 2 (HasVNodeChildren), 4 (HasNonKeyedChildren), 8 (HasKeyedChildren) or 16 (HasTextChildren).",
        );
    }

    #[test]
    fn should_import_create_vnode_next_to_new_vnode_for_a_child_flag_expression() {
        assert_js_eq(
            &transform_with("{}", "<div><span $ChildFlag={flag}>{a}</span></div>"),
            r#"import { newVNode, createVNode } from "inferno";
newVNode(9, "div", null, createVNode(1, "span", null, a, flag));"#,
        );
    }

    #[test]
    fn should_call_create_fragment_for_a_child_flag_expression_on_a_fragment() {
        assert_js_eq(
            &transform_with("{}", "<Fragment $ChildFlag={flag}>{a}</Fragment>"),
            r#"import { createFragment } from "inferno";
createFragment(a, flag);"#,
        );
    }
}

/// $ReCreate
mod re_create {
    use super::*;

    const MESSAGE: &str = "$ReCreate has been removed in Inferno 10. To re-create the element, change its key instead, for example key={version}.";

    // babel-plugin-inferno prints its code frame. swc prints its own, which points at the same
    // code.
    #[test]
    fn should_throw_for_re_create_on_elements_and_point_at_it() {
        let error = compile(Setup::default(), "<div $ReCreate/>").unwrap_err();

        assert!(error.contains(MESSAGE), "{error}");
        assert!(
            error.contains(" 1 | <div $ReCreate/>\n   :      ^^^^^^^^^\n"),
            "{error}"
        );
    }

    #[test]
    fn should_throw_for_re_create_on_components() {
        assert_transform_error("<Foo $ReCreate/>", MESSAGE);
    }

    #[test]
    fn should_throw_for_re_create_on_input_and_svg_elements() {
        assert_transform_error("<input $ReCreate/>", MESSAGE);
        assert_transform_error("<svg $ReCreate/>", MESSAGE);
    }

    #[test]
    fn should_throw_for_re_create_on_fragments() {
        assert_transform_error("<Fragment $ReCreate>{x}</Fragment>", MESSAGE);
    }

    #[test]
    fn should_throw_for_re_create_on_generic_components() {
        assert_compile_error(
            Setup {
                lang: Lang::Tsx,
                ..Default::default()
            },
            "<Foo<T> $ReCreate/>",
            MESSAGE,
        );
    }

    #[test]
    fn should_throw_for_re_create_whatever_the_useless_flags_option_is() {
        assert_compile_error(
            Setup {
                options: r#"{"uselessFlags": "off"}"#,
                ..Default::default()
            },
            "<div $ReCreate/>",
            MESSAGE,
        );
    }
}

/// other combinations
mod other_combinations {
    use super::*;

    #[test]
    fn should_throw_for_has_text_children_without_children() {
        assert_transform_error(
            "<div $HasTextChildren />",
            "$HasTextChildren needs one text child, but there are no children.",
        );
    }

    #[test]
    fn should_ignore_child_flags_on_components() {
        assert_transform(
            "<Foo $HasKeyedChildren>{a}</Foo>",
            r"newComponentVNode(0, Foo, {
  children: a
});",
        );
    }

    #[test]
    fn should_keep_flags_with_a_spread() {
        assert_transform(
            "<div $Flags={1} {...p}/>",
            r#"normalizeProps(newVNode(17, "div", null, null, {
  ...p
}));"#,
        );
    }

    #[test]
    fn should_keep_has_vnode_children_with_a_spread() {
        assert_transform(
            "<div {...p} $HasVNodeChildren>{a}</div>",
            r#"normalizeProps(newVNode(9, "div", null, a, {
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
            r#"newComponentVNode(131072, Foo, {
  "contentEditable": true
});"#,
        );
    }

    #[test]
    fn should_throw_for_has_text_children_on_several_children() {
        assert_transform_error(
            "<div $HasTextChildren><a/><b/></div>",
            "$HasTextChildren needs one text child, but there are 2 children.",
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
