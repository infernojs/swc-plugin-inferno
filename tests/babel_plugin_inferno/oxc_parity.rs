//! Ported from babel-plugin-inferno `tests/oxc-parity.test.js`: oxc parity

use crate::helpers::*;

/// text/escapes
mod text_escapes {
    use super::*;

    #[test]
    fn should_decode_named_entities() {
        assert_transform(
            "<div>&nbsp;&iexcl;&cent;&pound;&curren;&yen;&brvbar;&sect;&uml;&copy;</div>",
            "newVNode(3, \"div\", null, \"\\xA0¡¢£¤¥¦§¨©\");",
        );
    }

    #[test]
    fn should_decode_invisible_named_entities() {
        assert_transform(
            "<div>&shy; &ensp; &emsp; &thinsp; &zwnj; &zwj; &lrm; &rlm;</div>",
            "newVNode(3, \"div\", null, \"\u{AD} \\u2002 \\u2003 \\u2009 \u{200C} \u{200D} \u{200E} \u{200F}\");",
        );
    }

    #[test]
    fn should_decode_quote_ampersand_and_angle_entities_and_keep_unknown_ones() {
        assert_transform(
            "<div>&quot; &amp; &lt; &gt; &donkey;</div>",
            r#"newVNode(3, "div", null, "\" & < > &donkey;");"#,
        );
    }

    #[test]
    fn should_decode_accented_and_currency_entities() {
        assert_transform(
            "<div>&Egrave; &euro;</div>",
            "newVNode(3, \"div\", null, \"È €\");",
        );
    }
}

/// text/numeric-escapes
mod text_numeric_escapes {
    use super::*;

    #[test]
    fn should_decode_hexadecimal_entities_up_to_u_10_ffff() {
        assert_transform(
            "<div>&#xC; &#x41; &#x123; &#x1234; &#x10000; &#x10FFFF;</div>",
            "newVNode(3, \"div\", null, \"\\f A ģ ሴ 𐀀 \u{10FFFF}\");",
        );
    }

    #[test]
    fn should_decode_decimal_entities_up_to_u_10_ffff() {
        assert_transform(
            "<div>&#12; &#65; &#291; &#4660; &#65536; &#1114111;</div>",
            "newVNode(3, \"div\", null, \"\\f A ģ ሴ 𐀀 \u{10FFFF}\");",
        );
    }

    #[test]
    fn should_keep_invalid_numeric_entities_verbatim() {
        assert_transform(
            "<div>&#xG; &#C;</div>",
            r#"newVNode(3, "div", null, "&#xG; &#C;");"#,
        );
    }
}

/// text/unterminated-escapes
mod text_unterminated_escapes {
    use super::*;

    #[test]
    fn should_keep_a_named_entity_without_semicolon() {
        assert_transform(
            "<div>&Egrave</div>",
            r#"newVNode(3, "div", null, "&Egrave");"#,
        );
    }

    #[test]
    fn should_keep_a_named_entity_followed_by_text() {
        assert_transform(
            "<div>&euro xxx</div>",
            r#"newVNode(3, "div", null, "&euro xxx");"#,
        );
    }

    #[test]
    fn should_keep_a_decimal_entity_without_semicolon() {
        assert_transform(
            "<div>&#123 xxx</div>",
            r#"newVNode(3, "div", null, "&#123 xxx");"#,
        );
    }

    #[test]
    fn should_keep_a_hexadecimal_entity_without_semicolon() {
        assert_transform(
            "<div>&#x123 xxx</div>",
            r#"newVNode(3, "div", null, "&#x123 xxx");"#,
        );
    }
}

/// text/whitespace
mod text_whitespace {
    use super::*;

    #[test]
    fn should_keep_single_line_whitespace() {
        assert_transform(
            "<div> \t angry \t </div>",
            r#"newVNode(3, "div", null, "   angry   ");"#,
        );
    }

    #[test]
    fn should_keep_whitespace_of_the_first_and_last_lines() {
        assert_transform(
            "<div> \t boris\ncod\ndante \t </div>",
            r#"newVNode(3, "div", null, "   boris cod dante   ");"#,
        );
    }

