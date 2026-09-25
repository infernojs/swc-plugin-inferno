//! Ported from babel-plugin-inferno `tests/attributes.test.js`: Attributes

use crate::helpers::*;

/// verbatim attributes
mod verbatim_attributes {
    use super::*;

    #[test]
    fn should_keep_data_and_aria_attributes() {
        assert_transform(
            r#"<div data-foo="1" aria-label="x" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "data-foo": "1",
  "aria-label": "x"
});"#,
        );
    }

    #[test]
    fn should_keep_multi_hyphen_data_attributes() {
        assert_transform(
            "<div data-foo-bar={x} />",
            r#"createVNode(1, "div", null, null, 1, {
  "data-foo-bar": x
});"#,
        );
    }

    #[test]
    fn should_keep_the_casing_of_data_attributes() {
        assert_transform(
            r#"<div data-fooBar="true" aria="hello" on="tap:x" oncustomevent={f} />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "data-fooBar": "true",
  "aria": "hello",
  "on": "tap:x",
  "oncustomevent": f
});"#,
        );
    }

    #[test]
    fn should_keep_namespaced_attributes() {
        assert_transform(
            r#"<div xml:lang="en" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "xml:lang": "en"
});"#,
        );
    }

    #[test]
    fn should_keep_namespaced_attributes_with_hyphens() {
        assert_transform(
            r#"<div foo:bar-baz="1" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "foo:bar-baz": "1"
});"#,
        );
    }

    #[test]
    fn should_keep_xmlns_xlink_on_svg() {
        assert_transform(
            r#"<svg viewBox="0 0 10 10" xmlns:xlink="http://www.w3.org/1999/xlink"><g><path d="M0"/></g></svg>"#,
            r#"createVNode(32, "svg", null, createVNode(32, "g", null, createVNode(32, "path", null, null, 1, {
  "d": "M0"
}), 2), 2, {
  "viewBox": "0 0 10 10",
  "xmlns:xlink": "http://www.w3.org/1999/xlink"
});"#,
        );
    }

    #[test]
    fn should_keep_an_uppercase_children_attribute_as_a_prop() {
        assert_transform(
            r#"<div CHILDREN="5" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "CHILDREN": "5"
});"#,
        );
    }

    #[test]
    fn should_keep_the_is_attribute_next_to_mapped_attributes() {
        assert_transform(
            r#"<div is="custom-element" htmlFor="x" className="y" />"#,
            r#"createVNode(1, "div", "y", null, 1, {
  "is": "custom-element",
  "for": "x"
});"#,
        );
    }
}

/// reserved words and hyphens as names
mod reserved_words_and_hyphens_as_names {
    use super::*;

    #[test]
    fn should_quote_reserved_words_and_hyphenated_names_on_components() {
        assert_transform(
            "<F aaa new const var default foo-bar/>",
            r#"createComponentVNode(2, F, {
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
    fn should_quote_reserved_words_on_elements() {
        assert_transform(
            r#"<div new const="1" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "new": true,
  "const": "1"
});"#,
        );
    }
}

/// values
mod values {
    use super::*;

    #[test]
    fn should_compile_valueless_attributes_to_true() {
        assert_transform(
            r#"<input value={1} checked={c} defaultValue="x" defaultChecked />"#,
            r#"createVNode(64, "input", null, null, 1, {
  "value": 1,
  "checked": c,
  "defaultValue": "x",
  "defaultChecked": true
});"#,
        );
    }

    #[test]
    fn should_keep_a_style_object() {
        assert_transform(
            r#"<div style={{color: "red"}} />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "style": {
    color: "red"
  }
});"#,
        );
    }

    #[test]
    fn should_keep_a_style_string() {
        assert_transform(
            r#"<div style="color: red" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "style": "color: red"
});"#,
        );
    }

    #[test]
    fn should_keep_custom_properties_in_a_style_object() {
        assert_transform(
            r#"<div style={{"--foo": 5}} />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "style": {
    "--foo": 5
  }
});"#,
        );
    }

    #[test]
    fn should_keep_dangerously_set_inner_html() {
        assert_transform(
            "<div dangerouslySetInnerHTML={{__html: x}} />",
            r#"createVNode(1, "div", null, null, 1, {
  "dangerouslySetInnerHTML": {
    __html: x
  }
});"#,
        );
    }

    #[test]
    fn should_emit_both_dangerously_set_inner_html_and_children() {
        assert_transform(
            r#"<div dangerouslySetInnerHTML={{__html: "abcdef"}}>ghjkl</div>"#,
            r#"createVNode(1, "div", null, "ghjkl", 16, {
  "dangerouslySetInnerHTML": {
    __html: "abcdef"
  }
});"#,
        );
    }

    #[test]
    fn should_keep_dangerously_set_inner_html_on_a_void_element() {
        assert_transform(
            r#"<input dangerouslySetInnerHTML={{__html: "content"}} />"#,
            r#"createVNode(64, "input", null, null, 1, {
  "dangerouslySetInnerHTML": {
    __html: "content"
  }
});"#,
        );
    }
}

