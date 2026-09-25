//! Ported from babel-plugin-inferno `tests/entities-strings.test.js`: Entities and strings

use crate::helpers::*;

/// entities in text
mod entities_in_text {
    use super::*;

    #[test]
    fn should_decode_nbsp() {
        assert_transform(
            "<div>&nbsp;</div>",
            r#"createVNode(1, "div", null, "\xA0", 16);"#,
        );
    }

    #[test]
    fn should_decode_amp_lt_gt_quot() {
        assert_transform(
            "<div>&amp;&lt;&gt;&quot;</div>",
            r#"createVNode(1, "div", null, "&<>\"", 16);"#,
        );
    }

    #[test]
    fn should_decode_decimal_numeric_entities() {
        assert_transform(
            "<div>&#123;</div>",
            r#"createVNode(1, "div", null, "{", 16);"#,
        );
    }

    #[test]
    fn should_decode_hexadecimal_numeric_entities() {
        assert_transform(
            "<div>&#x20;a</div>",
            r#"createVNode(1, "div", null, " a", 16);"#,
        );
    }

    #[test]
    fn should_decode_0000_to_a_nul_character() {
        assert_transform(
            "<div>&#0000;</div>",
            r#"createVNode(1, "div", null, "\0", 16);"#,
        );
    }

    #[test]
    fn should_keep_unknown_entities_verbatim() {
        assert_transform(
            "<div>&nosuch;</div>",
            r#"createVNode(1, "div", null, "&nosuch;", 16);"#,
        );
    }

    #[test]
    fn should_keep_entity_like_text_verbatim() {
        assert_transform(
            "<div>&ampr;</div>",
            r#"createVNode(1, "div", null, "&ampr;", 16);"#,
        );
    }

    #[test]
    fn should_not_resolve_object_prototype_names_as_entities() {
        assert_transform(
            "<div>&valueOf;</div>",
            r#"createVNode(1, "div", null, "&valueOf;", 16);"#,
        );
    }

    #[test]
    fn should_keep_entities_without_a_terminating_semicolon_verbatim() {
        assert_transform(
            "<div>&Egrave &#123 &#x123</div>",
            r#"createVNode(1, "div", null, "&Egrave &#123 &#x123", 16);"#,
        );
    }
}

/// unicode text
mod unicode_text {
    use super::*;

    #[test]
    fn should_keep_emoji_and_accented_characters() {
        assert_transform(
            "<div>😀 ünïcödé</div>",
            "createVNode(1, \"div\", null, \"😀 ünïcödé\", 16);",
        );
    }

    #[test]
    fn should_keep_cjk_text() {
        assert_transform(
            "<div>日本語</div>",
            "createVNode(1, \"div\", null, \"日本語\", 16);",
        );
    }
}

/// backslashes in text
mod backslashes_in_text {
    use super::*;

    #[test]
    fn should_keep_backslashes_in_text_literal() {
        assert_transform(
            r"<div>C:\temp\new</div>",
            r#"createVNode(1, "div", null, "C:\\temp\\new", 16);"#,
        );
    }

    #[test]
    fn should_not_parse_u_escapes_in_text() {
        assert_transform(
            r"<div>this should not parse as unicode: \u00a0</div>",
            r#"createVNode(1, "div", null, "this should not parse as unicode: \\u00a0", 16);"#,
        );
    }
}

/// string attribute values
mod string_attribute_values {
    use super::*;

    #[test]
    fn should_keep_a_plain_attribute_string() {
        assert_transform(
            r#"<div title="plain" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "title": "plain"
});"#,
        );
    }

    #[test]
    fn should_keep_an_empty_attribute_string() {
        assert_transform(
            r#"<div title="" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "title": ""
});"#,
        );
    }

    #[test]
    fn should_keep_non_ascii_characters_in_attribute_strings() {
        assert_transform(
            "<div title=\"ünïcödé 😀\" />",
            "createVNode(1, \"div\", null, null, 1, {\n  \"title\": \"ünïcödé 😀\"\n});",
        );
    }

    #[test]
    fn should_print_a_single_quoted_attribute_string_with_double_quotes() {
        assert_transform(
            r#"<div title='it"s' />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "title": "it\"s"
});"#,
        );
    }
}

/// attribute strings with line breaks
mod attribute_strings_with_line_breaks {
    use super::*;

    #[test]
    fn should_compile_a_multi_line_class_name() {
        assert_transform(
            r#"<div className="flex
    items-center
    gap-2">x</div>"#,
            r#"createVNode(1, "div", "flex items-center gap-2", "x", 16);"#,
        );
    }

    #[test]
    fn should_compile_a_multi_line_svg_path_babel_parser_regression_7() {
        assert_transform(
            "<path d=\"M230 80\n\t\tA 45 45, 0, 1, 0, 275 125\n    L 275 80 Z\"/>",
            r#"createVNode(32, "path", null, null, 1, {
  "d": "M230 80 A 45 45, 0, 1, 0, 275 125 L 275 80 Z"
});"#,
        );
    }

    #[test]
    fn should_compile_a_multi_line_prop_on_an_element() {
        assert_transform(
            r#"<div title="a
   b" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "title": "a b"
});"#,
        );
    }

    #[test]
    fn should_compile_a_multi_line_prop_on_a_component_transform_react_inline_elements_regressions_6276()
     {
        assert_transform(
            r#"<T default="
    some string
  " />"#,
            r#"createComponentVNode(2, T, {
  "default": " some string "
});"#,
        );
    }

    #[test]
    fn should_compile_a_line_break_that_is_not_followed_by_whitespace() {
        assert_transform(
            r#"<div title="a
b" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "title": "a\nb"
});"#,
        );
    }

    #[test]
    fn should_compile_a_multi_line_key() {
        assert_valid_js(&transform(
            r#"<div key="line1
  line2" />"#,
        ));
    }

    #[test]
    fn should_compile_an_attribute_with_a_crlf_line_break() {
        assert_valid_js(&transform("<div a=\"x\r\n   y\" />"));
    }
}

