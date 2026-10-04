//! Ported from babel-plugin-inferno `tests/babel-parity.test.js`: Babel parity

use crate::helpers::*;

/// transform-react-jsx fixtures
mod transform_react_jsx_fixtures {
    use super::*;

    #[test]
    fn should_convert_simple_tags() {
        assert_transform("var x = <div></div>;", r#"var x = newVNode(17, "div");"#);
    }

    #[test]
    fn should_convert_simple_text() {
        assert_transform(
            "var x = <div>text</div>;",
            r#"var x = newVNode(3, "div", null, "text");"#,
        );
    }

    #[test]
    fn should_allow_js_namespacing() {
        assert_transform(
            "<Namespace.Component />;",
            "newComponentVNode(0, Namespace.Component);",
        );
    }

    #[test]
    fn should_allow_deeper_js_namespacing() {
        assert_transform(
            "<Namespace.DeepNamespace.Component />;",
            "newComponentVNode(0, Namespace.DeepNamespace.Component);",
        );
    }

    #[test]
    fn should_transform_known_hyphenated_tags() {
        assert_transform("<font-face />;", r#"newVNode(80, "font-face");"#);
    }

    #[test]
    fn this_tag_name() {
        assert_transform(
            "var div = <this.foo>test</this.foo>;",
            r#"var div = newComponentVNode(0, this.foo, {
  children: "test"
});"#,
        );
    }

    #[test]
    fn assignment() {
        assert_transform(
            r#"var div = <Component {...props} foo="bar" />"#,
            r#"var div = normalizeProps(newComponentVNode(0, Component, {
  ...props,
  "foo": "bar"
}));"#,
        );
    }

    #[test]
    fn should_allow_elements_as_attributes() {
        assert_transform(
            "<div attr=<div /> />",
            r#"newVNode(17, "div", null, null, {
  "attr": newVNode(17, "div")
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
    return newVNode(1, "div", null, [newTextVNode("Hello "), this.props.name]);
  }
});
React.render(newComponentVNode(0, HelloMessage, {
  "name": newVNode(3, "span", null, "Sebastian")
}), mountNode);"#,
        );
    }

    #[test]
    fn should_not_add_quotes_to_identifier_names() {
        assert_transform(
            "var e = <F aaa new const var default foo-bar/>;",
            r#"var e = newComponentVNode(0, F, {
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
            r#"newVNode(3, "button", null, "Button", {
  "data-value": "a value"
});"#,
        );
    }

    #[test]
    fn should_not_mangle_expressioncontainer_attribute_values() {
        assert_transform(
            r#"<button data-value={"a value\n  with\nnewlines\n   and spaces"}>Button</button>;"#,
            r#"newVNode(3, "button", null, "Button", {
  "data-value": "a value\n  with\nnewlines\n   and spaces"
});"#,
        );
    }

    #[test]
    fn duplicate_props_spread_variants() {
        assert_transform(
            r"<p {...{prop, prop}}></p>;
<p prop {...{prop}}></p>;
<p {...{prop}} prop></p>;",
            r#"normalizeProps(newVNode(17, "p", null, null, {
  ...{
    prop,
    prop
  }
}));
normalizeProps(newVNode(17, "p", null, null, {
  "prop": true,
  ...{
    prop
  }
}));
normalizeProps(newVNode(17, "p", null, null, {
  ...{
    prop
  },
  "prop": true
}));"#,
        );
    }

    #[test]
    fn duplicate_props_repeated_attribute() {
        assert_transform_error(
            "<p prop prop></p>;",
            "Multiple prop props are not supported. Remove the duplicate prop prop.",
        );
    }

    #[test]
    fn flattens_spread() {
        assert_transform(
            r#"<p {...props}>text</p>;
<div {...props}>{contents}</div>;
<img alt="" {...{src, title}} />;
<blockquote {...{cite}}>{items}</blockquote>;"#,
            r#"normalizeProps(newVNode(3, "p", null, "text", {
  ...props
}));
normalizeProps(newVNode(1, "div", null, contents, {
  ...props
}));
normalizeProps(newVNode(17, "img", null, null, {
  "alt": "",
  ...{
    src,
    title
  }
}));
normalizeProps(newVNode(1, "blockquote", null, items, {
  ...{
    cite
  }
}));"#,
        );
    }

    #[test]
    fn handle_spread_with_proto() {
        assert_transform(
            r#"<p {...{__proto__: null}}>text</p>;
<div {...{"__proto__": null}}>{contents}</div>;"#,
            r#"normalizeProps(newVNode(3, "p", null, "text", {
  ...{
    __proto__: null
  }
}));
normalizeProps(newVNode(1, "div", null, contents, {
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
            r#"normalizeProps(newComponentVNode(0, Component, {
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
            r#"normalizeProps(newComponentVNode(0, Component, {
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
            r#"normalizeProps(newComponentVNode(0, Component, {
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
            "newVNode(3, \"div\", null, \"wow\");\nnewVNode(3, \"div\", null, \"wôw\");\nnewVNode(3, \"div\", null, \"w & w\");\nnewVNode(3, \"div\", null, \"w & w\");\nnewVNode(3, \"div\", null, \"w \\xA0 w\");\nnewVNode(3, \"div\", null, \"this should not parse as unicode: \\xA0\");\nnewVNode(3, \"div\", null, \"this should parse as nbsp: \\xA0 \");\nnewVNode(1, \"div\", null, [newTextVNode(\"this should parse as unicode: \"), newTextVNode('\u{A0} ')]);\nnewVNode(3, \"div\", null, \"w < w\");",
        );
    }

    #[test]
    fn should_not_strip_nbsp_even_coupled_with_other_whitespace() {
        assert_transform(
            "<div>&nbsp; </div>;",
            r#"newVNode(3, "div", null, "\xA0 ");"#,
        );
    }

    #[test]
    fn should_not_strip_tags_with_a_single_child_of_nbsp() {
        assert_transform("<div>&nbsp;</div>;", r#"newVNode(3, "div", null, "\xA0");"#);
    }

    #[test]
    fn weird_symbols() {
        assert_transform(
            r"class MobileHomeActivityTaskPriorityIcon extends React.PureComponent {
  render() {
    return <Text>&nbsp;{this.props.value}&nbsp;</Text>;
  }
}",
            r#"class MobileHomeActivityTaskPriorityIcon extends React.PureComponent {
  render() {
    return newComponentVNode(0, Text, {
      children: ["\xA0", this.props.value, "\xA0"]
    });
  }
}"#,
        );
    }

    #[test]
    fn dont_coerce_expression_containers() {
        assert_transform(
            r#"<Text>
  To get started, edit index.ios.js!!!{"\n"}
  Press Cmd+R to reload
</Text>"#,
            r#"newComponentVNode(0, Text, {
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
            r#"var x = newVNode(1, "div", null, [newTextVNode("foo"), newTextVNode("bar"), newTextVNode("baz"), newVNode(3, "div", null, "buz bang"), newTextVNode("qux"), null, newTextVNode("quack")]);"#,
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
            r#"var x = newVNode(17, "div", null, null, {
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
            r#"var x = newVNode(5, "div", null, [newVNode(9, "div", null, newVNode(17, "br")), newComponentVNode(0, Component, {
  children: [foo, newVNode(17, "br"), bar]
}), newVNode(17, "br")]);"#,
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
            r#"var x = newVNode(9, "div", null, newComponentVNode(0, Component));