/// className and class
mod class_name_and_class {
    use super::*;

    #[test]
    fn should_pass_an_empty_class_name() {
        assert_transform(r#"<div className="" />"#, r#"createVNode(1, "div", "");"#);
    }

    #[test]
    fn should_pass_an_undefined_class_name() {
        assert_transform(
            "<div className={undefined} />",
            r#"createVNode(1, "div", undefined);"#,
        );
    }

    #[test]
    fn should_omit_a_null_class_name() {
        assert_transform(
            "<div className={null} />",
            r#"createVNode(1, "div", null);"#,
        );
    }

    #[test]
    fn should_keep_class_name_and_class_as_props_on_components() {
        assert_transform(
            r#"<Foo className="x" class="y" />"#,
            r#"createComponentVNode(2, Foo, {
  "className": "x",
  "class": "y"
});"#,
        );
    }

    #[test]
    fn should_use_class_on_svg_elements() {
        assert_transform(
            r#"<svg class="a"><g className="b"/></svg>"#,
            r#"createVNode(32, "svg", "a", createVNode(32, "g", "b"), 2);"#,
        );
    }
}

/// duplicate attributes
mod duplicate_attributes {
    use super::*;

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_key_props() {
        assert_transform_error(
            r#"<div key="a" key={b()} />"#,
            "Multiple key props are not supported. Remove the duplicate key prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_props_on_elements() {
        assert_transform_error(
            "<p prop prop />",
            "Multiple prop props are not supported. Remove the duplicate prop prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_props_on_components() {
        assert_transform_error(
            r#"<Foo title="a" id="x" title="b" />"#,
            "Multiple title props are not supported. Remove the duplicate title prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_on_component_hooks() {
        assert_transform_error(
            "<Foo onComponentDidMount={a} onComponentDidMount={b} />",
            "Multiple onComponentDidMount props are not supported. Remove the duplicate onComponentDidMount prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_special_flags() {
        assert_transform_error(
            "<div $HasKeyedChildren $HasKeyedChildren>{a}</div>",
            "Multiple $HasKeyedChildren props are not supported. Remove the duplicate $HasKeyedChildren prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_point_the_duplicate_prop_error_at_the_duplicate() {
        assert_transform_error(
            r#"<Foo title="a" id="x" title="b" />"#,
            r#"> 1 | <Foo title="a" id="x" title="b" />
    |                       ^^^^^^^^^"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_html_for_together_with_for_on_elements() {
        assert_transform_error(
            r#"<label htmlFor="a" for="b" />"#,
            "htmlFor and for both set the for prop. Remove one of them.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_a_lowercased_attribute_together_with_its_camel_case_name() {
        assert_transform_error(
            r#"<div tabIndex="1" tabindex="2" />"#,
            "tabIndex and tabindex both set the tabindex prop. Remove one of them.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_an_svg_attribute_together_with_its_camel_case_name() {
        assert_transform_error(
            r#"<rect strokeWidth="1" stroke-width="2" />"#,
            "strokeWidth and stroke-width both set the stroke-width prop. Remove one of them.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_a_namespaced_attribute_together_with_its_camel_case_name() {
        assert_transform_error(
            r##"<use xlinkHref="#a" xlink:href="#b" />"##,
            "xlinkHref and xlink:href both set the xlink:href prop. Remove one of them.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_point_the_mapped_attribute_error_at_the_second_attribute() {
        assert_transform_error(
            r#"<label
  htmlFor="a"
  for="b"
/>"#,
            r#"> 3 |   for="b"
    |   ^^^^^^^"#,
        );
    }

    #[test]
    fn should_allow_html_for_together_with_for_on_components() {
        assert_transform(
            r#"<Foo htmlFor="a" for="b" />"#,
            r#"createComponentVNode(2, Foo, {
  "htmlFor": "a",
  "for": "b"
});"#,
        );
    }

    // babel-plugin-inferno keeps the object literal spread; swc-plugin-inferno flattens it into the props,
    // which creates the same props.
    #[test]
    fn should_allow_a_prop_next_to_a_spread_containing_the_same_prop() {
        assert_transform(
            "<p {...{prop}} prop />",
            r#"normalizeProps(createVNode(1, "p", null, null, 1, {
    prop: prop,
    prop: true
}));"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno drops a children prop replaced by JSX children without evaluating it"]
    fn should_evaluate_a_component_children_prop_replaced_by_jsx_children() {
        assert_transform(
            "<Foo children={f()}>2</Foo>",
            r#"createComponentVNode(2, Foo, {
  children: (f(), "2")
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno drops a children prop replaced by JSX children without evaluating it"]
    fn should_evaluate_an_element_children_prop_replaced_by_jsx_children() {
        assert_transform(
            "<div children={f()}>x</div>",
            r#"createVNode(1, "div", null, (f(), "x"), 16);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno drops a children prop replaced by JSX children without evaluating it"]
    fn should_evaluate_a_children_prop_replaced_by_several_jsx_children() {
        assert_transform(
            "<div children={f()}><a/><b/></div>",
            r#"createVNode(1, "div", null, (f(), [createVNode(1, "a"), createVNode(1, "b")]), 4);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_children_props_on_components() {
        assert_transform_error(
            "<Foo children={1} children={4}>2</Foo>",
            "Multiple children props are not supported. Remove the duplicate children prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_children_props_on_elements() {
        assert_transform_error(
            "<div children={a()} children={b()} />",
            "Multiple children props are not supported. Remove the duplicate children prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_point_the_duplicate_children_prop_error_at_the_duplicate() {
        assert_transform_error(
            r#"<div
  id="x"
  children={a()}
  children={b()}
/>"#,
            r"> 4 |   children={b()}
    |   ^^^^^^^^^^^^^^",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_ref_props_on_elements() {
        assert_transform_error(
            "<div ref={a} ref={b} />",
            "Multiple ref props are not supported. Remove the duplicate ref prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_ref_props_on_components() {
        assert_transform_error(
            "<Foo ref={a} onComponentDidMount={m} ref={b} />",
            "Multiple ref props are not supported. Remove the duplicate ref prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_point_the_duplicate_ref_prop_error_at_the_duplicate() {
        assert_transform_error(
            "<div ref={a} ref={b} />",
            r"> 1 | <div ref={a} ref={b} />
    |              ^^^^^^^",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_class_name_props_on_elements() {
        assert_transform_error(
            r#"<div className="a" className={b} />"#,
            "Multiple className props are not supported. Remove the duplicate className prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_class_props_on_elements() {
        assert_transform_error(
            r#"<div class="a" class={b} />"#,
            "Multiple class props are not supported. Remove the duplicate class prop.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_class_name_together_with_class_on_elements() {
        assert_transform_error(
            "<div className={a} class={b} />",
            "className and class both set the class name. Remove one of them.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_class_together_with_class_name_on_elements() {
        assert_transform_error(
            r#"<div class="a" className="b" />"#,
            "className and class both set the class name. Remove one of them.",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_point_the_class_name_and_class_error_at_the_second_one() {
        assert_transform_error(
            r"<div
  className={a}
  class={b}
/>",
            r"> 3 |   class={b}
    |   ^^^^^^^^^",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not reject duplicate props"]
    fn should_reject_duplicate_class_name_props_on_components() {
        assert_transform_error(
            r#"<Foo className="a" className="b" />"#,
            "Multiple className props are not supported. Remove the duplicate className prop.",
        );
    }

    #[test]
    fn should_drop_replaced_values_without_side_effects() {
        assert_transform(
            r#"<div a children={["a", {b: 1}, () => x, -1]}>c</div>"#,
            r#"createVNode(1, "div", null, "c", 16, {
  "a": true
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno drops a children prop replaced by JSX children without evaluating it"]
    fn should_keep_replaced_values_that_may_have_side_effects() {
        assert_transform(
            "<div children={[...a]}>c</div>",
            r#"createVNode(1, "div", null, ([...a], "c"), 16);"#,
        );
    }
}

/// JSX as attribute values
mod jsx_as_attribute_values {
    use super::*;

    #[test]
    fn should_compile_an_element_attribute_value_without_braces_on_an_element() {
        assert_transform(
            "<div attr=<span/> />",
            r#"createVNode(1, "div", null, null, 1, {
  "attr": createVNode(1, "span")
});"#,
        );
    }

    #[test]
    fn should_compile_an_element_attribute_value_without_braces_on_a_component() {
        assert_transform(
            "<Foo attr=<span/> />",
            r#"createComponentVNode(2, Foo, {
  "attr": createVNode(1, "span")
});"#,
        );
    }

    #[test]
    fn should_compile_a_fragment_attribute_value() {
        assert_transform(
            "<Foo value={<>{a}</>} />",
            r#"createComponentVNode(2, Foo, {
  "value": createFragment(a, 0)
});"#,
        );
    }

    #[test]
    fn should_compile_jsx_in_a_conditional_attribute_value() {
        assert_transform(
            "<a b={x ? <c /> : <d />} />",
            r#"createVNode(1, "a", null, null, 1, {
  "b": x ? createVNode(1, "c") : createVNode(1, "d")
});"#,
        );
    }

    #[test]
    fn should_compile_render_props_and_element_props() {
        assert_transform(
            "<Foo render={() => <div>{x}</div>} icon={<Icon/>} />",
            r#"createComponentVNode(2, Foo, {
  "render": () => createVNode(1, "div", null, x, 0),
  "icon": createComponentVNode(2, Icon)
});"#,
        );
    }
}

/// attribute layout
mod attribute_layout {
    use super::*;

    #[test]
    fn should_keep_multi_line_expression_attributes_with_comments() {
        assert_transform(
            r#"<div attr2={
  "foo" + "bar" +

  "baz" + "bug"
  // Extra line here.
} />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "attr2": "foo" + "bar" + "baz" + "bug"
  // Extra line here.
});"#,
        );
    }

    #[test]
    fn should_allow_spaces_around() {
        assert_transform(
            r#"<Trans b = "2" />"#,
            r#"createComponentVNode(2, Trans, {
  "b": "2"
});"#,
        );
    }

    #[test]
    fn should_allow_a_line_break_before() {
        assert_transform(
            r"<Foo y
={2 } z />",
            r#"createComponentVNode(2, Foo, {
  "y": 2,
  "z": true
});"#,
        );
    }
}

/// mapping tables only apply to elements
mod mapping_tables_only_apply_to_elements {
    use super::*;

    #[test]
    fn should_not_map_html_for_accept_charset_or_col_span_on_components() {
        assert_transform(
            r#"<Foo htmlFor="x" acceptCharset="y" colSpan={2} />"#,
            r#"createComponentVNode(2, Foo, {
  "htmlFor": "x",
  "acceptCharset": "y",
  "colSpan": 2
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno maps onDoubleClick on components too"]
    fn should_not_map_on_double_click_on_components() {
        assert_transform(
            "<Foo onDoubleClick={f} />",
            r#"createComponentVNode(2, Foo, {
  "onDoubleClick": f
});"#,
        );
    }
}

/// mapped attributes
mod mapped_attributes {
    use super::*;

    #[test]
    fn should_map_http_equiv_and_char_set() {
        assert_transform(
            r#"<meta httpEquiv="refresh" charSet="utf-8" />"#,
            r#"createVNode(1, "meta", null, null, 1, {
  "http-equiv": "refresh",
  "charset": "utf-8"
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno does not map this SVG attribute"]
    fn should_map_text_anchor_on_svg_text() {
        assert_transform(
            r#"<svg><text textAnchor="middle" /></svg>"#,
            r#"createVNode(32, "svg", null, createVNode(32, "text", null, null, 1, {
  "text-anchor": "middle"
}), 2);"#,
        );
    }

    #[test]
    fn should_map_transform_origin() {
        assert_transform(
            r#"<div transformOrigin="0 0" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "transform-origin": "0 0"
});"#,
        );
    }

    #[test]
    fn should_lowercase_tab_index_read_only_and_max_length() {
        assert_transform(
            r#"<div tabIndex="1" readOnly maxLength={3} />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "tabindex": "1",
  "readonly": true,
  "maxlength": 3
});"#,
        );
    }

    #[test]
    fn should_map_on_double_click_and_keep_ondblclick() {
        assert_transform(
            "<div onDoubleClick={f} ondblclick={g} />",
            r#"createVNode(1, "div", null, null, 1, {
  "onDblClick": f,
  "ondblclick": g
});"#,
        );
    }

    #[test]
    fn should_map_accent_height_on_font_face() {
        assert_transform(
            "<font-face accentHeight={10} />",
            r#"createVNode(32, "font-face", null, null, 1, {
  "accent-height": 10
});"#,
        );
    }
}

