//! Ported from babel-plugin-inferno `tests/preact-parity.test.js`: Preact parity

use crate::helpers::*;

/// children and text
mod children_and_text {
    use super::*;

    #[test]
    fn should_not_merge_text_with_a_string_literal_child_hydrate_test() {
        assert_transform(
            r#"<p>hello {"foo"}</p>"#,
            r#"createVNode(1, "p", null, [createTextVNode("hello "), createTextVNode("foo")], 0);"#,
        );
    }

    #[test]
    fn should_keep_text_between_falsy_expression_children_render_test() {
        assert_transform(
            "<div>{null},{undefined},{false},{0},{NaN}</div>",
            r#"createVNode(1, "div", null, [null, createTextVNode(","), undefined, createTextVNode(","), false, createTextVNode(","), 0, createTextVNode(","), NaN], 0);"#,
        );
    }

    #[test]
    fn should_keep_text_on_the_first_and_last_lines_around_elements_render_test() {
        assert_transform(
            r"<div>0<span />
<input />
<div />1</div>",
            r#"createVNode(1, "div", null, [createTextVNode("0"), createVNode(1, "span"), createVNode(64, "input"), createVNode(1, "div"), createTextVNode("1")], 4);"#,
        );
    }

    #[test]
    fn should_keep_the_trailing_space_before_an_element_render_test() {
        assert_transform(
            "<div>hello <span>world</span></div>",
            r#"createVNode(1, "div", null, [createTextVNode("hello "), createVNode(1, "span", null, "world", 16)], 4);"#,
        );
    }

    #[test]
    fn should_keep_quotes_and_in_text_render_test() {
        assert_transform(
            r##"<a href="#">href="#"</a>"##,
            r##"createVNode(1, "a", null, "href=\"#\"", 16, {
  "href": "#"
});"##,
        );
    }

    #[test]
    fn should_split_text_and_an_expression_without_space_render_test() {
        assert_transform(
            r#"<h1 class="fade-down">Hi{name}</h1>"#,
            r#"createVNode(1, "h1", "fade-down", [createTextVNode("Hi"), name], 0);"#,
        );
    }

    #[test]
    fn should_pass_an_array_children_prop_to_a_component_create_element_test() {
        assert_transform(
            r#"<Foo a="b" children={[<span class="bar">bar</span>, "123", 456]} />"#,
            r#"createComponentVNode(2, Foo, {
  "a": "b",
  "children": [createVNode(1, "span", "bar", "bar", 16), "123", 456]
});"#,
        );
    }

    #[test]
    fn should_prefer_jsx_children_over_an_array_children_prop_render_test() {
        assert_transform(
            r#"<div a children={["a", "b"]}>c</div>"#,
            r#"createVNode(1, "div", null, "c", 16, {
  "a": true
});"#,
        );
    }

    #[test]
    fn should_compile_a_function_child_of_a_lowercase_consumer_create_context_test() {
        assert_transform(
            "<context.Consumer>{v => <p>{v.state}</p>}</context.Consumer>",
            r#"createComponentVNode(2, context.Consumer, {
  children: v => createVNode(1, "p", null, v.state, 0)
});"#,
        );
    }
}

/// spread (compat tests)
mod spread_compat_tests {
    use super::*;

    #[test]
    fn should_keep_several_spreads() {
        assert_transform(
            "<Inner {...data} {...childData} />",
            r"normalizeProps(createComponentVNode(2, Inner, {
  ...data,
  ...childData
}));",
        );
    }

    #[test]
    fn should_keep_class_before_a_spread() {
        assert_transform(
            r#"<ul class="old" {...props} />"#,
            r#"normalizeProps(createVNode(1, "ul", "old", null, 1, {
  ...props
}));"#,
        );
    }

    #[test]
    fn should_keep_a_template_literal_class_name_before_a_spread() {
        assert_transform(
            "<div className={`${className} foo`} {...props} />",
            r#"normalizeProps(createVNode(1, "div", `${className} foo`, null, 1, {
  ...props
}));"#,
        );
    }

    #[test]
    fn should_keep_a_key_after_a_spread_of_an_object_with_a_nested_spread() {
        assert_transform(
            "<ListItem {...{ isSelected, setSelected, ...item }} key={item.name} />",
            r"normalizeProps(createComponentVNode(2, ListItem, {
  ...{
    isSelected,
    setSelected,
    ...item
  }
}, item.name));",
        );
    }

    #[test]
    fn should_keep_a_conditional_class_expression_on_svg() {
        assert_transform(
            r#"<svg class={c && "bar_" + c} />"#,
            r#"createVNode(32, "svg", c && "bar_" + c);"#,
        );
    }
}

