//! Ported from babel-plugin-inferno `tests/whitespace-text.test.js`: Whitespace and text

use crate::helpers::*;

/// multi-line text
mod multi_line_text {
    use super::*;

    #[test]
    fn should_join_multi_line_text_with_a_single_space() {
        assert_transform(
            r"<div>
  hello
  world
</div>",
            r#"createVNode(1, "div", null, "hello world", 16);"#,
        );
    }

    #[test]
    fn should_drop_blank_lines_inside_multi_line_text() {
        assert_transform(
            r"<div>
  a

  b
</div>",
            r#"createVNode(1, "div", null, "a b", 16);"#,
        );
    }

    #[test]
    fn should_drop_whitespace_only_first_and_last_lines() {
        assert_transform(
            r"<div>  
  a  
  </div>",
            r#"createVNode(1, "div", null, "a", 16);"#,
        );
    }

    #[test]
    fn should_drop_a_trailing_whitespace_only_line_after_text() {
        assert_transform(
            r"<div>a  
  </div>",
            r#"createVNode(1, "div", null, "a", 16);"#,
        );
    }

    #[test]
    fn should_keep_trailing_spaces_of_the_last_line() {
        assert_transform(
            r"<div>
  a
  b  </div>",
            r#"createVNode(1, "div", null, "a b  ", 16);"#,
        );
    }

    #[test]
    fn should_convert_tabs_to_spaces_and_trim_tab_indentation() {
        assert_transform(
            "<div>\n\t\ta\n\t\tb\n</div>",
            r#"createVNode(1, "div", null, "a b", 16);"#,
        );
    }

    #[test]
    fn should_treat_r_n_as_a_line_break() {
        assert_transform(
            "<div>a\r\nb</div>",
            r#"createVNode(1, "div", null, "a b", 16);"#,
        );
    }

    #[test]
    fn should_treat_a_lone_r_as_a_line_break() {
        assert_transform(
            "<div>a\rb</div>",
            r#"createVNode(1, "div", null, "a b", 16);"#,
        );
    }

    #[test]
    fn should_remove_whitespace_only_multi_line_children_of_an_element() {
        assert_transform(
            r"<div>

</div>",
            r#"createVNode(1, "div");"#,
        );
    }
}

/// single-line text
mod single_line_text {
    use super::*;

    #[test]
    fn should_keep_leading_and_trailing_spaces_of_single_line_text() {
        assert_transform(
            "<div>  hello  </div>",
            r#"createVNode(1, "div", null, "  hello  ", 16);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not convert tabs in JSX text to spaces"]
    fn should_convert_tabs_inside_single_line_text_to_spaces() {
        assert_transform(
            "<div>\ta\tb\t</div>",
            r#"createVNode(1, "div", null, " a b ", 16);"#,
        );
    }

    #[test]
    fn should_keep_a_single_space_between_two_expressions() {
        assert_transform(
            "<div>{a} {b}</div>",
            r#"createVNode(1, "div", null, [a, createTextVNode(" "), b], 0);"#,
        );
    }

    #[test]
    fn should_keep_spaces_around_a_single_expression_on_one_line() {
        assert_transform(
            "<div>  {a}  </div>",
            r#"createVNode(1, "div", null, [createTextVNode("  "), a, createTextVNode("  ")], 0);"#,
        );
    }

    #[test]
    fn should_drop_a_line_break_between_two_expressions() {
        assert_transform(
            r"<div>{a}
{b}</div>",
            r#"createVNode(1, "div", null, [a, b], 0);"#,
        );
    }

    #[test]
    fn should_drop_indentation_between_expressions() {
        assert_transform(
            r"<div>
  {a}
  {b}
</div>",
            r#"createVNode(1, "div", null, [a, b], 0);"#,
        );
    }
}

/// text next to expressions
mod text_next_to_expressions {
    use super::*;

    #[test]
    fn should_keep_the_space_between_text_and_an_expression_on_the_same_line() {
        assert_transform(
            r"<div>
  foo {bar}
</div>",
            r#"createVNode(1, "div", null, [createTextVNode("foo "), bar], 0);"#,
        );
    }

    #[test]
    fn should_split_text_lines_separated_by_an_expression_line() {
        assert_transform(
            r"<div>
  foo
  {bar}
  baz
</div>",
            r#"createVNode(1, "div", null, [createTextVNode("foo"), bar, createTextVNode("baz")], 0);"#,
        );
    }

    #[test]
    fn should_keep_an_explicit_child() {
        assert_transform(
            r#"<div>{a}{" "}{b}</div>"#,
            r#"createVNode(1, "div", null, [a, " ", b], 0);"#,
        );
    }
}

/// whitespace-only children
mod whitespace_only_children {
    use super::*;

    #[test]
    fn should_pass_single_line_whitespace_as_component_children() {
        assert_transform(
            "<Foo>  </Foo>",
            r#"createComponentVNode(2, Foo, {
  children: "  "
});"#,
        );
    }

    #[test]
    fn should_drop_indentation_around_a_single_component_child() {
        assert_transform(
            r"<Foo>
  <div/>
</Foo>",
            r#"createComponentVNode(2, Foo, {
  children: createVNode(1, "div")
});"#,
        );
    }

    #[test]
    fn should_create_no_children_for_a_component_with_only_a_line_break() {
        assert_transform(
            r"<Baz>
</Baz>",
            "createComponentVNode(2, Baz);",
        );
    }

    #[test]
    fn should_create_an_empty_fragment_when_it_only_contains_whitespace_lines() {
        assert_transform(
            r"<>
  
</>",
            "createFragment();",
        );
    }

    #[test]
    fn should_keep_single_line_whitespace_inside_a_long_syntax_fragment() {
        assert_transform(
            "<Fragment>  </Fragment>",
            r#"createFragment([createTextVNode("  ")], 4);"#,
        );
    }
}

/// non-breaking spaces
mod non_breaking_spaces {
    use super::*;

    #[test]
    fn should_not_trim_nbsp_on_its_own_line() {
        assert_transform(
            r"<div>
  &nbsp;
</div>",
            r#"createVNode(1, "div", null, "\xA0", 16);"#,
        );
    }

    #[test]
    fn should_keep_literal_non_breaking_spaces() {
        assert_transform(
            "<div>\u{A0} \u{A0}</div>",
            r#"createVNode(1, "div", null, "\xA0 \xA0", 16);"#,
        );
    }

    #[test]
    fn should_only_trim_spaces_and_tabs_not_literal_non_breaking_spaces() {
        assert_transform(
            "<div>\n  \u{A0}a\u{A0}\n</div>",
            r#"createVNode(1, "div", null, "\xA0a\xA0", 16);"#,
        );
    }
}