/// event names
mod event_names {
    use super::*;

    #[test]
    fn should_keep_capture_event_names() {
        assert_transform(
            "<div onClickCapture={f} onGotPointerCaptureCapture={g} onTouchMoveCapture={h} />",
            r#"createVNode(1, "div", null, null, 1, {
  "onClickCapture": f,
  "onGotPointerCaptureCapture": g,
  "onTouchMoveCapture": h
});"#,
        );
    }

    #[test]
    fn should_keep_lowercase_and_custom_event_names() {
        assert_transform(
            "<div onclick={f} onanimationend={g} onOtherClick={h} />",
            r#"createVNode(1, "div", null, null, 1, {
  "onclick": f,
  "onanimationend": g,
  "onOtherClick": h
});"#,
        );
    }

    #[test]
    fn should_keep_newer_event_names() {
        assert_transform(
            "<div onScrollEnd={a} onBeforeToggle={b} onCommand={c} onFormData={d} onAuxClick={e} />",
            r#"createVNode(1, "div", null, null, 1, {
  "onScrollEnd": a,
  "onBeforeToggle": b,
  "onCommand": c,
  "onFormData": d,
  "onAuxClick": e
});"#,
        );
    }

    #[test]
    fn should_keep_on_change_and_on_input_together() {
        assert_transform(
            "<input onChange={f} onInput={g} />",
            r#"createVNode(64, "input", null, null, 1, {
  "onChange": f,
  "onInput": g
});"#,
        );
    }

    #[test]
    fn should_keep_focus_events_and_false_handlers() {
        assert_transform(
            "<div onClick={false} onFocusIn={h} onFocusOut={i} />",
            r#"createVNode(1, "div", null, null, 1, {
  "onClick": false,
  "onFocusIn": h,
  "onFocusOut": i
});"#,
        );
    }

    #[test]
    fn should_keep_a_string_event_handler_on_an_element() {
        assert_transform(
            r#"<div onclick="a" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "onclick": "a"
});"#,
        );
    }
}

