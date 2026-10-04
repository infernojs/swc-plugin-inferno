//! Ported from babel-plugin-inferno `tests/tag-names.test.js`: Tag names

use crate::helpers::*;

/// member expressions
mod member_expressions {
    use super::*;

    #[test]
    fn should_compile_this_foo_as_a_component() {
        assert_transform("<this.foo />", "newComponentVNode(0, this.foo);");
    }

    #[test]
    fn should_compile_a_nested_this_member_expression_with_children() {
        assert_transform(
            "<this.foo.bar>x</this.foo.bar>",
            r#"newComponentVNode(0, this.foo.bar, {
  children: "x"
});"#,
        );
    }

    #[test]
    fn should_compile_a_deep_member_expression() {
        assert_transform("<a.b.c.d />", "newComponentVNode(0, a.b.c.d);");
    }

    #[test]
    fn should_compile_a_lowercase_member_expression_as_a_component() {
        assert_transform("<foo.bar />", "newComponentVNode(0, foo.bar);");
    }

    #[test]
    fn should_compile_a_lowercase_context_provider_as_a_component() {
        assert_transform(
            "<ctx.Provider value={v}>{a}</ctx.Provider>",
            r#"newComponentVNode(0, ctx.Provider, {
  "value": v,
  children: a
});"#,
        );
    }

    #[test]
    fn should_compile_a_member_expression_ending_in_this() {
        assert_transform("<a.this />", "newComponentVNode(0, a.this);");
    }
}

/// identifier tags
mod identifier_tags {
    use super::*;

    #[test]
    fn should_compile_an_underscore_prefixed_tag_as_a_component() {
        assert_transform("<_foo />", "newComponentVNode(0, _foo);");
    }

    #[test]
    fn should_compile_a_dollar_prefixed_tag_as_a_component() {
        assert_transform("<$foo />", "newComponentVNode(0, $foo);");
    }

    #[test]
    fn should_compile_an_underscore_prefixed_uppercase_tag_as_a_component() {
        assert_transform("<_Foo />", "newComponentVNode(0, _Foo);");
    }

    #[test]
    fn should_compile_proto_as_a_component() {
        assert_transform("<__proto__ />", "newComponentVNode(0, __proto__);");
    }

    #[test]
    fn should_compile_an_uppercase_non_ascii_tag_as_a_component() {
        assert_transform("<Ünicode />", "newComponentVNode(0, Ünicode);");
    }

    #[test]
    fn should_compile_an_all_caps_tag_as_a_component() {
        assert_transform("<SVG />", "newComponentVNode(0, SVG);");
    }

    #[test]
    fn should_compile_a_capitalised_tag_as_a_component() {
        assert_transform("<Svg />", "newComponentVNode(0, Svg);");
    }

    #[test]
    fn should_not_treat_a_lowercase_fragment_tag_as_a_fragment() {
        assert_transform(
            "<fragment>a</fragment>",
            r#"newVNode(3, "fragment", null, "a");"#,
        );
    }

    #[test]
    fn should_not_treat_a_fragment_prefixed_component_as_a_fragment() {
        assert_transform(
            "<FragmentX>a</FragmentX>",
            r#"newComponentVNode(0, FragmentX, {
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
newComponentVNode(0, TagName);"#,
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
            r#"newVNode(3, "hasOwnProperty", null, "testing");"#,
        );
    }

