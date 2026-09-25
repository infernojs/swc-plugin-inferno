//! Ported from babel-plugin-inferno `tests/babel-parity.test.js`: Babel parity

use crate::helpers::*;

/// transform-react-jsx fixtures
mod transform_react_jsx_fixtures {
    use super::*;

    #[test]
    fn should_convert_simple_tags() {
        assert_transform("var x = <div></div>;", r#"var x = createVNode(1, "div");"#);
    }

    #[test]
    fn should_convert_simple_text() {
        assert_transform(
            "var x = <div>text</div>;",
            r#"var x = createVNode(1, "div", null, "text", 16);"#,
        );
    }

    #[test]
    fn should_allow_js_namespacing() {
        assert_transform(
            "<Namespace.Component />;",
            "createComponentVNode(2, Namespace.Component);",
        );
    }

    #[test]
    fn should_allow_deeper_js_namespacing() {
        assert_transform(
            "<Namespace.DeepNamespace.Component />;",
            "createComponentVNode(2, Namespace.DeepNamespace.Component);",
        );
    }

    #[test]
    fn should_transform_known_hyphenated_tags() {
        assert_transform("<font-face />;", r#"createVNode(32, "font-face");"#);
    }

    #[test]
    fn this_tag_name() {
        assert_transform(
            "var div = <this.foo>test</this.foo>;",
            r#"var div = createComponentVNode(2, this.foo, {
  children: "test"
});"#,
        );
    }

    #[test]
    fn assignment() {
        assert_transform(
            r#"var div = <Component {...props} foo="bar" />"#,
            r#"var div = normalizeProps(createComponentVNode(2, Component, {
  ...props,
  "foo": "bar"
}));"#,
        );
    }

    #[test]
    fn should_allow_elements_as_attributes() {
        assert_transform(
            "<div attr=<div /> />",
            r#"createVNode(1, "div", null, null, 1, {
  "attr": createVNode(1, "div")
});"#,
        );
    }

    #[test]
    fn should_handle_attributed_elements() {
        assert_transform(
            r"var HelloMessage = React.createClass({
  render: function() {
    return <div>Hello {this.props.name}</div>;
  }
});

