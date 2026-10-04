//! Ported from babel-plugin-inferno `tests/preact-parity.test.js`: Preact parity

use crate::helpers::*;

/// children and text
mod children_and_text {
    use super::*;

    #[test]
    fn should_not_merge_text_with_a_string_literal_child_hydrate_test() {
        assert_transform(
            r#"<p>hello {"foo"}</p>"#,
            r#"newVNode(1, "p", null, [newTextVNode("hello "), newTextVNode("foo")]);"#,
        );
    }

    #[test]
    fn should_keep_text_between_falsy_expression_children_render_test() {
        assert_transform(
            "<div>{null},{undefined},{false},{0},{NaN}</div>",
            r#"newVNode(1, "div", null, [null, newTextVNode(","), undefined, newTextVNode(","), false, newTextVNode(","), 0, newTextVNode(","), NaN]);"#,
        );
    }

    #[test]
    fn should_keep_text_on_the_first_and_last_lines_around_elements_render_test() {
        assert_transform(
            r"<div>0<span />
<input />
<div />1</div>",
            r#"newVNode(5, "div", null, [newTextVNode("0"), newVNode(17, "span"), newVNode(528, "input"), newVNode(17, "div"), newTextVNode("1")]);"#,
        );
    }

    #[test]
    fn should_keep_the_trailing_space_before_an_element_render_test() {
        assert_transform(
            "<div>hello <span>world</span></div>",
            r#"newVNode(5, "div", null, [newTextVNode("hello "), newVNode(3, "span", null, "world")]);"#,
        );
    }