/// __proto__ prop
mod proto_prop {
    use super::*;

    // Not ported:
    // - Should give the component an own __proto__ prop (evaluates the generated code; should_emit_proto_as_a_computed_key_on_components checks the same key)

    #[test]
    #[ignore = "swc-plugin-inferno emits __proto__ as a plain key, which sets the prototype of the props"]
    fn should_emit_proto_as_a_computed_key_on_components() {
        assert_transform(
            "<Foo __proto__={x} />",
            r#"createComponentVNode(2, Foo, {
  ["__proto__"]: x
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno emits __proto__ as a plain key, which sets the prototype of the props"]
    fn should_emit_proto_as_a_computed_key_on_elements() {
        assert_transform(
            "<div __proto__={x} />",
            r#"createVNode(1, "div", null, null, 1, {
  ["__proto__"]: x
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno emits __proto__ as a plain key, which sets the prototype of the props"]
    fn should_keep_proto_next_to_other_props_babel_proto_in_jsx_attribute() {
        assert_transform(
            r#"<p __proto__={null} class="bar" />"#,
            r#"createVNode(1, "p", "bar", null, 1, {
  ["__proto__"]: null
});"#,
        );
    }
}

/// Object.prototype names as attributes
mod object_prototype_names_as_attributes {
    use super::*;

    #[test]
    fn should_pass_constructor_as_a_prop() {
        assert_transform(
            r#"<div constructor="foo" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "constructor": "foo"
});"#,
        );
    }

