//! Ported from babel-plugin-inferno `tests/react-parity.test.js`: React parity

use crate::helpers::*;

/// TransformJSXToReactJSX-test
mod transform_jsxto_react_jsx_test {
    use super::*;

    #[test]
    fn should_keep_a_trailing_space_before_an_expression_should_handle_attributed_elements() {
        assert_transform(
            "<div>Hello {this.props.name}</div>",
            r#"createVNode(1, "div", null, [createTextVNode("Hello "), this.props.name], 0);"#,
        );
    }

    #[test]
    fn should_compile_an_element_inside_a_multi_line_attribute_expression() {
        assert_transform(
            r"<HelloMessage name={
  <span>
    Sebastian
  </span>
} />",
            r#"createComponentVNode(2, HelloMessage, {
  "name": createVNode(1, "span", null, "Sebastian", 16)
});"#,
        );
    }

    #[test]
    fn should_not_strip_nbsp_followed_by_a_space() {
        assert_transform(
            "<div>&nbsp; </div>",
            r#"createVNode(1, "div", null, "\xA0 ", 16);"#,
        );
    }

    #[test]
    fn should_keep_nbsp_between_words() {
        assert_transform(
            "<div>w &nbsp; w</div>",
            r#"createVNode(1, "div", null, "w \xA0 w", 16);"#,
        );
    }

    #[test]
    fn should_keep_a_bare_ampersand() {
        assert_transform(
            "<div>w & w</div>",
            r#"createVNode(1, "div", null, "w & w", 16);"#,
        );
    }

    #[test]
    fn should_decode_amp_and_lt_in_text() {
        assert_transform(
            r"<div>w &amp; w</div>;
<div>w &lt; w</div>",
            r#"createVNode(1, "div", null, "w & w", 16);
createVNode(1, "div", null, "w < w", 16);"#,
        );
    }

    #[test]
    fn should_keep_non_ascii_text() {
        assert_transform(
            "<div>wôw</div>",
            "createVNode(1, \"div\", null, \"wôw\", 16);",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
    fn should_compile_a_keyed_react_fragment_without_children() {
        assert_transform(
            r#"<React.Fragment key="foo"></React.Fragment>"#,
            r#"createFragment(null, 1, "foo");"#,
        );
    }

    #[test]
    fn should_compile_a_spread_of_a_null_variable() {
        assert_transform(
            r"var foo = null;
<div {...foo} />",
            r#"var foo = null;
normalizeProps(createVNode(1, "div", null, null, 1, {
  ...foo
}));"#,
        );
    }

    #[test]
    fn should_compile_a_spread_followed_by_a_prop() {
        assert_transform(
            r#"<Component {...props} sound="moo" />"#,
            r#"normalizeProps(createComponentVNode(2, Component, {
  ...props,
  "sound": "moo"
}));"#,
        );
    }

    #[test]
    fn should_drop_a_children_prop_that_is_overridden_by_a_spread_and_jsx_children() {
        assert_transform(
            "<Component children={1} {...x}>2</Component>",
            r#"normalizeProps(createComponentVNode(2, Component, {
  ...x,
  children: "2"
}));"#,
        );
    }

    #[test]
    fn should_compile_a_mixed_static_element_and_array_child() {
        assert_transform(
            r#"<div><span />{[<span key="0" />, <span key="1" />]}</div>"#,
            r#"createVNode(1, "div", null, [createVNode(1, "span"), [createVNode(1, "span", null, null, 1, null, "0"), createVNode(1, "span", null, null, 1, null, "1")]], 0);"#,
        );
    }

    #[test]
    fn should_compile_a_single_array_child() {
        assert_transform(
            r#"<div>{[<span key="0" />, <span key="1" />]}</div>"#,
            r#"createVNode(1, "div", null, [createVNode(1, "span", null, null, 1, null, "0"), createVNode(1, "span", null, null, 1, null, "1")], 0);"#,
        );
    }
}

/// jstransform react-test
mod jstransform_react_test {
    use super::*;

