//! Ported from babel-plugin-inferno `tests/tag-names.test.js`: Tag names

use crate::helpers::*;

/// member expressions
mod member_expressions {
    use super::*;

    #[test]
    fn should_compile_this_foo_as_a_component() {
        assert_transform("<this.foo />", "createComponentVNode(2, this.foo);");
    }

    #[test]
    fn should_compile_a_nested_this_member_expression_with_children() {
        assert_transform(
            "<this.foo.bar>x</this.foo.bar>",
            r#"createComponentVNode(2, this.foo.bar, {
  children: "x"
});"#,
        );
    }

    #[test]
    fn should_compile_a_deep_member_expression() {
        assert_transform("<a.b.c.d />", "createComponentVNode(2, a.b.c.d);");
    }

    #[test]
    fn should_compile_a_lowercase_member_expression_as_a_component() {
        assert_transform("<foo.bar />", "createComponentVNode(2, foo.bar);");
    }

    #[test]
    fn should_compile_a_lowercase_context_provider_as_a_component() {
        assert_transform(
            "<ctx.Provider value={v}>{a}</ctx.Provider>",
            r#"createComponentVNode(2, ctx.Provider, {
  "value": v,
  children: a
});"#,
        );
    }

    #[test]
    fn should_compile_a_member_expression_ending_in_this() {
        assert_transform("<a.this />", "createComponentVNode(2, a.this);");
    }
}

/// identifier tags
mod identifier_tags {
    use super::*;