    #[test]
    fn should_keep_quotes_and_in_text_render_test() {
        assert_transform(
            r##"<a href="#">href="#"</a>"##,
            r##"newVNode(3, "a", null, "href=\"#\"", {
  "href": "#"
});"##,
        );
    }

    #[test]
    fn should_split_text_and_an_expression_without_space_render_test() {
        assert_transform(
            r#"<h1 class="fade-down">Hi{name}</h1>"#,
            r#"newVNode(1, "h1", "fade-down", [newTextVNode("Hi"), name]);"#,
        );
    }

    #[test]
    fn should_pass_an_array_children_prop_to_a_component_create_element_test() {
        assert_transform(
            r#"<Foo a="b" children={[<span class="bar">bar</span>, "123", 456]} />"#,
            r#"newComponentVNode(0, Foo, {
  "a": "b",
  "children": [newVNode(3, "span", "bar", "bar"), "123", 456]
});"#,
        );
    }

    #[test]
    fn should_prefer_jsx_children_over_an_array_children_prop_render_test() {
        assert_transform(
            r#"<div a children={["a", "b"]}>c</div>"#,
            r#"newVNode(3, "div", null, "c", {
  "a": true
});"#,
        );
    }

    #[test]
    fn should_compile_a_function_child_of_a_lowercase_consumer_create_context_test() {
        assert_transform(
            "<context.Consumer>{v => <p>{v.state}</p>}</context.Consumer>",
            r#"newComponentVNode(0, context.Consumer, {
  children: v => newVNode(1, "p", null, v.state)
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
            r"normalizeProps(newComponentVNode(0, Inner, {
  ...data,
  ...childData
}));",
        );
    }

    #[test]
    fn should_keep_class_before_a_spread() {
        assert_transform(
            r#"<ul class="old" {...props} />"#,
            r#"normalizeProps(newVNode(17, "ul", "old", null, {
  ...props
}));"#,
        );
    }

    #[test]
    fn should_keep_a_template_literal_class_name_before_a_spread() {
        assert_transform(
            "<div className={`${className} foo`} {...props} />",
            r#"normalizeProps(newVNode(17, "div", `${className} foo`, null, {
  ...props
}));"#,
        );
    }

    #[test]
    fn should_keep_a_key_after_a_spread_of_an_object_with_a_nested_spread() {
        assert_transform(
            "<ListItem {...{ isSelected, setSelected, ...item }} key={item.name} />",
            r"normalizeProps(newComponentVNode(0, ListItem, {
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
            r#"newVNode(80, "svg", c && "bar_" + c);"#,
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
            r#"newVNode(17, "div", null, null, {
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
            r#"newVNode(17, "a", null, null, {
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
            r#"newVNode(17, "div", null, null, {
  "style": "top: 5px; position: relative;"
});"#,
        );
    }

    #[test]
    fn should_pass_a_style_object_with_mixed_key_styles() {
        assert_transform(
            r#"<div style={{gridRowStart: 1, opacity: 0, "--fooBar": 1, "background-size": "cover"}} />"#,
            r#"newVNode(17, "div", null, null, {
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
            r#"newVNode(17, "table", null, null, {
  "border": false
});"#,
        );
    }

    #[test]
    fn should_lowercase_row_span_and_col_span() {
        assert_transform(
            "<td rowSpan={2} colSpan={2} />",
            r#"newVNode(17, "td", null, null, {
  "rowspan": 2,
  "colspan": 2
});"#,
        );
    }

    #[test]
    fn should_lowercase_null_max_length_and_min_length() {
        assert_transform(
            "<input maxLength={null} minLength={null} />",
            r#"newVNode(528, "input", null, null, {
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
            r#"newVNode(4104, "select", null, newVNode(3, "option", null, "B", {
  "selected": true,
  "value": "B"
}), {
  "multiple": true,
  "value": ["B", "C"]
});"#,
        );
    }

    #[test]
    fn should_compile_a_select_with_default_value() {
        assert_transform(
            r#"<select defaultValue="2"><option value="2">2</option></select>"#,
            r#"newVNode(4104, "select", null, newVNode(3, "option", null, "2", {
  "value": "2"
}), {
  "defaultValue": "2"
});"#,
        );
    }

    #[test]
    fn should_compile_a_textarea_with_default_value() {
        assert_transform(
            r#"<textarea defaultValue="foo" />"#,
            r#"newVNode(2064, "textarea", null, null, {
  "defaultValue": "foo"
});"#,
        );
    }

    #[test]
    fn should_compile_a_textarea_with_a_null_value() {
        assert_transform(
            "<textarea value={null} />",
            r#"newVNode(2064, "textarea", null, null, {
  "value": null
});"#,
        );
    }

    #[test]
    fn should_compile_a_range_input() {
        assert_transform(
            r#"<input type="range" value={0.5} min="0" max="1" step="0.05" />"#,
            r#"newVNode(528, "input", null, null, {
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
            r#"newVNode(528, "input", null, null, {
  "defaultChecked": true,
  "checked": false
});"#,
        );
    }

    #[test]
    fn should_compile_a_progress_element() {
        assert_transform(
            r#"<progress value={50} max="100" />"#,
            r#"newVNode(17, "progress", null, null, {
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
            r#"newVNode(17, "annotation-xml", null, null, {
  "encoding": "text/html"
});"#,
        );
    }

    #[test]
    fn should_compile_new_html_tags() {
        assert_transform(
            "<search><selectedcontent /></search>",
            r#"newVNode(9, "search", null, newVNode(17, "selectedcontent"));"#,
        );
    }

    #[test]
    fn should_compile_keyed_children_of_a_template() {
        assert_transform(
            "<template>{items.map(i => <li key={i}>{i}</li>)}</template>",
            r#"newVNode(1, "template", null, items.map(i => newVNode(1, "li", null, i, null, i)));"#,
        );
    }

    #[test]
    fn should_keep_an_explicit_xmlns_on_svg() {
        assert_transform(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1" />"#,
            r#"newVNode(80, "svg", null, null, {
  "xmlns": "http://www.w3.org/2000/svg",
  "viewBox": "0 0 1 1"
});"#,
        );
    }

    #[test]
    fn should_keep_the_is_attribute() {
        assert_transform(
            r#"<div is="built-in" />"#,
            r#"newVNode(17, "div", null, null, {
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
            r#"newVNode(9, "p", null, newVNode(17, "div"));"#,
        );
    }

    #[test]
    fn should_compile_nested_anchors() {
        assert_transform(
            "<a><a /></a>",
            r#"newVNode(9, "a", null, newVNode(17, "a"));"#,
        );
    }

    #[test]
    fn should_compile_nested_buttons() {
        assert_transform(
            "<button><button /></button>",
            r#"newVNode(9, "button", null, newVNode(17, "button"));"#,
        );
    }

    #[test]
    fn should_compile_a_table_row_inside_a_div() {
        assert_transform(
            "<div><tr /></div>",
            r#"newVNode(9, "div", null, newVNode(17, "tr"));"#,
        );
    }

    #[test]
    fn should_compile_a_table_cell_inside_tbody() {
        assert_transform(
            "<tbody><td /></tbody>",
            r#"newVNode(9, "tbody", null, newVNode(17, "td"));"#,
        );
    }

    #[test]
    fn should_compile_a_complete_table() {
        assert_transform(
            "<table><tbody><tr><td /></tr></tbody></table>",
            r#"newVNode(9, "table", null, newVNode(9, "tbody", null, newVNode(9, "tr", null, newVNode(17, "td"))));"#,
        );
    }
}