    #[test]
    fn should_keep_parenthesized_attribute_values() {
        assert_transform(
            "<foo a={(b)} c={(d)}>Hello</foo>",
            r#"createVNode(1, "foo", null, "Hello", 16, {
  "a": b,
  "c": d
});"#,
        );
    }

    #[test]
    fn should_allow_constructor_as_a_component_prop() {
        assert_transform(
            r#"<Component constructor="foo" />"#,
            r#"createComponentVNode(2, Component, {
  "constructor": "foo"
});"#,
        );
    }

    #[test]
    fn should_keep_comments_inside_a_parenthesized_child_expression() {
        assert_transform(
            r"<div>
  Foo {(e+f //A line comment
  /* A multiline comment */)
  } bar
</div>",
            r#"createVNode(1, "div", null, [createTextVNode("Foo "), e + f //A line comment
/* A multiline comment */, createTextVNode(" bar")], 0);"#,
        );
    }

    #[test]
    fn should_keep_leading_spaces_of_the_first_text_line() {
        assert_transform(
            r"<div>  sdfsdfsdf
  sdlkfjsdfljs
   </div>",
            r#"createVNode(1, "div", null, "  sdfsdfsdf sdlkfjsdfljs", 16);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not convert tabs in JSX text to spaces"]
    fn should_convert_trailing_tabs_to_spaces() {
        assert_transform(
            "<div>a  \t \t </div>",
            r#"createVNode(1, "div", null, "a      ", 16);"#,
        );
    }

    #[test]
    fn should_keep_a_leading_space() {
        assert_transform("<div> a</div>", r#"createVNode(1, "div", null, " a", 16);"#);
    }

    #[test]
    fn should_keep_a_trailing_space() {
        assert_transform("<div>a </div>", r#"createVNode(1, "div", null, "a ", 16);"#);
    }
}

/// compiler fixtures
mod compiler_fixtures {
    use super::*;

    #[test]
    fn should_keep_jsx_text_and_a_string_literal_child_apart_preserve_jsxtext_stringliteral_distinction()
     {
        assert_transform(
            r#"<div> {", "}</div>"#,
            r#"createVNode(1, "div", null, [createTextVNode(" "), createTextVNode(", ")], 0);"#,
        );
    }

    #[test]
    fn should_compile_nested_member_expression_tags_jsx_member_expression() {
        assert_transform(
            "<Sathya.Codes.Forget><Foo.Bar.Baz /></Sathya.Codes.Forget>",
            r"createComponentVNode(2, Sathya.Codes.Forget, {
  children: createComponentVNode(2, Foo.Bar.Baz)
});",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno wraps text in createTextVNode inside component children"]
    fn should_compile_a_lowercase_local_member_expression_as_a_component_jsx_lowercase_localvar_memberexpr()
     {
        assert_transform(
            "<localVar.Stringify>hello world {name}</localVar.Stringify>",
            r#"createComponentVNode(2, localVar.Stringify, {
  children: ["hello world ", name]
});"#,
        );
    }

    #[test]
    fn should_keep_a_lowercase_tag_a_string_even_with_a_same_named_binding_invalid_jsx_lowercase_localvar()
     {
        assert_transform(
            r"const invalidTag = Throw;
<invalidTag val={{val: 2}} />;",
            r#"const invalidTag = Throw;
createVNode(1, "invalidTag", null, null, 1, {
  "val": {
    val: 2
  }
});"#,
        );
    }

    #[test]
    fn should_compile_a_valueless_attribute_to_true_jsx_attribute_default_to_true() {
        assert_transform(
            "<Stringify truthyAttribute />",
            r#"createComponentVNode(2, Stringify, {
  "truthyAttribute": true
});"#,
        );
    }

    #[test]
    fn should_decode_entities_in_text_jsx_html_entity() {
        assert_transform(
            "<div>&gt;&lt;span &amp;</div>",
            r#"createVNode(1, "div", null, "><span &", 16);"#,
        );
    }

    #[test]
    fn should_decode_numeric_entities_across_a_line_break_jsx_bracket_in_text() {
        assert_transform(
            r"<div>If the string contains the string &#123;pageNumber&#125; it will be
    replaced</div>",
            r#"createVNode(1, "div", null, "If the string contains the string {pageNumber} it will be replaced", 16);"#,
        );
    }

    #[test]
    fn should_keep_double_quotes_inside_a_single_quoted_attribute_quoted_strings_in_jsx_attribute()
    {
        assert_transform(
            r#"<Stringify text='Some "text"' />"#,
            r#"createComponentVNode(2, Stringify, {
  "text": "Some \"text\""
});"#,
        );
    }

    #[test]
    fn should_keep_escapes_of_strings_in_expression_containers_jsx_string_attribute_expression_container()
     {
        assert_transform(
            r"<Foo value={'\n'} other={'A\tE'} />",
            r#"createComponentVNode(2, Foo, {
  "value": '\n',
  "other": 'A\tE'
});"#,
        );
    }

    #[test]
    fn should_keep_lone_surrogates_in_expression_containers_lone_surrogate_string_values() {
        assert_transform(
            r"<Foo codepoints={['\uD83E', '\uDD21']} />",
            r#"createComponentVNode(2, Foo, {
  "codepoints": ['\uD83E', '\uDD21']
});"#,
        );
    }

    #[test]
    fn should_keep_key_and_style_after_a_spread_repro_undefined_expression_of_jsxexpressioncontainer()
     {
        assert_transform(
            "<Stringify {...buttonProps} key={`button-${i}`} style={s} />",
            r#"normalizeProps(createComponentVNode(2, Stringify, {
  ...buttonProps,
  "style": s
}, `button-${i}`));"#,
        );
    }
}