/// attribute values
mod attribute_values {
    use super::*;

    #[test]
    fn should_pass_falsy_attribute_values_verbatim() {
        assert_transform(
            "<div a0={0} anull={null} anan={NaN} afalse={false} />",
            r#"createVNode(1, "div", null, null, 1, {
  "a0": 0,
  "anull": null,
  "anan": NaN,
  "afalse": false
});"#,
        );
    }

    #[test]
    fn should_pass_boolean_like_attributes_verbatim() {
        assert_transform(
            "<a download popover translate={false} aria-checked={false} data-checked={false} />",
            r#"createVNode(1, "a", null, null, 1, {
  "download": true,
  "popover": true,
  "translate": false,
  "aria-checked": false,
  "data-checked": false
});"#,
        );
    }

    #[test]
    fn should_pass_a_style_string() {
        assert_transform(
            r#"<div style="top: 5px; position: relative;" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "style": "top: 5px; position: relative;"
});"#,
        );
    }

    #[test]
    fn should_pass_a_style_object_with_mixed_key_styles() {
        assert_transform(
            r#"<div style={{gridRowStart: 1, opacity: 0, "--fooBar": 1, "background-size": "cover"}} />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "style": {
    gridRowStart: 1,
    opacity: 0,
    "--fooBar": 1,
    "background-size": "cover"
  }
});"#,
        );
    }

    #[test]
    fn should_pass_a_false_table_border() {
        assert_transform(
            "<table border={false} />",
            r#"createVNode(1, "table", null, null, 1, {
  "border": false
});"#,
        );
    }

    #[test]
    fn should_lowercase_row_span_and_col_span() {
        assert_transform(
            "<td rowSpan={2} colSpan={2} />",
            r#"createVNode(1, "td", null, null, 1, {
  "rowspan": 2,
  "colspan": 2
});"#,
        );
    }

    #[test]
    fn should_lowercase_null_max_length_and_min_length() {
        assert_transform(
            "<input maxLength={null} minLength={null} />",
            r#"createVNode(64, "input", null, null, 1, {
  "maxlength": null,
  "minlength": null
});"#,
        );
    }
}

/// form controls
mod form_controls {
    use super::*;

    #[test]
    fn should_compile_a_multiple_select_with_an_array_value() {
        assert_transform(
            r#"<select multiple value={["B", "C"]}><option selected value="B">B</option></select>"#,
            r#"createVNode(256, "select", null, createVNode(1, "option", null, "B", 16, {
  "selected": true,
  "value": "B"
}), 2, {
  "multiple": true,
  "value": ["B", "C"]
});"#,
        );
    }

    #[test]
    fn should_compile_a_select_with_default_value() {
        assert_transform(
            r#"<select defaultValue="2"><option value="2">2</option></select>"#,
            r#"createVNode(256, "select", null, createVNode(1, "option", null, "2", 16, {
  "value": "2"
}), 2, {
  "defaultValue": "2"
});"#,
        );
    }

    #[test]
    fn should_compile_a_textarea_with_default_value() {
        assert_transform(
            r#"<textarea defaultValue="foo" />"#,
            r#"createVNode(128, "textarea", null, null, 1, {
  "defaultValue": "foo"
});"#,
        );
    }

    #[test]
    fn should_compile_a_textarea_with_a_null_value() {
        assert_transform(
            "<textarea value={null} />",
            r#"createVNode(128, "textarea", null, null, 1, {
  "value": null
});"#,
        );
    }

    #[test]
    fn should_compile_a_range_input() {
        assert_transform(
            r#"<input type="range" value={0.5} min="0" max="1" step="0.05" />"#,
            r#"createVNode(64, "input", null, null, 1, {
  "type": "range",
  "value": 0.5,
  "min": "0",
  "max": "1",
  "step": "0.05"
});"#,
        );
    }

    #[test]
    fn should_compile_default_checked_with_checked_false() {
        assert_transform(
            "<input defaultChecked checked={false} />",
            r#"createVNode(64, "input", null, null, 1, {
  "defaultChecked": true,
  "checked": false
});"#,
        );
    }

    #[test]
    fn should_compile_a_progress_element() {
        assert_transform(
            r#"<progress value={50} max="100" />"#,
            r#"createVNode(1, "progress", null, null, 1, {
  "value": 50,
  "max": "100"
});"#,
        );
    }
}

