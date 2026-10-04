//! Ported from babel-plugin-inferno `tests/useless-flags.test.js`: Useless flags
//!
//! babel-plugin-inferno prints its warnings with `console.warn`, as
//! `babel-plugin-inferno: file:line:column: message` followed by a code frame.
//! swc-plugin-inferno reports the same message as an swc diagnostic, which swc
//! prints with the location and the code frame. The tests compare the message,
//! the location and the code the warning points at.

use crate::helpers::*;

const KNOWN: &str = " is not needed: the children are known at compile time, so the plugin sets their child flags. Child flags only help with dynamic children such as {expression}.";
const COMPONENT: &str =
    " has no effect on components. Their children are passed in props.children.";
const FRAGMENT: &str = " has no effect on Fragments.";

const CHILD_FLAGS: [&str; 5] = [
    "$HasVNodeChildren",
    "$HasTextChildren",
    "$HasNonKeyedChildren",
    "$HasKeyedChildren",
    "$ChildFlag={1}",
];

fn flag_name(flag: &str) -> &str {
    flag.split('=').next().unwrap()
}

fn needs(flag: &str) -> &'static str {
    match flag {
        "$HasVNodeChildren" => "one element or component child",
        "$HasTextChildren" => "one text child",
        "$HasNonKeyedChildren" => "an array of elements or components",
        "$HasKeyedChildren" => "an array of elements or components that all have a key",
        "$ChildFlag={1}" => "no children",
        _ => unreachable!("{flag}"),
    }
}

fn label(flag: &str) -> &str {
    match flag {
        "$ChildFlag={1}" => "$ChildFlag={1} (HasInvalidChildren)",
        _ => flag,
    }
}

/// Why each child flag cannot have children of each shape; flags that are missing fit the shape
const SHAPE_ERRORS: &[(&str, &[(&str, &str)])] = &[
    (
        "none",
        &[
            ("$HasVNodeChildren", "there are no children"),
            ("$HasTextChildren", "there are no children"),
            ("$HasNonKeyedChildren", "there are no children"),
            ("$HasKeyedChildren", "there are no children"),
        ],
    ),
    (
        "text",
        &[
            ("$HasVNodeChildren", "the child is text"),
            ("$HasNonKeyedChildren", "the child is text"),
            ("$HasKeyedChildren", "the child is text"),
            ("$ChildFlag={1}", "the child is text"),
        ],
    ),
    (
        "vnode",
        &[
            ("$HasTextChildren", "the child is an element"),
            (
                "$HasNonKeyedChildren",
                "the only child is an element, not an array",
            ),
            (
                "$HasKeyedChildren",
                "the only child is an element, not an array",
            ),
            ("$ChildFlag={1}", "the child is an element"),
        ],
    ),
    // Two elements without keys, or an element and an expression
    (
        "two",
        &[
            ("$HasVNodeChildren", "there are 2 children"),
            ("$HasTextChildren", "there are 2 children"),
            ("$HasKeyedChildren", "the child at index 0 has no key"),
            ("$ChildFlag={1}", "there are 2 children"),
        ],
    ),
    (
        "keyed",
        &[
            ("$HasVNodeChildren", "there are 2 children"),
            ("$HasTextChildren", "there are 2 children"),
            ("$ChildFlag={1}", "there are 2 children"),
        ],
    ),
    // Text and another child
    (
        "textFirst",
        &[
            ("$HasVNodeChildren", "there are 2 children"),
            ("$HasTextChildren", "there are 2 children"),
            (
                "$HasKeyedChildren",
                "the child at index 0 is text, which has no key",
            ),
            ("$ChildFlag={1}", "there are 2 children"),
        ],
    ),
    (
        "null",
        &[
            (
                "$HasVNodeChildren",
                "the child is null, which renders nothing",
            ),
            (
                "$HasTextChildren",
                "the child is null, which renders nothing",
            ),
            (
                "$HasNonKeyedChildren",
                "the child is null, which renders nothing",
            ),
            (
                "$HasKeyedChildren",
                "the child is null, which renders nothing",
            ),
        ],
    ),
    (
        "spread",
        &[
            (
                "$HasVNodeChildren",
                "the child is a spread, which makes an array",
            ),
            (
                "$HasTextChildren",
                "the child is a spread, which makes an array",
            ),
            (
                "$ChildFlag={1}",
                "the child is a spread, which makes an array",
            ),
        ],
    ),
    ("dynamic", &[]),
];