    #[test]
    fn should_pass_to_string_and_has_own_property_as_props() {
        assert_transform(
            r#"<div toString="x" hasOwnProperty="y" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "toString": "x",
  "hasOwnProperty": "y"
});"#,
        );
    }

    #[test]
    fn should_pass_value_of_as_a_prop() {
        assert_transform(
            "<div valueOf={v} />",
            r#"createVNode(1, "div", null, null, 1, {
  "valueOf": v
});"#,
        );
    }

    #[test]
    fn should_pass_is_prototype_of_and_property_is_enumerable_as_props() {
        assert_transform(
            "<div isPrototypeOf={v} propertyIsEnumerable={w} />",
            r#"createVNode(1, "div", null, null, 1, {
  "isPrototypeOf": v,
  "propertyIsEnumerable": w
});"#,
        );
    }

    #[test]
    fn should_pass_constructor_as_a_prop_on_svg_elements() {
        assert_transform(
            r#"<rect constructor="x" />"#,
            r#"createVNode(32, "rect", null, null, 1, {
  "constructor": "x"
});"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    // babel-plugin-inferno passes true as className.
    #[test]
    fn should_drop_a_valueless_class_name() {
        assert_transform("<div className />", r#"createVNode(1, "div");"#);
    }

    #[test]
    fn should_drop_comments_between_attributes() {
        assert_transform(
            r#"<div
  /* a multi-line
     comment */
  attr1="foo">
  <span // a double-slash comment
    attr2="bar"
  />
</div>"#,
            r#"createVNode(1, "div", null, createVNode(1, "span", null, null, 1, {
  "attr2": "bar"
}), 2, {
  "attr1": "foo"
});"#,
        );
    }
}