/// tags
mod tags {
    use super::*;

    #[test]
    fn should_compile_annotation_xml_as_an_element() {
        assert_transform(
            r#"<annotation-xml encoding="text/html" />"#,
            r#"createVNode(1, "annotation-xml", null, null, 1, {
  "encoding": "text/html"
});"#,
        );
    }

    #[test]
    fn should_compile_new_html_tags() {
        assert_transform(
            "<search><selectedcontent /></search>",
            r#"createVNode(1, "search", null, createVNode(1, "selectedcontent"), 2);"#,
        );
    }

    #[test]
    fn should_compile_keyed_children_of_a_template() {
        assert_transform(
            "<template>{items.map(i => <li key={i}>{i}</li>)}</template>",
            r#"createVNode(1, "template", null, items.map(i => createVNode(1, "li", null, i, 0, null, i)), 0);"#,
        );
    }

    #[test]
    fn should_keep_an_explicit_xmlns_on_svg() {
        assert_transform(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1" />"#,
            r#"createVNode(32, "svg", null, null, 1, {
  "xmlns": "http://www.w3.org/2000/svg",
  "viewBox": "0 0 1 1"
});"#,
        );
    }

    #[test]
    fn should_keep_the_is_attribute() {
        assert_transform(
            r#"<div is="built-in" />"#,
            r#"createVNode(1, "div", null, null, 1, {
  "is": "built-in"
});"#,
        );
    }
}

/// shapes rejected by preact/debug
mod shapes_rejected_by_preact_debug {
    use super::*;

    #[test]
    fn should_compile_a_div_inside_a_paragraph() {
        assert_transform(
            "<p><div /></p>",
            r#"createVNode(1, "p", null, createVNode(1, "div"), 2);"#,
        );
    }

    #[test]
    fn should_compile_nested_anchors() {
        assert_transform(
            "<a><a /></a>",
            r#"createVNode(1, "a", null, createVNode(1, "a"), 2);"#,
        );
    }

    #[test]
    fn should_compile_nested_buttons() {
        assert_transform(
            "<button><button /></button>",
            r#"createVNode(1, "button", null, createVNode(1, "button"), 2);"#,
        );
    }

    #[test]
    fn should_compile_a_table_row_inside_a_div() {
        assert_transform(
            "<div><tr /></div>",
            r#"createVNode(1, "div", null, createVNode(1, "tr"), 2);"#,
        );
    }

    #[test]
    fn should_compile_a_table_cell_inside_tbody() {
        assert_transform(
            "<tbody><td /></tbody>",
            r#"createVNode(1, "tbody", null, createVNode(1, "td"), 2);"#,
        );
    }

    #[test]
    fn should_compile_a_complete_table() {
        assert_transform(
            "<table><tbody><tr><td /></tr></tbody></table>",
            r#"createVNode(1, "table", null, createVNode(1, "tbody", null, createVNode(1, "tr", null, createVNode(1, "td"), 2), 2), 2);"#,
        );
    }
}