/// attribute strings with backslashes
mod attribute_strings_with_backslashes {
    use super::*;

    #[test]
    fn should_compile_a_value_ending_in_a_backslash() {
        assert_transform(
            r#"<div title="\" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "title": "\\"
});"#,
        );
    }

    #[test]
    fn should_keep_regular_expression_escapes_in_pattern() {
        assert_transform(
            r#"<input pattern="\d{3}" />"#,
            r#"createVNode(64, "input", null, null, 1, {
  "pattern": "\\d{3}"
});"#,
        );
    }

    #[test]
    fn should_keep_a_complex_pattern_babel_parser_regression_issue_2114() {
        assert_transform(
            r#"<input pattern="^([\w\.\-]+\s)*[\w\.\-]+\s?$" />"#,
            r#"createVNode(64, "input", null, null, 1, {
  "pattern": "^([\\w\\.\\-]+\\s)*[\\w\\.\\-]+\\s?$"
});"#,
        );
    }

    #[test]
    fn should_keep_a_backslash_in_a_key_babel_should_escape_xhtml_jsxattribute() {
        assert_transform(
            r#"<div key="\w" />"#,
            r#"createVNode(1, "div", null, null, 1, null, "\\w");"#,
        );
    }

    #[test]
    fn should_keep_backslashes_before_quotes_react_compiler_quoted_strings_in_jsx_attribute_escaped()
     {
        assert_transform(
            r#"<Stringify text='Some \"text\"' />"#,
            r#"createComponentVNode(2, Stringify, {
  "text": "Some \\\"text\\\""
});"#,
        );
    }

    #[test]
    fn should_produce_valid_module_code_for_9_oxc_jsx_attribute_legacy_escapes() {
        assert_transform(
            r#"<Component mask="+4\9 99 999 99" />"#,
            r#"createComponentVNode(2, Component, {
  "mask": "+4\\9 99 999 99"
});"#,
        );
    }

    #[test]
    fn should_keep_0_as_a_backslash_and_a_zero() {
        assert_transform(
            r#"<div re="\0" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "re": "\\0"
});"#,
        );
    }

    #[test]
    fn should_keep_n_as_a_backslash_and_an_n() {
        assert_transform(
            r#"<div title="\n" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "title": "\\n"
});"#,
        );
    }
}

/// attribute strings with entities
mod attribute_strings_with_entities {
    use super::*;

    #[test]
    fn should_decode_entities_in_element_props() {
        assert_transform(
            r#"<div title="a&amp;b" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "title": "a&b"
});"#,
        );
    }

    #[test]
    fn should_decode_entities_in_component_props() {
        assert_transform(
            r#"<Foo title="a&amp;b" />"#,
            r#"createComponentVNode(2, Foo, {
  "title": "a&b"
});"#,
        );
    }

    #[test]
    fn should_decode_entities_in_class_name() {
        assert_transform(
            r#"<div className="a &amp; b" />"#,
            r#"createVNode(1, "div", "a & b");"#,
        );
    }

    #[test]
    fn should_decode_quote_entities_in_class() {
        assert_transform(
            r#"<div class="&quot;q&quot;" />"#,
            r#"createVNode(1, "div", "\"q\"");"#,
        );
    }

    #[test]
    fn should_decode_entities_in_key() {
        assert_transform(
            r#"<div key="a&amp;b" />"#,
            r#"createVNode(1, "div", null, null, 1, null, "a&b");"#,
        );
    }

    #[test]
    fn should_decode_named_entities_oxc_attribute_escapes() {
        assert_transform(
            r#"<Foo bar="&Egrave; &euro; &quot;" />"#,
            "createComponentVNode(2, Foo, {\n  \"bar\": \"È € \\\"\"\n});",
        );
    }

    #[test]
    fn should_decode_numeric_entities_oxc_attribute_escapes() {
        assert_transform(
            r#"<Foo bar="&#xC; &#x41;" />"#,
            r#"createComponentVNode(2, Foo, {
  "bar": "\f A"
});"#,
        );
    }

    #[test]
    fn should_decode_amp_and_keep_unknown_entities_babel_parser_basic_4() {
        assert_transform(
            r#"<a d="&amp;" e="&ampr;" />"#,
            r#"createVNode(1, "a", null, null, 1, {
  "d": "&",
  "e": "&ampr;"
});"#,
        );
    }
}

/// children prop strings
mod children_prop_strings {
    use super::*;

    #[test]
    fn should_decode_entities_in_an_element_children_prop_string() {
        assert_transform(
            r#"<div children="a&amp;b" />"#,
            r#"createVNode(1, "div", null, "a&b", 16);"#,
        );
    }

    #[test]
    fn should_keep_a_whitespace_only_element_children_prop_string() {
        assert_transform(
            r#"<div children="   " />"#,
            r#"createVNode(1, "div", null, "   ", 16);"#,
        );
    }

    #[test]
    fn should_create_no_children_for_an_empty_element_children_prop_string() {
        assert_transform(r#"<div children="" />"#, r#"createVNode(1, "div");"#);
    }
}