var x = newVNode(1, "div", null, props.children);
var x = newComponentVNode(0, Composite, {
  children: props.children
});
var x = newComponentVNode(0, Composite, {
  children: newComponentVNode(0, Composite2)
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
            r#"newVNode(9, "div", null, newFragment(260, [newFragment(260, [newVNode(3, "span", null, "Hello"), newVNode(3, "span", null, "world")]), newFragment(260, [newVNode(3, "span", null, "Goodbye"), newVNode(3, "span", null, "world")])]));"#,
        );
    }

    #[test]
    fn comments() {
        assert_transform(
            r#"<div {.../*i18n*/{ id: "hello" }} />;
<Trans /*test1 */a="1"/**test2 */b="2"/**test3 */ />;"#,
            r#"normalizeProps(newVNode(17, "div", null, null, {
  ... /*i18n*/{
    id: "hello"
  }
}));
newComponentVNode(0, Trans, {
  "a": "1",
  "b": "2"
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
            r#"newVNode(1, "a", null, [newTextVNode(" "), value, newTextVNode(" "), newVNode(9, "b", null, newVNode(17, "c"))], {
  "n:foo": "bar"
});"#,
        );
    }

    #[test]
    fn basic_6() {
        assert_transform("<日本語></日本語>", "newComponentVNode(0, 日本語);");
    }

    #[test]
    fn basic_11() {
        assert_transform(
            "<div>@test content</div>",
            r#"newVNode(3, "div", null, "@test content");"#,
        );
    }

    #[test]
    fn basic_12() {
        assert_transform(
            "<div><br />7x invalid-js-identifier</div>",
            r#"newVNode(5, "div", null, [newVNode(17, "br"), newTextVNode("7x invalid-js-identifier")]);"#,
        );
    }

    #[test]
    fn basic_16() {
        assert_transform("(<div />) < x;", r#"newVNode(17, "div") < x;"#);
    }

    #[test]
    fn basic_asi() {
        assert_transform(
            r"let x
<div />",
            r#"let x;
newVNode(17, "div");"#,
        );
    }

    #[test]
    fn keyword_tag() {
        assert_transform("<var></var>", r#"newVNode(17, "var");"#);
    }

    #[test]
    fn yield_tag() {
        assert_transform(
            "function* g() { yield <a></a>; }",
            r#"function* g() {
  yield newVNode(17, "a");
}"#,
        );
    }

    #[test]
    fn entity() {
        assert_transform(
            "<A>&#x1f4a9;</A>",
            "newComponentVNode(0, A, {\n  children: \"💩\"\n});",
        );
    }

    #[test]
    fn nonentity() {
        assert_transform(
            "<A>&#x1g4q9;</A>",
            r#"newComponentVNode(0, A, {
  children: "&#x1g4q9;"
});"#,
        );
    }

    #[test]
    fn html_entities_code_point() {
        assert_transform(
            "<div>&#1234;&#xABC;&#x10ffff;</div>",
            "newVNode(3, \"div\", null, \"Ӓ\u{ABC}\u{10FFFF}\");",
        );
    }

    #[test]
    fn html_entities_invalid() {
        assert_transform(
            "<div>&amp &ampa; &amp ; &xamp; &#0_0;</div>",
            r#"newVNode(3, "div", null, "&amp &ampa; &amp ; &xamp; &#0_0;");"#,
        );
    }

    #[test]
    fn fragment_5() {
        assert_transform(
            r"<
// comment1
/* comment2 */
><div/></>",
            r#"newFragment(260, [newVNode(17, "div")]);"#,
        );
    }

    #[test]
    fn fragment_6() {
        assert_transform(
            r#"<><div>JSXElement</div>JSXText{"JSXExpressionContainer"}</>"#,
            r#"newFragment(256, [newVNode(3, "div", null, "JSXElement"), newTextVNode("JSXText"), newTextVNode("JSXExpressionContainer")]);"#,
        );
    }

    #[test]
    fn regression_1() {
        assert_transform(
            r#"<p>foo <a href="test"> bar</a> baz</p>"#,
            r#"newVNode(5, "p", null, [newTextVNode("foo "), newVNode(3, "a", null, " bar", {
  "href": "test"
}), newTextVNode(" baz")]);"#,
        );
    }

    #[test]
    fn regression_4() {
        assert_transform("<div>/text</div>", r#"newVNode(3, "div", null, "/text");"#);
    }

    #[test]
    fn regression_issue_2083() {
        assert_transform(
            "true ? (<div />) : <div />",
            r#"true ? newVNode(17, "div") : newVNode(17, "div");"#,
        );
    }

    #[test]
    fn issue_8891() {
        assert_transform(
            r#"<div prop={{ function: "test" }} />"#,
            r#"newVNode(17, "div", null, null, {
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
            r#"newVNode(1, "div", null, (this?.class, this.class, this?.function, this.function));"#,
        );
    }

    #[test]
    fn tsx_assignment_in_conditional_expression() {
        assert_transform(
            "a == 3 ? (a = <h1>123</h1>) : (a = <h1>abc</h1>)",
            r#"a == 3 ? a = newVNode(3, "h1", null, "123") : a = newVNode(3, "h1", null, "abc");"#,
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
    return newComponentVNode(0, super.Foo);
  }
}",
        );
    }

    #[test]
    fn magical_bindings_new_target() {
        assert_transform(
            "function f() { return <new.target.Foo/>; }",
            r"function f() {
  return newComponentVNode(0, new.target.Foo);
}",
        );
    }

    #[test]
    fn magical_bindings_arguments() {
        assert_transform(
            "function f() { return <arguments.Foo/>; }",
            r"function f() {
  return newComponentVNode(0, arguments.Foo);
}",
        );
    }

    #[test]
    fn lowercase_member_expression_transform_react_inline_elements() {
        assert_transform(
            "<form.TestComponent />",
            "newComponentVNode(0, form.TestComponent);",
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    #[test]
    fn arrow_functions_compiles_this_to_an_element() {
        assert_transform(
            r"var foo = function () {
  return () => <this />;
};

var bar = function () {
  return () => <this.foo />;
};",
            r#"var foo = function () {
  return () => newVNode(17, "this");
};
var bar = function () {
  return () => newComponentVNode(0, this.foo);
};"#,
        );
    }
}
