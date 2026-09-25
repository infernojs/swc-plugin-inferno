//! Ported from babel-plugin-inferno `tests/parser-errors.test.js`: Parser errors

use crate::helpers::*;

// @babel/parser and swc_ecma_parser report this differently. @babel/parser reports:
// Adjacent JSX elements must be wrapped in an enclosing tag
#[test]
fn should_reject_adjacent_root_elements() {
    assert_transform_error(
        "var x = <div>one</div><div>two</div>;",
        "Expression expected",
    );
}

// @babel/parser and swc_ecma_parser report this differently. @babel/parser reports:
// Unexpected token `>`. Did you mean `&gt;` or `{'>'}`?
#[test]
fn should_reject_gt_in_jsx_text() {
    assert_transform_error(
        "<div>></div>",
        "Unexpected token. Did you mean `{'>'}` or `&gt;`?",
    );
}

// @babel/parser and swc_ecma_parser report this differently. @babel/parser reports:
// Unexpected token `}`. Did you mean `&rbrace;` or `{'}'}`?
#[test]
fn should_reject_rbrace_in_jsx_text() {
    assert_transform_error(
        "<div>}</div>",
        "Unexpected token. Did you mean `{'}'}` or `&rbrace;`?",
    );
}

#[test]
fn should_reject_mismatched_closing_tags() {
    assert_transform_error(
        "<Foo></Bar>",
        "Expected corresponding JSX closing tag for <Foo>",
    );
}

// @babel/parser and swc_ecma_parser report this differently. @babel/parser reports:
// Expected corresponding JSX closing tag for <>
#[test]
fn should_reject_a_fragment_closed_by_an_element_tag() {
    assert_transform_error("<></something>", "Expected '>', got 'ident'");
}

#[test]
fn should_reject_a_namespace_inside_a_member_expression() {
    assert_transform_error("<a.b:c />", "Unexpected token");
}

// swc_ecma_parser accepts this. @babel/parser reports:
// Sequence expressions cannot be directly nested inside JSX
#[test]
fn should_compile_an_unparenthesized_sequence_expression() {
    assert_transform(
        "<div a={b, c} />",
        "createVNode(1, \"div\", null, null, 1, {\n  \"a\": (b, c)\n});",
    );
}

#[test]
fn should_reject_an_unquoted_call_as_attribute_value() {
    assert_transform_error(
        "<Foo bar=bar() />",
        "JSX value should be either an expression or a quoted JSX text",
    );
}

// @babel/parser and swc_ecma_parser report this differently. @babel/parser reports:
// Unterminated JSX contents
#[test]
fn should_reject_unterminated_jsx_contents() {
    assert_transform_error("<foo>yes", "Expected '</', got '<eof>'");
}

#[test]
fn should_reject_attributes_on_a_short_syntax_fragment() {
    assert_transform_error(r#"< key="nope"></>"#, "Unexpected token");
}