React.render(<HelloMessage name={
  <span>
    Sebastian
  </span>
} />, mountNode);",
            r#"var HelloMessage = React.createClass({
  render: function () {
    return createVNode(1, "div", null, [createTextVNode("Hello "), this.props.name], 0);
  }
});
React.render(createComponentVNode(2, HelloMessage, {
  "name": createVNode(1, "span", null, "Sebastian", 16)
}), mountNode);"#,
        );
    }

    #[test]
    fn should_not_add_quotes_to_identifier_names() {
        assert_transform(
            "var e = <F aaa new const var default foo-bar/>;",
            r#"var e = createComponentVNode(2, F, {
  "aaa": true,
  "new": true,
  "const": true,
  "var": true,
  "default": true,
  "foo-bar": true
});"#,
        );
    }

    #[test]
    fn should_quote_jsx_attributes() {
        assert_transform(
            "<button data-value='a value'>Button</button>;",
            r#"createVNode(1, "button", null, "Button", 16, {
  "data-value": "a value"
});"#,
        );
    }

    #[test]
    fn should_not_mangle_expressioncontainer_attribute_values() {
        assert_transform(
            r#"<button data-value={"a value\n  with\nnewlines\n   and spaces"}>Button</button>;"#,
            r#"createVNode(1, "button", null, "Button", 16, {
  "data-value": "a value\n  with\nnewlines\n   and spaces"
});"#,
        );
    }

    // babel-plugin-inferno keeps the object literal spread; swc-plugin-inferno flattens it into the props,
    // which creates the same props.
    #[test]
    fn duplicate_props_spread_variants() {
        assert_transform(
            r"<p {...{prop, prop}}></p>;
<p prop {...{prop}}></p>;
<p {...{prop}} prop></p>;",
            r#"normalizeProps(createVNode(1, "p", null, null, 1, {
    prop: prop,
    prop: prop
}));
normalizeProps(createVNode(1, "p", null, null, 1, {
    prop: true,
    prop: prop
}));
normalizeProps(createVNode(1, "p", null, null, 1, {
    prop: prop,
    prop: true
}));"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn duplicate_props_repeated_attribute() {
        assert_transform_error(
            "<p prop prop></p>;",
            "Multiple prop props are not supported. Remove the duplicate prop prop.",
        );
    }

    // babel-plugin-inferno keeps the object literal spread; swc-plugin-inferno flattens it into the props,
    // which creates the same props.
    #[test]
    fn flattens_spread() {
        assert_transform(
            r#"<p {...props}>text</p>;
<div {...props}>{contents}</div>;
<img alt="" {...{src, title}} />;
<blockquote {...{cite}}>{items}</blockquote>;"#,
            r#"normalizeProps(createVNode(1, "p", null, "text", 16, {
    ...props
}));
normalizeProps(createVNode(1, "div", null, contents, 0, {
    ...props
}));
normalizeProps(createVNode(1, "img", null, null, 1, {
    alt: "",
    src: src,
    title: title
}));
normalizeProps(createVNode(1, "blockquote", null, items, 0, {
    cite: cite
}));"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno flattens an object literal spread containing __proto__, which sets the prototype of the props"]
    fn handle_spread_with_proto() {
        assert_transform(
            r#"<p {...{__proto__: null}}>text</p>;
<div {...{"__proto__": null}}>{contents}</div>;"#,
            r#"normalizeProps(createVNode(1, "p", null, "text", 16, {
  ...{
    __proto__: null
  }
}));
normalizeProps(createVNode(1, "div", null, contents, 0, {
  ...{
    "__proto__": null
  }
}));"#,
        );
    }

    #[test]
    fn wraps_props_in_react_spread_for_first_spread_attributes() {
        assert_transform(
            r"<Component { ... x } y
={2 } z />",
            r#"normalizeProps(createComponentVNode(2, Component, {
  ...x,
  "y": 2,
  "z": true
}));"#,
        );
    }

    #[test]
    fn wraps_props_in_react_spread_for_last_spread_attributes() {
        assert_transform(
            "<Component y={2} z { ... x } />",
            r#"normalizeProps(createComponentVNode(2, Component, {
  "y": 2,
  "z": true,
  ...x
}));"#,
        );
    }

    #[test]
    fn wraps_props_in_react_spread_for_middle_spread_attributes() {
        assert_transform(
            "<Component y={2} { ... x } z />",
            r#"normalizeProps(createComponentVNode(2, Component, {
  "y": 2,
  ...x,
  "z": true
}));"#,
        );
    }

    #[test]
    fn should_escape_xhtml_jsxtext() {
        assert_transform(
            "<div>wow</div>;\n<div>wôw</div>;\n<div>w & w</div>;\n<div>w &amp; w</div>;\n<div>w &nbsp; w</div>;\n<div>this should not parse as unicode: \u{A0}</div>;\n<div>this should parse as nbsp: \u{A0} </div>;\n<div>this should parse as unicode: {'\u{A0} '}</div>;\n<div>w &lt; w</div>;",
            "createVNode(1, \"div\", null, \"wow\", 16);\ncreateVNode(1, \"div\", null, \"wôw\", 16);\ncreateVNode(1, \"div\", null, \"w & w\", 16);\ncreateVNode(1, \"div\", null, \"w & w\", 16);\ncreateVNode(1, \"div\", null, \"w \\xA0 w\", 16);\ncreateVNode(1, \"div\", null, \"this should not parse as unicode: \\xA0\", 16);\ncreateVNode(1, \"div\", null, \"this should parse as nbsp: \\xA0 \", 16);\ncreateVNode(1, \"div\", null, [createTextVNode(\"this should parse as unicode: \"), createTextVNode('\u{A0} ')], 0);\ncreateVNode(1, \"div\", null, \"w < w\", 16);",
        );
    }

    #[test]
    fn should_not_strip_nbsp_even_coupled_with_other_whitespace() {
        assert_transform(
            "<div>&nbsp; </div>;",
            r#"createVNode(1, "div", null, "\xA0 ", 16);"#,
        );
    }

    #[test]
    fn should_not_strip_tags_with_a_single_child_of_nbsp() {
        assert_transform(
            "<div>&nbsp;</div>;",
            r#"createVNode(1, "div", null, "\xA0", 16);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno wraps text in createTextVNode inside component children"]
    fn weird_symbols() {
        assert_transform(
            r"class MobileHomeActivityTaskPriorityIcon extends React.PureComponent {
  render() {
    return <Text>&nbsp;{this.props.value}&nbsp;</Text>;
  }
}",
            r#"class MobileHomeActivityTaskPriorityIcon extends React.PureComponent {
  render() {
    return createComponentVNode(2, Text, {
      children: ["\xA0", this.props.value, "\xA0"]
    });
  }
}"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno wraps text in createTextVNode inside component children"]
    fn dont_coerce_expression_containers() {
        assert_transform(
            r#"<Text>
  To get started, edit index.ios.js!!!{"\n"}
  Press Cmd+R to reload
</Text>"#,
            r#"createComponentVNode(2, Text, {
  children: ["To get started, edit index.ios.js!!!", "\n", "Press Cmd+R to reload"]
});"#,
        );
    }

    #[test]
    fn concatenates_adjacent_string_literals() {
        assert_transform(
            r#"var x =
  <div>
    foo
    {"bar"}
    baz
    <div>
      buz
      bang
    </div>
    qux
    {null}
    quack
  </div>"#,
            r#"var x = createVNode(1, "div", null, [createTextVNode("foo"), createTextVNode("bar"), createTextVNode("baz"), createVNode(1, "div", null, "buz bang", 16), createTextVNode("qux"), null, createTextVNode("quack")], 0);"#,
        );
    }

    #[test]
    fn should_insert_commas_after_expressions_before_whitespace() {
        assert_transform(
            r#"var x =
  <div
    attr1={
      "foo" + "bar"
    }
    attr2={
      "foo" + "bar" +

      "baz" + "bug"
    }
    attr3={
      "foo" + "bar" +
      "baz" + "bug"
      // Extra line here.
    }
    attr4="baz">
  </div>"#,
            r#"var x = createVNode(1, "div", null, null, 1, {
  "attr1": "foo" + "bar",
  "attr2": "foo" + "bar" + "baz" + "bug",
  "attr3": "foo" + "bar" + "baz" + "bug"
  // Extra line here.
  ,
  "attr4": "baz"
});"#,
        );
    }

    #[test]
    fn should_have_correct_comma_in_nested_children() {
        assert_transform(
            r"var x = <div>
  <div><br /></div>
  <Component>{foo}<br />{bar}</Component>
  <br />
</div>;",
            r#"var x = createVNode(1, "div", null, [createVNode(1, "div", null, createVNode(1, "br"), 2), createComponentVNode(2, Component, {
  children: [foo, createVNode(1, "br"), bar]
}), createVNode(1, "br")], 4);"#,
        );
    }

    #[test]
    fn should_avoid_wrapping_in_extra_parens_if_not_needed() {
        assert_transform(
            r"var x = <div>
  <Component />
</div>;

var x = <div>
  {props.children}
</div>;

var x = <Composite>
  {props.children}
</Composite>;

var x = <Composite>
  <Composite2 />
</Composite>;",
            r#"var x = createVNode(1, "div", null, createComponentVNode(2, Component), 2);
var x = createVNode(1, "div", null, props.children, 0);
var x = createComponentVNode(2, Composite, {
  children: props.children
});
var x = createComponentVNode(2, Composite, {
  children: createComponentVNode(2, Composite2)
});"#,
        );
    }

    #[test]
    fn should_allow_nested_fragments() {
        assert_transform(
            r"<div>
  <  >
    <>
      <span>Hello</span>
      <span>world</span>
    </>
    <>
      <span>Goodbye</span>
      <span>world</span>
    </>
  </>
</div>",
            r#"createVNode(1, "div", null, createFragment([createFragment([createVNode(1, "span", null, "Hello", 16), createVNode(1, "span", null, "world", 16)], 4), createFragment([createVNode(1, "span", null, "Goodbye", 16), createVNode(1, "span", null, "world", 16)], 4)], 4), 2);"#,
        );
    }

    // babel-plugin-inferno keeps the object literal spread; swc-plugin-inferno flattens it into the props,
    // which creates the same props.
    #[test]
    fn comments() {
        assert_transform(
            r#"<div {.../*i18n*/{ id: "hello" }} />;
<Trans /*test1 */a="1"/**test2 */b="2"/**test3 */ />;"#,
            r#"normalizeProps(createVNode(1, "div", null, null, 1, {
    id: "hello"
}));
createComponentVNode(2, Trans, {
    a: "1",
    b: "2"
});"#,
        );
    }
}