    #[test]
    #[ignore = "swc-plugin-inferno compiles tags that do not start with an uppercase letter as elements"]
    fn should_compile_an_underscore_prefixed_tag_as_a_component() {
        assert_transform("<_foo />", "createComponentVNode(2, _foo);");
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles tags that do not start with an uppercase letter as elements"]
    fn should_compile_a_dollar_prefixed_tag_as_a_component() {
        assert_transform("<$foo />", "createComponentVNode(2, $foo);");
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles tags that do not start with an uppercase letter as elements"]
    fn should_compile_an_underscore_prefixed_uppercase_tag_as_a_component() {
        assert_transform("<_Foo />", "createComponentVNode(2, _Foo);");
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles tags that do not start with an uppercase letter as elements"]
    fn should_compile_proto_as_a_component() {
        assert_transform("<__proto__ />", "createComponentVNode(2, __proto__);");
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles tags that do not start with an uppercase letter as elements"]
    fn should_compile_an_uppercase_non_ascii_tag_as_a_component() {
        assert_transform("<Ünicode />", "createComponentVNode(2, Ünicode);");
    }

    #[test]
    fn should_compile_an_all_caps_tag_as_a_component() {
        assert_transform("<SVG />", "createComponentVNode(2, SVG);");
    }

    #[test]
    fn should_compile_a_capitalised_tag_as_a_component() {
        assert_transform("<Svg />", "createComponentVNode(2, Svg);");
    }

    #[test]
    fn should_not_treat_a_lowercase_fragment_tag_as_a_fragment() {
        assert_transform(
            "<fragment>a</fragment>",
            r#"createVNode(1, "fragment", null, "a", 16);"#,
        );
    }

    #[test]
    fn should_not_treat_a_fragment_prefixed_component_as_a_fragment() {
        assert_transform(
            "<FragmentX>a</FragmentX>",
            r#"createComponentVNode(2, FragmentX, {
  children: "a"
});"#,
        );
    }

    #[test]
    fn should_compile_a_variable_holding_a_tag_name_as_a_component() {
        assert_transform(
            r#"const TagName = "div";
<TagName />;"#,
            r#"const TagName = "div";
createComponentVNode(2, TagName);"#,
        );
    }
}

/// Object.prototype names as tags
mod object_prototype_names_as_tags {
    use super::*;

    #[test]
    fn should_compile_has_own_property_as_an_element_babel_should_handle_has_own_property_correctly()
     {
        assert_transform(
            "<hasOwnProperty>testing</hasOwnProperty>",
            r#"createVNode(1, "hasOwnProperty", null, "testing", 16);"#,
        );
    }

    #[test]
    fn should_compile_constructor_as_an_element() {
        assert_transform("<constructor />", r#"createVNode(1, "constructor");"#);
    }

    #[test]
    fn should_compile_to_string_as_an_element() {
        assert_transform("<toString />", r#"createVNode(1, "toString");"#);
    }

    #[test]
    fn should_compile_value_of_as_an_element() {
        assert_transform("<valueOf />", r#"createVNode(1, "valueOf");"#);
    }
}

/// namespaced tags
mod namespaced_tags {
    use super::*;

    // babel-plugin-inferno reports: Namespace tags like <svg:rect> are not supported.
    #[test]
    fn should_reject_namespaced_svg_tags() {
        assert_transform_error("<svg:rect />", "JSX Namespace is disabled");
    }

    // babel-plugin-inferno reports: Namespace tags like <f:image> are not supported.
    #[test]
    fn should_reject_namespaced_tags_with_namespaced_attributes() {
        assert_transform_error("<f:image n:attr />", "JSX Namespace is disabled");
    }

    // babel-plugin-inferno reports: Namespace tags like <Namespace:Component> are not supported.
    #[test]
    fn should_reject_namespaced_component_tags() {
        assert_transform_error("<Namespace:Component />", "JSX Namespace is disabled");
    }

    // babel-plugin-inferno reports: Namespace tags like <svg:rect> are not supported.
    #[test]
    fn should_point_the_namespace_tag_error_at_the_tag_name() {
        assert_transform_error(
            r"<div>
  <svg:rect />
</div>",
            r" 2 |   <svg:rect />
   :    ^^^^^^^^",
        );
    }
}

/// tags that are not valid identifiers
mod tags_that_are_not_valid_identifiers {
    use super::*;

    #[test]
    #[ignore = "swc-plugin-inferno compiles hyphens in tag names to subtractions"]
    fn should_compile_an_uppercase_hyphenated_tag_as_an_element() {
        assert_transform("<Foo-bar />", r#"createVNode(1, "Foo-bar");"#);
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles hyphens in tag names to subtractions"]
    fn should_compile_a_mixed_case_hyphenated_tag_with_children_babel_parser_basic_7() {
        assert_transform(
            r#"<AbC-def test="x">bar</AbC-def>"#,
            r#"createVNode(1, "AbC-def", null, "bar", 16, {
  "test": "x"
});"#,
        );
    }

    #[test]
    fn should_compile_an_underscore_prefixed_hyphenated_tag_as_an_element() {
        assert_transform("<_foo-bar />", r#"createVNode(1, "_foo-bar");"#);
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles hyphens in tag names to subtractions"]
    fn should_compile_a_hyphenated_member_expression_property_as_a_computed_access() {
        assert_transform(
            "<Foo.bar-baz />",
            r#"createComponentVNode(2, Foo["bar-baz"]);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles hyphens in tag names to subtractions"]
    fn should_compile_a_hyphenated_property_in_a_deeper_member_expression() {
        assert_transform(
            "<Foo.bar-baz.Qux>x</Foo.bar-baz.Qux>",
            r#"createComponentVNode(2, Foo["bar-baz"].Qux, {
  children: "x"
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles hyphens in tag names to subtractions"]
    fn should_compile_a_hyphenated_property_of_this() {
        assert_transform(
            "<this.foo-bar />",
            r#"createComponentVNode(2, this["foo-bar"]);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles hyphens in tag names to subtractions"]
    fn should_reject_a_hyphenated_member_expression_object() {
        assert_transform_error(
            "<a-b.c />",
            r"a-b is not a valid variable name for a member expression tag.
> 1 | <a-b.c />
    |  ^^^",
        );
    }
}

/// custom elements
mod custom_elements {
    use super::*;

    #[test]
    fn should_compile_a_hyphenated_tag_as_an_element() {
        assert_transform(
            r#"<my-element foo="bar" />"#,
            r#"createVNode(1, "my-element", null, null, 1, {
  "foo": "bar"
});"#,
        );
    }

    #[test]
    fn should_apply_class_name_and_html_for_handling_to_custom_elements() {
        assert_transform(
            r#"<my-element className="x" htmlFor="y" />"#,
            r#"createVNode(1, "my-element", "x", null, 1, {
  "for": "y"
});"#,
        );
    }

    #[test]
    fn should_compile_x_component_as_an_element() {
        assert_transform("<x-component />", r#"createVNode(1, "x-component");"#);
    }

    #[test]
    fn should_compile_a_custom_element_with_a_boolean_attribute() {
        assert_transform(
            "<o-checkbox checked />",
            r#"createVNode(1, "o-checkbox", null, null, 1, {
  "checked": true
});"#,
        );
    }
}

/// element flags
mod element_flags {
    use super::*;

    #[test]
    fn should_flag_select_elements() {
        assert_transform(
            "<select value={v}><option>1</option></select>",
            r#"createVNode(256, "select", null, createVNode(1, "option", null, "1", 16), 2, {
  "value": v
});"#,
        );
    }

    #[test]
    fn should_flag_textarea_elements_without_children() {
        assert_transform(
            "<textarea value={v} />",
            r#"createVNode(128, "textarea", null, null, 1, {
  "value": v
});"#,
        );
    }

    #[test]
    fn should_flag_input_elements() {
        assert_transform(
            "<input disabled />",
            r#"createVNode(64, "input", null, null, 1, {
  "disabled": true
});"#,
        );
    }

    #[test]
    fn should_flag_svg_children_like_foreign_object_and_keep_html_inside() {
        assert_transform(
            "<svg><foreignObject><div/></foreignObject></svg>",
            r#"createVNode(32, "svg", null, createVNode(32, "foreignObject", null, createVNode(1, "div"), 2), 2);"#,
        );
    }

    #[test]
    fn should_flag_svg_text_elements() {
        assert_transform(
            r#"<text x="1">hi</text>"#,
            r#"createVNode(32, "text", null, "hi", 16, {
  "x": "1"
});"#,
        );
    }