    #[test]
    fn should_compile_constructor_as_an_element() {
        assert_transform("<constructor />", r#"newVNode(17, "constructor");"#);
    }

    #[test]
    fn should_compile_to_string_as_an_element() {
        assert_transform("<toString />", r#"newVNode(17, "toString");"#);
    }

    #[test]
    fn should_compile_value_of_as_an_element() {
        assert_transform("<valueOf />", r#"newVNode(17, "valueOf");"#);
    }
}

/// namespaced tags
mod namespaced_tags {
    use super::*;

    #[test]
    fn should_reject_namespaced_svg_tags() {
        assert_transform_error(
            "<svg:rect />",
            "Namespace tags like <svg:rect> are not supported.",
        );
    }

    #[test]
    fn should_reject_namespaced_tags_with_namespaced_attributes() {
        assert_transform_error(
            "<f:image n:attr />",
            "Namespace tags like <f:image> are not supported.",
        );
    }

    #[test]
    fn should_reject_namespaced_component_tags() {
        assert_transform_error(
            "<Namespace:Component />",
            "Namespace tags like <Namespace:Component> are not supported.",
        );
    }

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
    fn should_compile_an_uppercase_hyphenated_tag_as_an_element() {
        assert_transform("<Foo-bar />", r#"newVNode(17, "Foo-bar");"#);
    }

    #[test]
    fn should_compile_a_mixed_case_hyphenated_tag_with_children_babel_parser_basic_7() {
        assert_transform(
            r#"<AbC-def test="x">bar</AbC-def>"#,
            r#"newVNode(3, "AbC-def", null, "bar", {
  "test": "x"
});"#,
        );
    }

    #[test]
    fn should_compile_an_underscore_prefixed_hyphenated_tag_as_an_element() {
        assert_transform("<_foo-bar />", r#"newVNode(17, "_foo-bar");"#);
    }

    #[test]
    fn should_compile_a_hyphenated_member_expression_property_as_a_computed_access() {
        assert_transform(
            "<Foo.bar-baz />",
            r#"newComponentVNode(0, Foo["bar-baz"]);"#,
        );
    }

    #[test]
    fn should_compile_a_hyphenated_property_in_a_deeper_member_expression() {
        assert_transform(
            "<Foo.bar-baz.Qux>x</Foo.bar-baz.Qux>",
            r#"newComponentVNode(0, Foo["bar-baz"].Qux, {
  children: "x"
});"#,
        );
    }

    #[test]
    fn should_compile_a_hyphenated_property_of_this() {
        assert_transform(
            "<this.foo-bar />",
            r#"newComponentVNode(0, this["foo-bar"]);"#,
        );
    }

    #[test]
    fn should_reject_a_hyphenated_member_expression_object() {
        assert_transform_error(
            "<a-b.c />",
            "a-b is not a valid variable name for a member expression tag.",
        );
        assert_transform_error(
            "<a-b.c />",
            r" 1 | <a-b.c />
   :  ^^^",
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
            r#"newVNode(17, "my-element", null, null, {
  "foo": "bar"
});"#,
        );
    }

    #[test]
    fn should_apply_class_name_and_html_for_handling_to_custom_elements() {
        assert_transform(
            r#"<my-element className="x" htmlFor="y" />"#,
            r#"newVNode(17, "my-element", "x", null, {
  "for": "y"
});"#,
        );
    }

    #[test]
    fn should_compile_x_component_as_an_element() {
        assert_transform("<x-component />", r#"newVNode(17, "x-component");"#);
    }

    #[test]
    fn should_compile_a_custom_element_with_a_boolean_attribute() {
        assert_transform(
            "<o-checkbox checked />",
            r#"newVNode(17, "o-checkbox", null, null, {
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
            r#"newVNode(4104, "select", null, newVNode(3, "option", null, "1"), {
  "value": v
});"#,
        );
    }

    #[test]
    fn should_flag_textarea_elements_without_children() {
        assert_transform(
            "<textarea value={v} />",
            r#"newVNode(2064, "textarea", null, null, {
  "value": v
});"#,
        );
    }

    #[test]
    fn should_flag_input_elements() {
        assert_transform(
            "<input disabled />",
            r#"newVNode(528, "input", null, null, {
  "disabled": true
});"#,
        );
    }

    #[test]
    fn should_flag_svg_children_like_foreign_object_and_keep_html_inside() {
        assert_transform(
            "<svg><foreignObject><div/></foreignObject></svg>",
            r#"newVNode(72, "svg", null, newVNode(72, "foreignObject", null, newVNode(17, "div")));"#,
        );
    }

    #[test]
    fn should_flag_svg_text_elements() {
        assert_transform(
            r#"<text x="1">hi</text>"#,
            r#"newVNode(66, "text", null, "hi", {
  "x": "1"
});"#,
        );
    }

    #[test]
    fn should_flag_camel_case_svg_tags() {
        assert_transform(
            r#"<svg><linearGradient /><feGaussianBlur stdDeviation="2" /><textPath /><animateMotion /></svg>"#,
            r#"newVNode(68, "svg", null, [newVNode(80, "linearGradient"), newVNode(80, "feGaussianBlur", null, null, {
  "stdDeviation": "2"
}), newVNode(80, "textPath"), newVNode(80, "animateMotion")]);"#,
        );
    }

    #[test]
    fn should_compile_option_elements_as_plain_html_elements() {
        assert_transform("<option>x</option>", r#"newVNode(3, "option", null, "x");"#);
    }
}

/// this in arrow functions
mod this_in_arrow_functions {
    use super::*;

    #[test]
    fn should_rewrite_this_foo_to_this_foo_when_arrow_functions_are_compiled() {
        assert_js_eq(
            &transform_with("{}", "class A { m() { return () => <this.Foo/>; } }"),
            r#"import { newComponentVNode } from "inferno";
class A {
  m() {
    return () => newComponentVNode(0, this.Foo);
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
            r#"import { newComponentVNode } from "inferno";
class A {
  m() {
    return () => newComponentVNode(0, this.foo.bar.qux, {
      children: newComponentVNode(0, this.foo)
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
            r#"import { newVNode } from "inferno";
class A {
  m() {
    return () => newVNode(1, "div", null, this.x);
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
            r"newComponentVNode(0, Tag, {
  children: [(Tag = Other, v), newComponentVNode(0, Tag)]
});",
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    #[test]
    fn should_compile_this_as_an_element_named_this() {
        assert_transform("() => <this />", r#"() => newVNode(17, "this");"#);
    }

    #[test]
    fn should_compile_a_lowercase_non_ascii_tag_as_an_element() {
        assert_transform("<é />", "newVNode(17, \"é\");");
    }

    #[test]
    fn should_compile_a_lowercase_non_ascii_word_tag_as_an_element() {
        assert_transform("<ünicode />", "newVNode(17, \"ünicode\");");
    }

    #[test]
    fn should_treat_any_member_expression_ending_in_fragment_as_a_fragment() {
        assert_transform("<x.Fragment>{a}</x.Fragment>", "newFragment(256, a);");
    }

    #[test]
    fn should_treat_a_deep_member_expression_ending_in_fragment_as_a_fragment() {
        assert_transform(
            "<Foo.Bar.Fragment>x</Foo.Bar.Fragment>",
            r#"newFragment(260, [newTextVNode("x")]);"#,
        );
    }

    #[test]
    fn should_flag_svg_image_as_a_plain_html_element() {
        assert_transform(
            r#"<image xlinkHref="a.png" />"#,
            r#"newVNode(17, "image", null, null, {
  "xlink:href": "a.png"
});"#,
        );
    }

    #[test]
    fn should_flag_math_ml_as_plain_html_elements() {
        assert_transform(
            "<math><mi>x</mi></math>",
            r#"newVNode(9, "math", null, newVNode(3, "mi", null, "x"));"#,
        );
    }
}