/// react-dom and runtime tests
mod react_dom_and_runtime_tests {
    use super::*;

    #[test]
    fn should_keep_explicit_space_children_react_domserver_integration_elements() {
        assert_transform(
            r#"<div>{" "}{" "}{" "}</div>"#,
            r#"createVNode(1, "div", null, [" ", " ", " "], 0);"#,
        );
    }

    #[test]
    fn should_keep_a_space_between_two_expressions_in_an_option_react_domoption() {
        assert_transform(
            r#"<option>
  {1} {"foo"}
</option>"#,
            r#"createVNode(1, "option", null, [1, createTextVNode(" "), createTextVNode("foo")], 0);"#,
        );
    }

    #[test]
    fn should_split_text_around_an_expression_in_an_option_react_domoption() {
        assert_transform(
            "<option>gir{a}ffe</option>",
            r#"createVNode(1, "option", null, [createTextVNode("gir"), a, createTextVNode("ffe")], 0);"#,
        );
    }

    #[test]
    fn should_keep_text_directly_next_to_an_element_react_domserver_integration_elements() {
        assert_transform(
            r"<div>
  Text<span>More Text</span>
</div>",
            r#"createVNode(1, "div", null, [createTextVNode("Text"), createVNode(1, "span", null, "More Text", 16)], 4);"#,
        );
    }

    #[test]
    fn should_compile_deeply_nested_fragments_with_null_and_false_react_domserver_integration_fragment()
     {
        assert_transform(
            "<><><div>text1</div></><span/><><><>{null}<p /></>{false}</></></>",
            r#"createFragment([createFragment([createVNode(1, "div", null, "text1", 16)], 4), createVNode(1, "span"), createFragment([createFragment([createFragment([null, createVNode(1, "p")], 0), false], 0)], 4)], 4);"#,
        );
    }

    #[test]
    fn should_pass_expression_children_of_a_textarea_react_domtextarea() {
        assert_transform(
            "<textarea>{17}</textarea>",
            r#"createVNode(128, "textarea", null, 17, 0);"#,
        );
    }

    #[test]
    fn should_keep_select_props_react_domselect() {
        assert_transform(
            r#"<select multiple={true} defaultValue={["giraffe"]} />"#,
            r#"createVNode(256, "select", null, null, 1, {
  "multiple": true,
  "defaultValue": ["giraffe"]
});"#,
        );
    }

    #[test]
    fn should_pass_source_and_self_as_ordinary_props_react_element_validator() {
        assert_transform(
            r#"<div __source={{fileName: "a"}} __self={this} />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "__source": {
    fileName: "a"
  },
  "__self": this
});"#,
        );
    }

    #[test]
    fn should_keep_a_whitespace_only_line_between_inline_elements_whitespace_transformer_readme() {
        assert_transform(
            r#"<div>
  Monkeys:
  <input type="text" /> <button />
</div>"#,
            r#"createVNode(1, "div", null, [createTextVNode("Monkeys:"), createVNode(64, "input", null, null, 1, {
  "type": "text"
}), createTextVNode(" "), createVNode(1, "button")], 4);"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    // babel-plugin-inferno marks the span and the children as unknown children.
    #[test]
    fn should_ignore_comments_between_children_transform_jsxto_react_jsx_test() {
        assert_transform(
            r"<div>
  {/* A comment at the beginning */}
  {/* A second comment at the beginning */}
  <span>
    {/* A nested comment */}
  </span>
  {/* A sandwiched comment */}
  <br />
  {/* A comment at the end */}
  {/* A second comment at the end */}
</div>",
            r#"createVNode(1, "div", null, [
    createVNode(1, "span"),
    createVNode(1, "br")
], 4);"#,
        );
    }
}