/// The error for a child flag on children of a shape, or `None` when the flag fits them
fn shape_error(flag: &str, shape: &str) -> Option<String> {
    let (_, reasons) = SHAPE_ERRORS
        .iter()
        .find(|(name, _)| *name == shape)
        .unwrap_or_else(|| panic!("unknown shape {shape}"));

    reasons
        .iter()
        .find(|(name, _)| *name == flag)
        .map(|(_, reason)| format!("{} needs {}, but {reason}.", label(flag), needs(flag)))
}

fn warnings_with(options: &str, input: &str) -> Vec<Warning> {
    transform_warnings(options, input).1
}

fn warnings(input: &str) -> Vec<Warning> {
    warnings_with("{}", input)
}

/// The message of each warning, without the location
fn messages_with(options: &str, input: &str) -> Vec<String> {
    warnings_with(options, input)
        .into_iter()
        .map(|warning| warning.message)
        .collect()
}

fn messages(input: &str) -> Vec<String> {
    messages_with("{}", input)
}

fn expected(messages: &[&str]) -> Vec<String> {
    messages.iter().map(|message| message.to_string()).collect()
}

/// children known at compile time
mod children_known_at_compile_time {
    use super::*;

    #[test]
    fn should_warn_about_has_vnode_children_on_a_single_static_element() {
        let input = "<div $HasVNodeChildren><h1>Hi</h1></div>";
        let (code, warnings) = transform_warnings("{}", input);

        assert_eq!(
            warnings,
            vec![Warning {
                location: Some("input.js:1:6".into()),
                message: format!("$HasVNodeChildren{KNOWN}"),
                snippet: "$HasVNodeChildren".into(),
            }]
        );
        assert_js_eq(
            &code,
            &strip_inferno_import(&transform_with(r#"{"uselessFlags": "off"}"#, input)),
        );
    }

    #[test]
    fn should_point_at_the_flag_in_multi_line_code() {
        let warnings = warnings(
            "function App() {\n  return (\n    <div $HasVNodeChildren>\n      <h1>Hi</h1>\n    </div>\n  );\n}",
        );

        assert_eq!(
            warnings,
            vec![Warning {
                location: Some("input.js:3:10".into()),
                message: format!("$HasVNodeChildren{KNOWN}"),
                snippet: "$HasVNodeChildren".into(),
            }]
        );
    }

    // The shape of the children as the JSX shows it, see SHAPE_ERRORS
    const SHAPES: [(&str, &str); 24] = [
        ("<div FLAG />", "none"),
        ("<div FLAG>\n  </div>", "none"),
        ("<div FLAG>text</div>", "text"),
        ("<div FLAG><a/></div>", "vnode"),
        ("<div FLAG><Foo/></div>", "vnode"),
        ("<div FLAG><></></div>", "vnode"),
        ("<div FLAG><a/><b/></div>", "two"),
        ("<div FLAG><a key=\"1\"/><b key=\"2\"/></div>", "keyed"),
        ("<div FLAG>text<a/></div>", "textFirst"),
        ("<input FLAG />", "none"),
        ("<div FLAG children=\"t\" />", "text"),
        ("<div FLAG children={<a/>} />", "vnode"),
        ("<div FLAG children=<a/> />", "vnode"),
        ("<div FLAG children={<></>} />", "vnode"),
        ("<div FLAG children={null} />", "null"),
        ("<div FLAG children={a}><b/></div>", "vnode"),
        ("<Fragment FLAG />", "none"),
        ("<Fragment FLAG>text</Fragment>", "text"),
        ("<Fragment FLAG><a/></Fragment>", "vnode"),
        ("<Fragment FLAG><a/><b/></Fragment>", "two"),
        (
            "<Fragment FLAG key=\"k\"><a key=\"1\"/><b key=\"2\"/></Fragment>",
            "keyed",
        ),
        ("<Fragment FLAG children=\"t\" />", "text"),
        ("<Fragment FLAG children={null} />", "null"),
        ("<Inferno.Fragment FLAG><a/></Inferno.Fragment>", "vnode"),
    ];

    #[test]
    fn should_throw_for_or_warn_about_every_shape() {
        let cases = CHILD_FLAGS.into_iter().flat_map(|flag| {
            SHAPES.map(|(input, shape)| (flag, input.replace("FLAG", flag), shape))
        });

        assert_all(cases, |(flag, input, shape)| {
            match shape_error(flag, shape) {
                Some(error) => assert_transform_error(input, &error),
                None => assert_eq!(messages(input), vec![format!("{}{KNOWN}", flag_name(flag))]),
            }
        });
    }

    #[test]
    fn should_warn_about_every_child_flag_when_there_are_several() {
        assert_eq!(
            messages(
                "<div $HasKeyedChildren $HasNonKeyedChildren><a key=\"1\"/><b key=\"2\"/></div>"
            ),
            vec![
                format!("$HasKeyedChildren{KNOWN}"),
                format!("$HasNonKeyedChildren{KNOWN}"),
            ]
        );
    }

    #[test]
    fn should_warn_with_a_spread_attribute() {
        assert_eq!(
            messages("<div {...p} $HasVNodeChildren><a/></div>"),
            vec![format!("$HasVNodeChildren{KNOWN}")]
        );
    }

    #[test]
    fn should_not_warn_about_flags_on_static_children() {
        assert_eq!(warnings("<div $Flags={1}><a/></div>"), vec![]);
    }
}

/// dynamic children
mod dynamic_children {
    use super::*;

    const SHAPES: [(&str, &str); 15] = [
        ("<div FLAG>{a}</div>", "dynamic"),
        ("<div FLAG>{...a}</div>", "spread"),
        ("<div FLAG>text{a}</div>", "textFirst"),
        ("<div FLAG><a/>{b}</div>", "two"),
        ("<div FLAG>{<a/>}</div>", "vnode"),
        ("<div FLAG>{\"text\"}</div>", "text"),
        ("<div FLAG>{/* c */}<a/></div>", "vnode"),
        ("<div FLAG children={a} />", "dynamic"),
        ("<div FLAG children={\"t\"} />", "text"),
        ("<div FLAG children={a}>\n  </div>", "dynamic"),
        ("<div {...p} FLAG>{a}</div>", "dynamic"),
        ("<Fragment FLAG>{a}</Fragment>", "dynamic"),
        ("<Fragment FLAG children={a} />", "dynamic"),
        ("<Fragment FLAG children={<a/>} />", "vnode"),
        ("<Fragment FLAG children=<a/> />", "vnode"),
    ];

    #[test]
    fn should_throw_for_or_not_warn_about_every_shape() {
        let cases = CHILD_FLAGS.into_iter().flat_map(|flag| {
            SHAPES.map(|(input, shape)| (flag, input.replace("FLAG", flag), shape))
        });

        assert_all(cases, |(flag, input, shape)| {
            match shape_error(flag, shape) {
                Some(error) => assert_transform_error(input, &error),
                None => assert_eq!(warnings(input), vec![]),
            }
        });
    }

    #[test]
    fn should_not_warn_about_a_child_flag_expression() {
        assert_eq!(warnings("<div $ChildFlag={x}>{a}</div>"), vec![]);
    }
}

/// components
mod components {
    use super::*;

    #[test]
    fn should_warn_about_each_child_flag_on_a_component() {
        assert_all(CHILD_FLAGS, |flag| {
            assert_eq!(
                messages(&format!("<Foo {flag}>{{a}}</Foo>")),
                vec![format!("{}{COMPONENT}", flag_name(flag))]
            );
        });
    }

    #[test]
    fn should_warn_about_a_child_flag_on_a_component_without_children() {
        assert_eq!(
            messages("<Foo $HasVNodeChildren />"),
            vec![format!("$HasVNodeChildren{COMPONENT}")]
        );
    }

    #[test]
    fn should_warn_about_a_child_flag_on_a_member_expression_component() {
        assert_eq!(
            messages("<Ns.Foo $HasTextChildren>text</Ns.Foo>"),
            vec![format!("$HasTextChildren{COMPONENT}")]
        );
    }

    #[test]
    fn should_warn_about_a_child_flag_on_a_component_with_type_arguments() {
        let compiled = compile_warnings(
            Setup {
                lang: Lang::Tsx,
                ..Default::default()
            },
            "<Foo<T> $HasKeyedChildren>{a}</Foo>",
        );

        assert_eq!(
            compiled.warnings,
            vec![Warning {
                location: Some("input.tsx:1:9".into()),
                message: format!("$HasKeyedChildren{COMPONENT}"),
                snippet: "$HasKeyedChildren".into(),
            }]
        );
    }

    #[test]
    fn should_warn_about_each_child_flag_on_one_component() {
        assert_eq!(
            messages("<Foo $HasKeyedChildren $ChildFlag={1}>{a}</Foo>"),
            vec![
                format!("$HasKeyedChildren{COMPONENT}"),
                format!("$ChildFlag{COMPONENT}"),
            ]
        );
    }

    #[test]
    fn should_compile_the_same_without_the_child_flag() {
        assert_js_eq(
            &transform("<Foo $HasKeyedChildren>{a}</Foo>"),
            &transform("<Foo>{a}</Foo>"),
        );
    }

    #[test]
    fn should_not_warn_about_flags_on_a_component() {
        assert_eq!(warnings("<Foo $Flags={4}/>"), vec![]);
    }
}

/// conflicting flags
mod conflicting_flags {
    use super::*;

    /// The input, the ignored flag and the flag that takes precedence
    const CASES: [(&str, &str, &str); 8] = [
        (
            "<div $HasKeyedChildren $HasNonKeyedChildren>{a}</div>",
            "$HasNonKeyedChildren",
            "$HasKeyedChildren",
        ),
        (
            "<div $HasNonKeyedChildren $HasKeyedChildren>{a}</div>",
            "$HasNonKeyedChildren",
            "$HasKeyedChildren",
        ),
        (
            "<div $HasTextChildren $HasVNodeChildren>{a}</div>",
            "$HasVNodeChildren",
            "$HasTextChildren",
        ),
        (
            "<div $HasNonKeyedChildren $HasTextChildren>{a}</div>",
            "$HasTextChildren",
            "$HasNonKeyedChildren",
        ),
        (
            "<div $ChildFlag={1} $HasKeyedChildren>{a}</div>",
            "$HasKeyedChildren",
            "$ChildFlag",
        ),
        (
            "<div $HasVNodeChildren $ChildFlag={x}>{a}</div>",
            "$HasVNodeChildren",
            "$ChildFlag",
        ),
        (
            "<Fragment $HasTextChildren $HasVNodeChildren>{a}</Fragment>",
            "$HasVNodeChildren",
            "$HasTextChildren",
        ),
        (
            "<Fragment $HasKeyedChildren $HasNonKeyedChildren key=\"k\">{a}</Fragment>",
            "$HasNonKeyedChildren",
            "$HasKeyedChildren",
        ),
    ];

    /// `input` without the attribute `flag`
    fn without(input: &str, flag: &str) -> String {
        let start = input.find(&format!(" {flag}")).unwrap();
        let mut end = start + 1 + flag.len();

        if input[end..].starts_with("={") {
            end += input[end..].find('}').unwrap() + 1;
        }
        format!("{}{}", &input[..start], &input[end..])
    }

    #[test]
    fn should_warn_about_the_ignored_flag() {
        assert_all(CASES, |(input, ignored, winner)| {
            assert_eq!(
                messages(input),
                vec![format!(
                    "{ignored} is ignored because {winner} takes precedence. Remove one of them."
                )]
            );
        });
    }

    #[test]
    fn should_compile_the_same_without_the_ignored_flag() {
        assert_all(CASES, |(input, ignored, _)| {
            assert_js_eq(&transform(input), &transform(&without(input, ignored)));
        });
    }

    #[test]
    fn should_warn_about_every_ignored_child_flag() {
        assert_eq!(
            messages("<div $HasVNodeChildren $HasTextChildren $ChildFlag={x}>{a}</div>"),
            expected(&[
                "$HasVNodeChildren is ignored because $ChildFlag takes precedence. Remove one of them.",
                "$HasTextChildren is ignored because $ChildFlag takes precedence. Remove one of them.",
            ])
        );
    }

    #[test]
    fn should_point_at_the_ignored_flag() {
        let warnings = warnings("<div $HasKeyedChildren $HasNonKeyedChildren>{a}</div>");

        assert_eq!(warnings[0].location.as_deref(), Some("input.js:1:24"));
        assert_eq!(warnings[0].snippet, "$HasNonKeyedChildren");
    }
}

/// Fragments
mod fragments {
    use super::*;

    const CASES: [(&str, &str); 3] = [
        ("<Fragment $Flags={1}>{x}</Fragment>", "$Flags"),
        (
            "<Inferno.Fragment $Flags={1} key=\"k\">{x}</Inferno.Fragment>",
            "$Flags",
        ),
        ("<React.Fragment $Flags={1}>{x}</React.Fragment>", "$Flags"),
    ];

    #[test]
    fn should_warn_about_the_flag() {
        assert_all(CASES, |(input, flag)| {
            assert_eq!(messages(input), vec![format!("{flag}{FRAGMENT}")]);
        });
    }

    #[test]
    fn should_compile_the_same_without_the_flag() {
        assert_all(CASES, |(input, _)| {
            let without = input.replace(" $Flags={1}", "");

            assert_js_eq(&transform(input), &transform(&without));
        });
    }

    #[test]
    fn should_warn_about_a_fragment_flag_and_a_child_flag_separately() {
        assert_eq!(
            messages("<Fragment $Flags={1} $HasNonKeyedChildren><a/><b/></Fragment>"),
            vec![
                format!("$Flags{FRAGMENT}"),
                format!("$HasNonKeyedChildren{KNOWN}"),
            ]
        );
    }
}

/// nested JSX
mod nested_jsx {
    use super::*;

    #[test]
    fn should_warn_about_a_flag_on_a_child_element() {
        let warnings = warnings("<div>{a}<p $HasTextChildren>text</p></div>");

        assert_eq!(
            warnings,
            vec![Warning {
                location: Some("input.js:1:12".into()),
                message: format!("$HasTextChildren{KNOWN}"),
                snippet: "$HasTextChildren".into(),
            }]
        );
    }

    #[test]
    fn should_warn_about_a_flag_in_jsx_inside_an_attribute() {
        assert_eq!(
            messages("<Foo icon={<b $HasVNodeChildren><i/></b>} />"),
            vec![format!("$HasVNodeChildren{KNOWN}")]
        );
    }

    #[test]
    fn should_warn_about_a_flag_in_a_children_prop() {
        assert_eq!(
            messages("<div children={<b $HasVNodeChildren><i/></b>} />"),
            vec![format!("$HasVNodeChildren{KNOWN}")]
        );
    }

    #[test]
    fn should_warn_once_for_each_element() {
        assert_eq!(
            messages(
                "<ul $HasNonKeyedChildren><li $HasTextChildren>a</li><li $HasTextChildren>b</li></ul>"
            ),
            vec![
                format!("$HasTextChildren{KNOWN}"),
                format!("$HasTextChildren{KNOWN}"),
                format!("$HasNonKeyedChildren{KNOWN}"),
            ]
        );
    }
}

/// uselessFlags option
mod useless_flags_option {
    use super::*;

    const INPUT: &str = "<div $HasVNodeChildren><h1>Hi</h1></div>";

    fn with_options(options: &str) -> Setup<'_> {
        Setup {
            options,
            ..Default::default()
        }
    }

    #[test]
    fn should_warn_by_default() {
        assert_eq!(
            messages_with("{}", INPUT),
            vec![format!("$HasVNodeChildren{KNOWN}")]
        );
    }

    #[test]
    fn should_warn_with_warn() {
        assert_eq!(
            messages_with(r#"{"uselessFlags": "warn"}"#, INPUT),
            vec![format!("$HasVNodeChildren{KNOWN}")]
        );
    }

    #[test]
    fn should_not_warn_with_off() {
        assert_eq!(warnings_with(r#"{"uselessFlags": "off"}"#, INPUT), vec![]);
        assert_eq!(
            warnings_with(
                r#"{"uselessFlags": "off"}"#,
                "<Foo $HasKeyedChildren>{a}</Foo>"
            ),
            vec![]
        );
    }

    // babel-plugin-inferno throws at the first useless flag. swc-plugin-inferno reports every
    // useless flag as an error, like its other errors, and swc prints them with a code frame.
    #[test]
    fn should_throw_with_error() {
        assert_compile_error(
            with_options(r#"{"uselessFlags": "error"}"#),
            INPUT,
            &format!("$HasVNodeChildren{KNOWN}"),
        );
    }

    #[test]
    fn should_throw_for_conflicting_flags_with_error() {
        assert_compile_error(
            with_options(r#"{"uselessFlags": "error"}"#),
            "<div $HasKeyedChildren $HasNonKeyedChildren>{a}</div>",
            "$HasNonKeyedChildren is ignored because $HasKeyedChildren takes precedence. Remove one of them.",
        );
    }

    #[test]
    fn should_compile_the_same_with_every_level() {
        let expected = transform_warnings(r#"{"uselessFlags": "off"}"#, INPUT).0;

        assert_eq!(
            transform_warnings(r#"{"uselessFlags": "warn"}"#, INPUT).0,
            expected
        );
        assert_eq!(transform_warnings("{}", INPUT).0, expected);
    }

    // babel-plugin-inferno throws when Babel loads the config. swc-plugin-inferno rejects the
    // plugin options with the same message.
    #[test]
    fn should_reject_an_unknown_level() {
        let err =
            serde_json::from_str::<swc_plugin_inferno::Options>(r#"{"uselessFlags": "warning"}"#)
                .unwrap_err();

        assert!(
            err.to_string().contains(
                r#"the uselessFlags option must be "warn", "error" or "off", got "warning"."#
            ),
            "{err}"
        );
    }

    #[test]
    fn should_reject_a_boolean_level() {
        let err = serde_json::from_str::<swc_plugin_inferno::Options>(r#"{"uselessFlags": false}"#)
            .unwrap_err();

        assert!(
            err.to_string().contains(
                r#"the uselessFlags option must be "warn", "error" or "off", got false."#
            ),
            "{err}"
        );
    }

    // babel-plugin-inferno only accepts a missing option; swc-plugin-inferno rejects null like it.
    #[test]
    fn should_reject_a_null_level() {
        let err = serde_json::from_str::<swc_plugin_inferno::Options>(r#"{"uselessFlags": null}"#)
            .unwrap_err();

        assert!(
            err.to_string()
                .contains(r#"the uselessFlags option must be "warn", "error" or "off", got null."#),
            "{err}"
        );
    }

    // Not ported:
    // - Should reject an unknown level in files without flags (swc-plugin-inferno reads its options before it sees the file, see should_reject_an_unknown_level)
}

/// warning format
mod warning_format {
    use super::*;

    use swc_core::{
        common::DUMMY_SP,
        ecma::{
            ast::*,
            visit::{VisitMut, VisitMutWith, visit_mut_pass},
        },
    };

    #[test]
    fn should_include_the_file_name() {
        let compiled =
            compile_warnings(Setup::default(), "<div $HasVNodeChildren><h1>Hi</h1></div>");

        assert_eq!(compiled.warnings.len(), 1);
        assert_eq!(
            compiled.warnings[0].location.as_deref(),
            Some("input.js:1:6")
        );
    }

    /// Replaces `makeJSX()` with `<div $HasTextChildren>text</div>` built without spans.
    struct MakeJsx;

    impl VisitMut for MakeJsx {
        fn visit_mut_expr(&mut self, expr: &mut Expr) {
            expr.visit_mut_children_with(self);

            let Expr::Call(CallExpr {
                callee: Callee::Expr(callee),
                ..
            }) = expr
            else {
                return;
            };
            if !matches!(&**callee, Expr::Ident(i) if i.sym == "makeJSX") {
                return;
            }

            let name = || JSXElementName::Ident(Ident::new_no_ctxt("div".into(), DUMMY_SP));

            *expr = Expr::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: name(),
                    attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName::new(
                            "$HasTextChildren".into(),
                            DUMMY_SP,
                        )),
                        value: None,
                    })],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: "text".into(),
                    raw: "text".into(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: name(),
                }),
            }));
        }
    }

    #[test]
    fn should_warn_without_a_code_frame_about_jsx_built_without_source_locations() {
        let compiled = compile_with(
            Setup::default(),
            "const el = makeJSX();",
            visit_mut_pass(MakeJsx),
        )
        .unwrap();

        assert_eq!(
            compiled.warnings,
            vec![Warning {
                location: None,
                message: format!("$HasTextChildren{KNOWN}"),
                snippet: String::new(),
            }]
        );
        assert_js_eq(
            &strip_inferno_import(&compiled.code),
            r#"const el = newVNode(3, "div", null, "text");"#,
        );
    }
}