    #[test]
    fn should_flag_camel_case_svg_tags() {
        assert_transform(
            r#"<svg><linearGradient /><feGaussianBlur stdDeviation="2" /><textPath /><animateMotion /></svg>"#,
            r#"createVNode(32, "svg", null, [createVNode(32, "linearGradient"), createVNode(32, "feGaussianBlur", null, null, 1, {
  "stdDeviation": "2"
}), createVNode(32, "textPath"), createVNode(32, "animateMotion")], 4);"#,
        );
    }

    #[test]
    fn should_compile_option_elements_as_plain_html_elements() {
        assert_transform(
            "<option>x</option>",
            r#"createVNode(1, "option", null, "x", 16);"#,
        );
    }
}

/// this in arrow functions
mod this_in_arrow_functions {
    use super::*;

    #[test]
    fn should_rewrite_this_foo_to_this_foo_when_arrow_functions_are_compiled() {
        assert_js_eq(
            &transform_with("{}", "class A { m() { return () => <this.Foo/>; } }"),
            r#"import { createComponentVNode } from "inferno";
class A {
  m() {
    return () => createComponentVNode(2, this.Foo);
  }
}"#,
        );
    }

    #[test]
    fn should_rewrite_this_in_opening_and_nested_member_tags() {
        assert_js_eq(
            &transform_with(
                "{}",
                "class A { m() { return () => <this.foo.bar.qux><this.foo></this.foo></this.foo.bar.qux>; } }",
            ),
            r#"import { createComponentVNode } from "inferno";
class A {
  m() {
    return () => createComponentVNode(2, this.foo.bar.qux, {
      children: createComponentVNode(2, this.foo)
    });
  }
}"#,
        );
    }

    #[test]
    fn should_rewrite_this_inside_element_children() {
        assert_js_eq(
            &transform_with(
                "{}",
                "class A { m() { return () => <div>{this.x}</div>; } }",
            ),
            r#"import { createVNode } from "inferno";
class A {
  m() {
    return () => createVNode(1, "div", null, this.x, 0);
  }
}"#,
        );
    }
}

/// tag evaluation order
mod tag_evaluation_order {
    use super::*;

    #[test]
    fn should_read_the_outer_component_tag_before_evaluating_its_children() {
        assert_transform(
            "<Tag>{((Tag = Other), v)}<Tag /></Tag>",
            r"createComponentVNode(2, Tag, {
  children: [(Tag = Other, v), createComponentVNode(2, Tag)]
});",
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    // babel-plugin-inferno compiles <this /> to an element named "this".
    #[test]
    fn should_compile_this_as_a_component() {
        assert_transform("() => <this />", "()=>createComponentVNode(2, this);");
    }

    #[test]
    fn should_compile_a_lowercase_non_ascii_tag_as_an_element() {
        assert_transform("<é />", "createVNode(1, \"é\");");
    }

    #[test]
    fn should_compile_a_lowercase_non_ascii_word_tag_as_an_element() {
        assert_transform("<ünicode />", "createVNode(1, \"ünicode\");");
    }

    #[test]
    #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
    fn should_treat_any_member_expression_ending_in_fragment_as_a_fragment() {
        assert_transform("<x.Fragment>{a}</x.Fragment>", "createFragment(a, 0);");
    }

    #[test]
    #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
    fn should_treat_a_deep_member_expression_ending_in_fragment_as_a_fragment() {
        assert_transform(
            "<Foo.Bar.Fragment>x</Foo.Bar.Fragment>",
            r#"createFragment([createTextVNode("x")], 4);"#,
        );
    }

    #[test]
    fn should_flag_svg_image_as_a_plain_html_element() {
        assert_transform(
            r#"<image xlinkHref="a.png" />"#,
            r#"createVNode(1, "image", null, null, 1, {
  "xlink:href": "a.png"
});"#,
        );
    }

    #[test]
    fn should_flag_math_ml_as_plain_html_elements() {
        assert_transform(
            "<math><mi>x</mi></math>",
            r#"createVNode(1, "math", null, createVNode(1, "mi", null, "x", 16), 2);"#,
        );
    }
}
