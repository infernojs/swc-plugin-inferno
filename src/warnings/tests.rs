//! The expected warnings are the ones babel-plugin-inferno prints for the same input, with its
//! name replaced.

use super::*;
use crate::{Options, inferno};
use swc_core::{
    common::{FileName, Mark, comments::SingleThreadedComments},
    ecma::{ast::Program, transforms::base::resolver},
};
use swc_ecma_parser::{EsSyntax, Syntax, parse_file_as_module};

const KNOWN: &str = " is not needed: the children are known at compile time, so the plugin sets their child flags. Child flags only help with dynamic children such as {expression}.";

/// The warnings printed for `input` in the file App.jsx
fn warnings(input: &str) -> Vec<String> {
    testing::run_test(false, |cm, _| {
        let fm = cm.new_source_file(FileName::Real("App.jsx".into()).into(), input.to_string());
        let comments = SingleThreadedComments::default();
        let module = parse_file_as_module(
            &fm,
            Syntax::Es(EsSyntax {
                jsx: true,
                ..Default::default()
            }),
            Default::default(),
            Some(&comments),
            &mut vec![],
        )
        .unwrap();
        let unresolved_mark = Mark::new();
        let (_, warnings) = collect_warnings(&*cm, "App.jsx", || {
            Program::Module(module).apply((
                resolver(unresolved_mark, Mark::new(), false),
                inferno(
                    cm.clone(),
                    Some(comments.clone()),
                    Options::default(),
                    unresolved_mark,
                ),
            ))
        });

        Ok(warnings)
    })
    .unwrap()
}

#[track_caller]
fn assert_warning(input: &str, location: &str, frame: &str) {
    assert_eq!(
        warnings(input),
        vec![format!(
            "swc-plugin-inferno: App.jsx:{location}: $HasVNodeChildren{KNOWN}\n{frame}"
        )]
    );
}

#[test]
fn should_print_a_single_line() {
    assert_warning(
        "<div $HasVNodeChildren><h1>Hi</h1></div>",
        "1:6",
        "> 1 | <div $HasVNodeChildren><h1>Hi</h1></div>\n    |      ^^^^^^^^^^^^^^^^^",
    );
}

#[test]
fn should_print_two_lines_above_and_three_below() {
    assert_warning(
        "function App() {\n  return (\n    <div $HasVNodeChildren>\n      <h1>Hi</h1>\n    </div>\n  );\n}",
        "3:10",
        "  1 | function App() {\n  2 |   return (\n> 3 |     <div $HasVNodeChildren>\n    |          ^^^^^^^^^^^^^^^^^\n  4 |       <h1>Hi</h1>\n  5 |     </div>\n  6 |   );",
    );
}

#[test]
fn should_align_line_numbers_of_different_widths() {
    assert_warning(
        "\n\n\n\n\n\n\n\nconst x = (\n  <div $HasVNodeChildren>\n    <h1>Hi</h1>\n  </div>\n);\n",
        "10:8",
        "   8 |\n   9 | const x = (\n> 10 |   <div $HasVNodeChildren>\n     |        ^^^^^^^^^^^^^^^^^\n  11 |     <h1>Hi</h1>\n  12 |   </div>\n  13 | );",
    );
}

#[test]
fn should_mark_one_character_of_a_flag_that_spans_lines() {
    assert_eq!(
        warnings("<div $ChildFlag={\n  1\n}><a/></div>"),
        vec![format!(
            "swc-plugin-inferno: App.jsx:1:6: $ChildFlag{KNOWN}\n> 1 | <div $ChildFlag={{\n    |      ^\n  2 |   1\n  3 | }}><a/></div>"
        )]
    );
}

#[test]
fn should_keep_tabs_in_front_of_the_markers() {
    assert_warning(
        "\t<div\t$HasVNodeChildren><a/></div>",
        "1:7",
        "> 1 | \t<div\t$HasVNodeChildren><a/></div>\n    | \t    \t^^^^^^^^^^^^^^^^^",
    );
}

#[test]
fn should_split_lines_at_crlf() {
    assert_warning(
        "const a = 1;\r\nconst b = <div $HasVNodeChildren><a/></div>;\r\n",
        "2:16",
        "  1 | const a = 1;\n> 2 | const b = <div $HasVNodeChildren><a/></div>;\n    |                ^^^^^^^^^^^^^^^^^\n  3 |",
    );
}

#[test]
fn should_print_empty_lines_without_a_trailing_space() {
    assert_warning(
        "const a = 1;\n\nconst b = <div $HasVNodeChildren><a/></div>;\n\n\nx;",
        "3:16",
        "  1 | const a = 1;\n  2 |\n> 3 | const b = <div $HasVNodeChildren><a/></div>;\n    |                ^^^^^^^^^^^^^^^^^\n  4 |\n  5 |\n  6 | x;",
    );
}

#[test]
fn should_count_columns_in_utf16_code_units() {
    assert_warning(
        "const s = \"😀\"; const b = <div $HasVNodeChildren><a/></div>;",
        "1:32",
        "> 1 | const s = \"😀\"; const b = <div $HasVNodeChildren><a/></div>;\n    |                                ^^^^^^^^^^^^^^^^^",
    );
}

#[test]
fn should_print_a_warning_without_a_position_without_a_code_frame() {
    assert_eq!(
        format_warning(
            &swc_core::common::SourceMap::default(),
            "unknown file",
            None,
            "$HasTextChildren is not needed."
        ),
        "swc-plugin-inferno: unknown file: $HasTextChildren is not needed."
    );
}

#[test]
fn should_pass_errors_on() {
    let err = testing::run_test(false, |cm, handler| {
        collect_warnings(&*cm, "App.jsx", || {
            HANDLER.with(|handler| handler.err("an error"));
        });
        assert!(handler.has_errors());

        Err::<(), _>(())
    })
    .unwrap_err();

    assert!(err.to_string().contains("an error"), "{err}");
}