/// babel-parser jsx fixtures
mod babel_parser_jsx_fixtures {
    use super::*;

    #[test]
    fn basic_3() {
        assert_transform(
            r#"<a n:foo="bar"> {value} <b><c /></b></a>"#,
            r#"createVNode(1, "a", null, [createTextVNode(" "), value, createTextVNode(" "), createVNode(1, "b", null, createVNode(1, "c"), 2)], 0, {
  "n:foo": "bar"
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno compiles tags that do not start with an uppercase letter as elements"]
    fn basic_6() {
        assert_transform("<日本語></日本語>", "createComponentVNode(2, 日本語);");
    }

    #[test]
    fn basic_11() {
        assert_transform(
            "<div>@test content</div>",
            r#"createVNode(1, "div", null, "@test content", 16);"#,
        );
    }

    #[test]
    fn basic_12() {
        assert_transform(
            "<div><br />7x invalid-js-identifier</div>",
            r#"createVNode(1, "div", null, [createVNode(1, "br"), createTextVNode("7x invalid-js-identifier")], 4);"#,
        );
    }

    #[test]
    fn basic_16() {
        assert_transform("(<div />) < x;", r#"createVNode(1, "div") < x;"#);
    }

    #[test]
    fn basic_asi() {
        assert_transform(
            r"let x
<div />",
            r#"let x;
createVNode(1, "div");"#,
        );
    }

    #[test]
    fn keyword_tag() {
        assert_transform("<var></var>", r#"createVNode(1, "var");"#);
    }

    #[test]
    fn yield_tag() {
        assert_transform(
            "function* g() { yield <a></a>; }",
            r#"function* g() {
  yield createVNode(1, "a");
}"#,
        );
    }

    #[test]
    fn entity() {
        assert_transform(
            "<A>&#x1f4a9;</A>",
            "createComponentVNode(2, A, {\n  children: \"💩\"\n});",
        );
    }

    #[test]
    fn nonentity() {
        assert_transform(
            "<A>&#x1g4q9;</A>",
            r#"createComponentVNode(2, A, {
  children: "&#x1g4q9;"
});"#,
        );
    }

    #[test]
    fn html_entities_code_point() {
        assert_transform(
            "<div>&#1234;&#xABC;&#x10ffff;</div>",
            "createVNode(1, \"div\", null, \"Ӓ\u{ABC}\u{10FFFF}\", 16);",
        );
    }

    #[test]
    fn html_entities_invalid() {
        assert_transform(
            "<div>&amp &ampa; &amp ; &xamp; &#0_0;</div>",
            r#"createVNode(1, "div", null, "&amp &ampa; &amp ; &xamp; &#0_0;", 16);"#,
        );
    }

    #[test]
    fn fragment_5() {
        assert_transform(
            r"<
// comment1
/* comment2 */
><div/></>",
            r#"createFragment([createVNode(1, "div")], 4);"#,
        );
    }

    // babel-plugin-inferno wraps the string in createTextVNode; Inferno normalizes the unknown
    // children to the same vNodes.
    #[test]
    fn fragment_6() {
        assert_transform(
            r#"<><div>JSXElement</div>JSXText{"JSXExpressionContainer"}</>"#,
            r#"createFragment([
    createVNode(1, "div", null, "JSXElement", 16),
    createTextVNode("JSXText"),
    "JSXExpressionContainer"
], 0);"#,
        );
    }

    #[test]
    fn regression_1() {
        assert_transform(
            r#"<p>foo <a href="test"> bar</a> baz</p>"#,
            r#"createVNode(1, "p", null, [createTextVNode("foo "), createVNode(1, "a", null, " bar", 16, {
  "href": "test"
}), createTextVNode(" baz")], 4);"#,
        );
    }

    #[test]
    fn regression_4() {
        assert_transform(
            "<div>/text</div>",
            r#"createVNode(1, "div", null, "/text", 16);"#,
        );
    }

    #[test]
    fn regression_issue_2083() {
        assert_transform(
            "true ? (<div />) : <div />",
            r#"true ? createVNode(1, "div") : createVNode(1, "div");"#,
        );
    }

    #[test]
    fn issue_8891() {
        assert_transform(
            r#"<div prop={{ function: "test" }} />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "prop": {
    function: "test"
  }
});"#,
        );
    }

    #[test]
    fn issue_11387() {
        assert_transform(
            "<div>{(this?.class, this.class, this?.function, this.function)}</div>",
            r#"createVNode(1, "div", null, (this?.class, this.class, this?.function, this.function), 0);"#,
        );
    }

    #[test]
    fn tsx_assignment_in_conditional_expression() {
        assert_transform(
            "a == 3 ? (a = <h1>123</h1>) : (a = <h1>abc</h1>)",
            r#"a == 3 ? a = createVNode(1, "h1", null, "123", 16) : a = createVNode(1, "h1", null, "abc", 16);"#,
        );
    }
}