    #[test]
    fn should_drop_whitespace_only_first_and_last_lines() {
        assert_transform(
            "<div> \t \naging\n \t </div>",
            r#"newVNode(3, "div", null, "aging");"#,
        );
    }

    #[test]
    fn should_keep_whitespace_inside_a_line() {
        assert_transform(
            "<div>\n \t bark \t club \t devil \t \n</div>",
            r#"newVNode(3, "div", null, "bark   club   devil");"#,
        );
    }
}

/// text/newline-entities
mod text_newline_entities {
    use super::*;

    #[test]
    fn should_collapse_an_encoded_newline_between_words() {
        assert_transform("<div>a&#10;b</div>", r#"newVNode(3, "div", null, "a b");"#);
    }

    #[test]
    fn should_collapse_an_encoded_newline_at_a_line_end() {
        assert_transform(
            r"<div>
  a&#10;
  b
</div>",
            r#"newVNode(3, "div", null, "a b");"#,
        );
    }

    #[test]
    fn should_convert_encoded_tabs_to_spaces() {
        assert_transform(
            "<div>&#9;x&#9;</div>",
            r#"newVNode(3, "div", null, " x ");"#,
        );
    }
}

/// text/unicode
mod text_unicode {
    use super::*;

    #[test]
    fn should_keep_an_emoji_with_a_variation_selector_on_its_own_line() {
        assert_transform(
            "<h2>\n🏝\u{FE0F}\n</h2>",
            "newVNode(3, \"h2\", null, \"🏝\u{FE0F}\");",
        );
    }
}

/// issues
mod issues {
    use super::*;

    #[test]
    fn issue_6638_should_drop_tab_indentation_of_nested_components() {
        assert_transform(
            "<Suspense fallback={\"Loading...\"}>\n\t<PanelGroup>\n\t\t<Panel>\n\t\t\t<A/>\n\t\t</Panel>\n\t</PanelGroup>\n</Suspense>",
            r#"newComponentVNode(0, Suspense, {
  "fallback": "Loading...",
  children: newComponentVNode(0, PanelGroup, {
    children: newComponentVNode(0, Panel, {
      children: newComponentVNode(0, A)
    })
  })
});"#,
        );
    }

    #[test]
    fn issue_20669_should_ignore_jsx_import_source_pragmas_in_comments() {
        assert_js_eq(
            &transform_with(
                "{}",
                r"/** @jsxImportSource react */
/**
 * Mentions `@jsxImportSource custom/source` in docs
 */
export const a = <div/>;",
            ),
            r#"import { newVNode } from "inferno";
/** @jsxImportSource react */
/**
 * Mentions `@jsxImportSource custom/source` in docs
 */
export const a = newVNode(17, "div");"#,
        );
    }

    #[test]
    fn issue_10956_should_ignore_jsx_and_jsx_runtime_pragmas_with_only_remove_type_imports() {
        assert_js_eq(
            &compile_ok(
                Setup {
                    lang: Lang::Tsx,
                    typescript: r#"{"verbatimModuleSyntax": true}"#,
                    ..Default::default()
                },
                r"/** @jsx h */
/** @jsxRuntime classic */
export const foo = <div/>;",
            ),
            r#"import { newVNode } from "inferno";
/** @jsx h */
/** @jsxRuntime classic */
export const foo = newVNode(17, "div");"#,
        );
    }
}

/// transform-arrow-functions/with-this-member-expression
mod transform_arrow_functions_with_this_member_expression {
    use super::*;

    #[test]
    fn should_rewrite_this_in_member_tags_inside_arrow_functions() {
        assert_js_eq(
            &transform_with(
                "{}",
                r"const f = function () {
  return () => <this.foo.bar.qux />;
};",
            ),
            r#"import { newComponentVNode } from "inferno";
const f = function () {
  return () => newComponentVNode(0, this.foo.bar.qux);
};"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    #[test]
    fn static_children_should_mark_a_comment_and_an_element_as_unknown_children() {
        assert_transform(
            "<div>{ /* comment only */ }<span/></div>",
            r#"newVNode(1, "div", null, newVNode(17, "span"));"#,
        );
    }
}