/// transform-react-constant-elements fixtures
mod transform_react_constant_elements_fixtures {
    use super::*;

    #[test]
    fn magical_bindings_super() {
        assert_transform(
            "class A extends B { m() { return <super.Foo/>; } }",
            r"class A extends B {
  m() {
    return createComponentVNode(2, super.Foo);
  }
}",
        );
    }

    #[test]
    fn magical_bindings_new_target() {
        assert_transform(
            "function f() { return <new.target.Foo/>; }",
            r"function f() {
  return createComponentVNode(2, new.target.Foo);
}",
        );
    }

    #[test]
    fn magical_bindings_arguments() {
        assert_transform(
            "function f() { return <arguments.Foo/>; }",
            r"function f() {
  return createComponentVNode(2, arguments.Foo);
}",
        );
    }

    #[test]
    fn lowercase_member_expression_transform_react_inline_elements() {
        assert_transform(
            "<form.TestComponent />",
            "createComponentVNode(2, form.TestComponent);",
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    // babel-plugin-inferno compiles <this /> to an element named "this".
    #[test]
    fn arrow_functions_compiles_this_to_a_component() {
        assert_transform(
            r"var foo = function () {
  return () => <this />;
};

var bar = function () {
  return () => <this.foo />;
};",
            r"var foo = function() {
    return ()=>createComponentVNode(2, this);
};
var bar = function() {
    return ()=>createComponentVNode(2, this.foo);
};",
        );
    }
}
