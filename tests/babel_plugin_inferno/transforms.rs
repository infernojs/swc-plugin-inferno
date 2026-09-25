//! Ported from babel-plugin-inferno `tests.js`: Transforms

use crate::helpers::*;

/// Empty attrs
mod empty_attrs {
    use super::*;

    // swc_ecma_parser rejects the empty expression before the plugin runs. babel-plugin-inferno reports:
    // JSX attributes must only be assigned a non-empty expression.
    #[test]
    fn should_reject_an_empty_ref() {
        assert_transform_error("<div ref={}>{a}</div>", "Expression expected");
    }
}

/// Dynamic children
mod dynamic_children {
    use super::*;

    #[test]
    fn should_add_normalize_call_when_there_is_dynamic_children() {
        assert_transform("<div>{a}</div>", r#"createVNode(1, "div", null, a, 0);"#);
    }

    #[test]
    fn should_add_normalize_call_when_there_is_dynamic_and_static_children_mixed() {
        assert_transform(
            "<div>{a}<div>1</div></div>",
            r#"createVNode(1, "div", null, [a, createVNode(1, "div", null, "1", 16)], 0);"#,
        );
    }

    #[test]
    fn should_not_add_normalize_call_when_all_children_are_known() {
        assert_transform(
            "<div><FooBar/><div>1</div></div>",
            r#"createVNode(1, "div", null, [createComponentVNode(2, FooBar), createVNode(1, "div", null, "1", 16)], 4);"#,
        );
    }

    #[test]
    fn should_not_convert_text_to_create_vnode_when_its_within_component() {
        assert_transform(
            "<FooBar>1</FooBar>",
            r#"createComponentVNode(2, FooBar, {
  children: "1"
});"#,
        );
    }

    #[test]
    fn should_create_text_vnodes_when_there_is_no_normalization_needed_and_its_multiple_children() {
        assert_transform(
            "<div><FooBar/>foobar</div>",
            r#"createVNode(1, "div", null, [createComponentVNode(2, FooBar), createTextVNode("foobar")], 4);"#,
        );
    }

    #[test]
    fn should_create_text_vnodes_when_there_is_single_children() {
        assert_transform(
            "<div>foobar</div>",
            r#"createVNode(1, "div", null, "foobar", 16);"#,
        );
    }

    #[test]
    fn should_create_text_vnodes_when_there_is_single_children_2() {
        assert_transform("<div>1</div>", r#"createVNode(1, "div", null, "1", 16);"#);
    }

    #[test]
    fn should_not_normalize_component_prop_children() {
        assert_transform(
            "<Com>{a}</Com>",
            r"createComponentVNode(2, Com, {
  children: a
});",
        );
    }

    #[test]
    fn should_not_normalize_component_children_as_they_are_in_props() {
        assert_transform(
            "<Com>{a}{b}{c}</Com>",
            r"createComponentVNode(2, Com, {
  children: [a, b, c]
});",
        );
    }

    #[test]
    fn should_mark_parent_v_node_with_has_non_keyed_children_if_no_normalize_is_needed_and_all_children_are_non_keyed()
     {
        assert_transform(
            "<div><FooBar/><div>1</div></div>",
            r#"createVNode(1, "div", null, [createComponentVNode(2, FooBar), createVNode(1, "div", null, "1", 16)], 4);"#,
        );
    }

    #[test]
    fn should_mark_parent_v_node_with_has_keyed_children_if_no_normalize_is_needed_and_all_children_are_keyed()
     {
        assert_transform(
            r#"<div><FooBar key="foo"/><div key="1">1</div></div>"#,
            r#"createVNode(1, "div", null, [createComponentVNode(2, FooBar, null, "foo"), createVNode(1, "div", null, "1", 16, null, "1")], 8);"#,
        );
    }

    #[test]
    fn should_mark_parent_v_node_with_has_keyed_children_if_even_one_child_is_keyed_directly() {
        assert_transform(
            r#"<div><span></span><div key="1">1</div></div>"#,
            r#"createVNode(1, "div", null, [createVNode(1, "span"), createVNode(1, "div", null, "1", 16, null, "1")], 8);"#,
        );
    }
}

/// Dynamic ChildFlags
mod dynamic_child_flags {
    use super::*;

    #[test]
    fn should_be_possible_to_define_override_child_flags_runtime_for_dynamic_children() {
        assert_transform(
            "<img $ChildFlag={bool ? 1 : 2}>{expression}</img>",
            r#"createVNode(1, "img", null, expression, bool ? 1 : 2);"#,
        );
    }

    #[test]
    fn should_be_possible_to_define_override_child_flags_runtime() {
        assert_transform(
            "<img $ChildFlag={1}>foobar</img>",
            r#"createVNode(1, "img", null, "foobar", 1);"#,
        );
    }

    #[test]
    fn should_be_possible_to_use_expression_for_child_flags() {
        assert_transform(
            "<img $ChildFlag={magic}>foobar</img>",
            r#"createVNode(1, "img", null, "foobar", magic);"#,
        );
    }
}

/// different types
mod different_types {
    use super::*;

    #[test]
    fn should_transform_img() {
        assert_transform(
            "<img>foobar</img>",
            r#"createVNode(1, "img", null, "foobar", 16);"#,
        );
    }

    #[test]
    fn should_transform_br() {
        assert_transform(
            "<br>foobar</br>",
            r#"createVNode(1, "br", null, "foobar", 16);"#,
        );
    }

    #[test]
    fn should_transform_media() {
        assert_transform(
            "<media>foobar</media>",
            r#"createVNode(1, "media", null, "foobar", 16);"#,
        );
    }

    #[test]
    fn should_transform_textarea() {
        assert_transform(
            "<textarea>foobar</textarea>",
            r#"createVNode(128, "textarea", null, "foobar", 16);"#,
        );
    }
}

/// Special flags
mod special_flags {
    use super::*;

    #[test]
    fn should_add_keyed_children_flag() {
        assert_transform(
            "<div $HasKeyedChildren>{magic}</div>",
            r#"createVNode(1, "div", null, magic, 8);"#,
        );
    }

    #[test]
    fn should_not_normalize_if_has_vnode_children_set() {
        assert_transform(
            "<div $HasVNodeChildren>{magic}</div>",
            r#"createVNode(1, "div", null, magic, 2);"#,
        );
    }

    #[test]
    fn should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_dynamic()
     {
        assert_transform(
            "<div $HasTextChildren>{foobar}</div>",
            r#"createVNode(1, "div", null, foobar, 16);"#,
        );
    }

    #[test]
    fn should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_hardcoded()
     {
        assert_transform(
            "<div $HasTextChildren>text</div>",
            r#"createVNode(1, "div", null, "text", 16);"#,
        );
    }

    #[test]
    fn should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_hardcoded_2()
     {
        assert_transform(
            r#"<div $HasTextChildren>{"testing"}</div>"#,
            r#"createVNode(1, "div", null, "testing", 16);"#,
        );
    }

    #[test]
    fn should_use_optimized_text_children_instead_create_text_vnode_for_element_single_child() {
        assert_transform(
            "<div>text</div>",
            r#"createVNode(1, "div", null, "text", 16);"#,
        );
    }

    #[test]
    fn should_add_non_keyed_children_flag() {
        assert_transform(
            "<div $HasNonKeyedChildren>{test}</div>",
            r#"createVNode(1, "div", null, test, 4);"#,
        );
    }

    #[test]
    fn should_add_re_create_flag() {
        assert_transform("<div $ReCreate/>", r#"createVNode(2049, "div");"#);
    }

    #[test]
    fn should_be_possible_to_define_override_flags_runtime() {
        assert_transform(
            "<img $Flags={bool ? 1 : 2}>{expression}</img>",
            r#"createVNode(bool ? 1 : 2, "img", null, expression, 0);"#,
        );
    }

    #[test]
    fn should_be_possible_to_define_override_flags_with_constant() {
        assert_transform(
            "<img $Flags={120}>foobar</img>",
            r#"createVNode(120, "img", null, "foobar", 16);"#,
        );
    }

    #[test]
    fn should_be_possible_to_use_expression_for_flags() {
        assert_transform(
            "<ComponentA $Flags={magic}/>",
            "createComponentVNode(magic, ComponentA);",
        );
    }
}

/// onComponent hooks
mod on_component_hooks {
    use super::*;

    #[test]
    fn should_add_hooks_to_refs_for_functional_components() {
        assert_transform(
            r"
<Child
    key={i}
    onComponentDidAppear={childOnComponentDidAppear}
    onComponentDidMount={childOnComponentDidMount}
>
  {i}
</Child>
",
            r#"createComponentVNode(2, Child, {
  children: i
}, i, {
  "onComponentDidAppear": childOnComponentDidAppear,
  "onComponentDidMount": childOnComponentDidMount
});"#,
        );
    }

    #[test]
    fn should_handle_hooks_to_refs_and_ref_for_functional_components() {
        assert_transform(
            r"
<Child
    key={i}
    onComponentDidAppear={childOnComponentDidAppear}
    onComponentDidMount={childOnComponentDidMount}
>
  {i}
</Child>
",
            r#"createComponentVNode(2, Child, {
  children: i
}, i, {
  "onComponentDidAppear": childOnComponentDidAppear,
  "onComponentDidMount": childOnComponentDidMount
});"#,
        );
    }
}

/// spreadOperator
mod spread_operator {
    use super::*;

    #[test]
    fn should_add_call_to_normalize_props_when_spread_operator_is_used() {
        assert_transform(
            "<div {...props}>1</div>",
            r#"normalizeProps(createVNode(1, "div", null, "1", 16, {
  ...props
}));"#,
        );
    }

    #[test]
    fn should_add_call_to_normalize_props_when_spread_operator_is_used_2() {
        assert_transform(
            r#"<div foo="bar" className="test" {...props}/>"#,
            r#"normalizeProps(createVNode(1, "div", "test", null, 1, {
  "foo": "bar",
  ...props
}));"#,
        );
    }

    #[test]
    fn should_add_call_to_normalize_props_when_spread_operator_is_used_inside_children_for_component()
     {
        assert_transform(
            "<FooBar><BarFoo {...props}/><NoNormalize/></FooBar>",
            r"createComponentVNode(2, FooBar, {
  children: [normalizeProps(createComponentVNode(2, BarFoo, {
    ...props
  })), createComponentVNode(2, NoNormalize)]
});",
        );
    }

    #[test]
    fn should_do_single_normalization_when_multiple_spread_operators_are_used() {
        assert_transform(
            "<FooBar><BarFoo {...magics} {...foobars} {...props}/><NoNormalize/></FooBar>",
            r"createComponentVNode(2, FooBar, {
  children: [normalizeProps(createComponentVNode(2, BarFoo, {
    ...magics,
    ...foobars,
    ...props
  })), createComponentVNode(2, NoNormalize)]
});",
        );
    }
}

/// Basic scenarios
mod basic_scenarios {
    use super::*;

    #[test]
    fn should_transform_div() {
        assert_transform("<div></div>", r#"createVNode(1, "div");"#);
    }

    #[test]
    fn should_transform_single_div() {
        assert_transform("<div>1</div>", r#"createVNode(1, "div", null, "1", 16);"#);
    }

    #[test]
    fn test_to_verify_stripping_imports_work() {
        assert_transform("<div>1</div>", r#"createVNode(1, "div", null, "1", 16);"#);
    }

    #[test]
    fn class_name_should_be_in_third_parameter_as_string_when_its_element() {
        assert_transform(
            r#"<div className="first second">1</div>"#,
            r#"createVNode(1, "div", "first second", "1", 16);"#,
        );
    }

    #[test]
    fn class_name_should_be_in_fifth_parameter_as_string_when_its_component() {
        assert_transform(
            r#"<UnknownClass className="first second">1</UnknownClass>"#,
            r#"createComponentVNode(2, UnknownClass, {
  "className": "first second",
  children: "1"
});"#,
        );
    }

    #[test]
    fn jsxmember_expressions_should_work() {
        assert_transform(
            "<Components.Unknown>1</Components.Unknown>",
            r#"createComponentVNode(2, Components.Unknown, {
  children: "1"
});"#,
        );
    }

    #[test]
    fn class_should_be_in_third_parameter_as_variable() {
        assert_transform(
            "<div class={variable}>1</div>",
            r#"createVNode(1, "div", variable, "1", 16);"#,
        );
    }

    #[test]
    fn should_call_create_vnode_twice_and_text_children() {
        assert_transform(
            r"<div>
          <div>single</div>
        </div>",
            r#"createVNode(1, "div", null, createVNode(1, "div", null, "single", 16), 2);"#,
        );
    }

    #[test]
    fn events_should_be_in_props() {
        assert_transform(
            r#"<div id="test" onClick={func} class={variable}>1</div>"#,
            r#"createVNode(1, "div", variable, "1", 16, {
  "id": "test",
  "onClick": func
});"#,
        );
    }

    #[test]
    fn should_transform_input_and_html_for_correctly() {
        assert_transform(
            r#"<label htmlFor={id}><input id={id} name={name} value={value} onChange={onChange} onInput={onInput} onKeyup={onKeyup} onFocus={onFocus} onClick={onClick} type="number" pattern="[0-9]+([,.][0-9]+)?" inputMode="numeric" min={minimum}/></label>"#,
            r#"createVNode(1, "label", null, createVNode(64, "input", null, null, 1, {
  "id": id,
  "name": name,
  "value": value,
  "onChange": onChange,
  "onInput": onInput,
  "onKeyup": onKeyup,
  "onFocus": onFocus,
  "onClick": onClick,
  "type": "number",
  "pattern": "[0-9]+([,.][0-9]+)?",
  "inputmode": "numeric",
  "min": minimum
}), 2, {
  "for": id
});"#,
        );
    }

    #[test]
    fn should_transform_accept_charset_correctly() {
        assert_transform(
            r#"<form acceptCharset="ISO-8859-1"/>"#,
            r#"createVNode(1, "form", null, null, 1, {
  "accept-charset": "ISO-8859-1"
});"#,
        );
    }

    #[test]
    fn should_lower_case_f_e_col_span() {
        assert_transform(
            r#"<td colSpan="5"/>"#,
            r#"createVNode(1, "td", null, null, 1, {
  "colspan": "5"
});"#,
        );
    }

    #[test]
    fn should_transform_on_double_click_to_native_html_event() {
        assert_transform(
            "<div onDoubleClick={foobar}></div>",
            r#"createVNode(1, "div", null, null, 1, {
  "onDblClick": foobar
});"#,
        );
    }
}

/// contenteditbale
mod contenteditbale {
    use super::*;

    #[test]
    fn should_set_additional_byte_on_when_contenteditbale_attribute_is_found() {
        assert_transform(
            "<div contentEditable></div>",
            r#"createVNode(4097, "div", null, null, 1, {
  "contentEditable": true
});"#,
        );
        assert_transform(
            r#"<span contenteditable="false"></span>"#,
            r#"createVNode(4097, "span", null, null, 1, {
  "contenteditable": "false"
});"#,
        );
        assert_transform(
            "<div contenteditable></div>",
            r#"createVNode(4097, "div", null, null, 1, {
  "contenteditable": true
});"#,
        );
        assert_transform(
            "<div contentEditable={logic}></div>",
            r#"createVNode(4097, "div", null, null, 1, {
  "contentEditable": logic
});"#,
        );
        assert_transform(
            r#"<div contentEditable="true"></div>"#,
            r#"createVNode(4097, "div", null, null, 1, {
  "contentEditable": "true"
});"#,
        );
    }
}

// Not ported from Pragma option:
// - Should replace createVNode to pragma option value (swc-plugin-inferno has no imports: false, pragma options)

// Not ported from defineAllArguments option:
// - Should replace createVNode to pragma option value (swc-plugin-inferno has no imports: false, defineAllArguments options)

/// SVG attributes React syntax support
mod svg_attributes_react_syntax_support {
    use super::*;

    #[test]
    fn should_support_native_xlink_href() {
        assert_transform(
            r##"<svg><use xlink:href="#tester"></use></svg>"##,
            r##"createVNode(32, "svg", null, createVNode(32, "use", null, null, 1, {
  "xlink:href": "#tester"
}), 2);"##,
        );
    }

    #[test]
    fn should_transform_xlink_href_to_xlink_href() {
        assert_transform(
            r##"<svg><use xlinkHref="#tester"></use></svg>"##,
            r##"createVNode(32, "svg", null, createVNode(32, "use", null, null, 1, {
  "xlink:href": "#tester"
}), 2);"##,
        );
    }

    #[test]
    fn should_transform_stroke_width_to_stroke_width() {
        assert_transform(
            r#"<svg><rect strokeWidth="1px"></rect></svg>"#,
            r#"createVNode(32, "svg", null, createVNode(32, "rect", null, null, 1, {
  "stroke-width": "1px"
}), 2);"#,
        );
    }

    #[test]
    fn should_transform_stroke_width_to_stroke_width_2() {
        assert_transform(
            r#"<svg><rect fillOpacity="1"></rect></svg>"#,
            r#"createVNode(32, "svg", null, createVNode(32, "rect", null, null, 1, {
  "fill-opacity": "1"
}), 2);"#,
        );
    }

    #[test]
    fn should_not_transform_stroke_with_or_other_svg_attributes_if_they_are_used_in_component() {
        assert_transform(
            r#"<Foobar strokeWidth="1px" fillOpacity="1"/>"#,
            r#"createComponentVNode(2, Foobar, {
  "strokeWidth": "1px",
  "fillOpacity": "1"
});"#,
        );
    }
}

/// text node and elements mixed
mod text_node_and_elements_mixed {
    use super::*;

    #[test]
    fn should_create_text_vnode_when_there_are_siblings() {
        assert_transform(
            "<div>Okay<span>foo</span></div>",
            r#"createVNode(1, "div", null, [createTextVNode("Okay"), createVNode(1, "span", null, "foo", 16)], 4);"#,
        );
    }

    #[test]
    fn should_create_text_vnode_when_text_node_is_under_short_syntax_fragment() {
        assert_transform(
            "<>Okay<span>foo</span></>",
            r#"createFragment([createTextVNode("Okay"), createVNode(1, "span", null, "foo", 16)], 4);"#,
        );
    }

    #[test]
    fn should_not_wrap_dynamic_value() {
        assert_transform("<>{magic}</>", "createFragment(magic, 0);");
    }

    #[test]
    fn should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_short_syntax_fragment()
     {
        assert_transform(
            "<><>Text</></>",
            r#"createFragment([createFragment([createTextVNode("Text")], 4)], 4);"#,
        );
    }

    #[test]
    fn should_always_short_syntax_fragment() {
        assert_transform(
            "<><><div>Text</div></></>",
            r#"createFragment([createFragment([createVNode(1, "div", null, "Text", 16)], 4)], 4);"#,
        );
    }

    #[test]
    fn should_handle_many_dynamic_children_short_syntax() {
        assert_transform(
            "<><>{Frag}Text{Wohoo}</></>",
            r#"createFragment([createFragment([Frag, createTextVNode("Text"), Wohoo], 0)], 4);"#,
        );
    }

    #[test]
    fn should_handle_many_dynamic_and_non_dynamic_children_short_syntax() {
        assert_transform(
            "<><><span></span>Text{Wohoo}</></>",
            r#"createFragment([createFragment([createVNode(1, "span"), createTextVNode("Text"), Wohoo], 0)], 4);"#,
        );
    }

    #[test]
    fn should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_long_syntax_fragment()
     {
        assert_transform(
            "<Fragment><Fragment>Text</Fragment></Fragment>",
            r#"createFragment([createFragment([createTextVNode("Text")], 4)], 4);"#,
        );
    }

    #[test]
    fn should_create_text_vnode_when_text_node_is_under_large_syntax_fragment() {
        assert_transform(
            "<Fragment>Okay<span>foo</span></Fragment>",
            r#"createFragment([createTextVNode("Okay"), createVNode(1, "span", null, "foo", 16)], 4);"#,
        );
    }

    #[test]
    fn should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_long_syntax_fragment_2()
     {
        assert_transform(
            "<Fragment><Fragment>Text</Fragment></Fragment>",
            r#"createFragment([createFragment([createTextVNode("Text")], 4)], 4);"#,
        );
    }

    #[test]
    fn should_always_long_syntax_fragment() {
        assert_transform(
            "<Fragment><Fragment><div>Text</div></Fragment></Fragment>",
            r#"createFragment([createFragment([createVNode(1, "div", null, "Text", 16)], 4)], 4);"#,
        );
    }

    #[test]
    fn should_handle_many_dynamic_children_long_syntax() {
        assert_transform(
            "<Fragment><Fragment>{Frag}Text{Wohoo}</Fragment></Fragment>",
            r#"createFragment([createFragment([Frag, createTextVNode("Text"), Wohoo], 0)], 4);"#,
        );
    }

    #[test]
    fn should_handle_many_dynamic_and_non_dynamic_children_long_syntax() {
        assert_transform(
            "<Fragment><Fragment><span></span>Text{Wohoo}</Fragment></Fragment>",
            r#"createFragment([createFragment([createVNode(1, "span"), createTextVNode("Text"), Wohoo], 0)], 4);"#,
        );
    }
}

/// Imports
mod imports {
    use super::*;

    #[test]
    fn should_not_fail_if_create_vnode_is_already_imported() {
        assert_js_eq(
            &transform_with(
                "{}",
                r#"import {createVNode} from "inferno"; var foo = <div/>;"#,
            ),
            r#"import { createVNode } from "inferno";
var foo = createVNode(1, "div");"#,
        );
    }

    // babel-plugin-inferno adds a separate import declaration.
    #[test]
    fn should_add_create_component_vnode_to_an_existing_create_vnode_import() {
        assert_js_eq(
            &transform_with(
                "{}",
                r#"import {createVNode} from "inferno"; var foo = <FooBar/>;"#,
            ),
            r#"import { createVNode, createComponentVNode } from "inferno";
var foo = createComponentVNode(2, FooBar);"#,
        );
    }
}

/// Children
mod children {
    use super::*;

    #[test]
    fn element_should_prefer_child_element_over_children_props() {
        assert_transform(
            r#"<div children="ab">test</div>"#,
            r#"createVNode(1, "div", null, "test", 16);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn element_should_prefer_prop_over_empty_children() {
        assert_transform(
            r#"<div children="ab"></div>"#,
            r#"createVNode(1, "div", null, "ab", 16);"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn element_should_use_prop_if_no_children_exists() {
        assert_transform(
            r#"<div children="ab"/>"#,
            r#"createVNode(1, "div", null, "ab", 16);"#,
        );
    }

    #[test]
    fn component_should_prefer_child_element_over_children_props() {
        assert_transform(
            r#"<Com children="ab">test</Com>"#,
            r#"createComponentVNode(2, Com, {
  children: "test"
});"#,
        );
    }

    #[test]
    fn component_should_prefer_prop_over_empty_children() {
        assert_transform(
            r#"<Com children="ab"></Com>"#,
            r#"createComponentVNode(2, Com, {
  "children": "ab"
});"#,
        );
    }

    #[test]
    fn component_should_use_prop_if_no_children_exists() {
        assert_transform(
            r#"<Com children="ab"/>"#,
            r#"createComponentVNode(2, Com, {
  "children": "ab"
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno passes an empty array literal child as children"]
    fn component_array_empty_children() {
        assert_transform("<Com>{[]}</Com>", "createComponentVNode(2, Com);");
    }

    #[test]
    fn component_should_create_v_node_for_children() {
        assert_transform(
            "<Com children={<div>1</div>}/>",
            r#"createComponentVNode(2, Com, {
  "children": createVNode(1, "div", null, "1", 16)
});"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno omits childFlags for a children prop, so Inferno ignores the children"]
    fn should_prefer_xml_children_over_props() {
        assert_transform(
            "<foo children={<span>b</span>}></foo>",
            r#"createVNode(1, "foo", null, createVNode(1, "span", null, "b", 16), 2);"#,
        );
    }

    // babel-plugin-inferno leaves the null children out; without childFlags Inferno ignores them too.
    #[test]
    fn should_prefer_xml_children_over_props_null() {
        assert_transform(
            "<foo children={null}></foo>",
            r#"createVNode(1, "foo", null, null);"#,
        );
    }
}

/// Fragments
mod fragments {
    use super::*;

    /// Short syntax
    mod short_syntax {
        use super::*;

        #[test]
        fn should_create_empty_create_fragment() {
            assert_transform("<></>", "createFragment();");
        }

        #[test]
        fn should_create_fragment() {
            assert_transform(
                "<>Test</>",
                r#"createFragment([createTextVNode("Test")], 4);"#,
            );
        }

        #[test]
        fn should_create_fragment_dynamic_children() {
            assert_transform("<>{dynamic}</>", "createFragment(dynamic, 0);");
        }

        #[test]
        fn should_create_fragment_keyed_children() {
            assert_transform(
                r#"<><span key="ok">kk</span><div key="ok2">ok</div></>"#,
                r#"createFragment([createVNode(1, "span", null, "kk", 16, null, "ok"), createVNode(1, "div", null, "ok", 16, null, "ok2")], 8);"#,
            );
        }

        #[test]
        fn should_create_fragment_non_keyed_children() {
            assert_transform(
                "<><div>1</div><span>foo</span></>",
                r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4);"#,
            );
        }
    }

    /// Long syntax
    mod long_syntax {
        use super::*;

        /// Fragment
        mod fragment {
            use super::*;

            #[test]
            fn should_create_empty_create_fragment() {
                assert_transform("<Fragment></Fragment>", "createFragment();");
                assert_transform("<Fragment/>", "createFragment();");
            }

            #[test]
            fn should_create_fragment() {
                assert_transform(
                    "<Fragment>Test</Fragment>",
                    r#"createFragment([createTextVNode("Test")], 4);"#,
                );
            }

            #[test]
            fn should_create_fragment_dynamic_children() {
                assert_transform(
                    "<Fragment>{dynamic}</Fragment>",
                    "createFragment(dynamic, 0);",
                );
            }

            #[test]
            fn should_create_fragment_keyed_children() {
                assert_transform(
                    r#"<Fragment><span key="ok">kk</span><div key="ok2">ok</div></Fragment>"#,
                    r#"createFragment([createVNode(1, "span", null, "kk", 16, null, "ok"), createVNode(1, "div", null, "ok", 16, null, "ok2")], 8);"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children() {
                assert_transform(
                    "<Fragment><div>1</div><span>foo</span></Fragment>",
                    r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4);"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_2() {
                assert_transform(
                    r#"<Fragment key="foo"><div>1</div><span>foo</span></Fragment>"#,
                    r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4, "foo");"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_3() {
                assert_transform(
                    r#"<Fragment key="foo" $HasKeyedChildren>{magic}</Fragment>"#,
                    r#"createFragment(magic, 8, "foo");"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_4() {
                assert_transform(
                    r#"<Fragment key="foo" $HasNonKeyedChildren>{magic}</Fragment>"#,
                    r#"createFragment(magic, 4, "foo");"#,
                );
            }
        }

        /// Inferno.Fragment
        mod inferno_fragment {
            use super::*;

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment() {
                assert_transform(
                    "<Inferno.Fragment>Test</Inferno.Fragment>",
                    r#"createFragment([createTextVNode("Test")], 4);"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_dynamic_children() {
                assert_transform(
                    "<Inferno.Fragment>{dynamic}</Inferno.Fragment>",
                    "createFragment(dynamic, 0);",
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_keyed_children() {
                assert_transform(
                    r#"<Inferno.Fragment><span key="ok">kk</span><div key="ok2">ok</div></Inferno.Fragment>"#,
                    r#"createFragment([createVNode(1, "span", null, "kk", 16, null, "ok"), createVNode(1, "div", null, "ok", 16, null, "ok2")], 8);"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_non_keyed_children() {
                assert_transform(
                    "<Inferno.Fragment><div>1</div><span>foo</span></Inferno.Fragment>",
                    r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4);"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_non_keyed_children_2() {
                assert_transform(
                    r#"<Inferno.Fragment key="foo"><div>1</div><span>foo</span></Inferno.Fragment>"#,
                    r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4, "foo");"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_ignore_all_other_props() {
                assert_transform(
                    r#"<Inferno.Fragment abc="foobar" id="test" key="foo"><div>1</div><span>foo</span></Inferno.Fragment>"#,
                    r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4, "foo");"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_non_keyed_children_3() {
                assert_transform(
                    r#"<Inferno.Fragment key="foo" $HasKeyedChildren>{magic}</Inferno.Fragment>"#,
                    r#"createFragment(magic, 8, "foo");"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_non_keyed_children_4() {
                assert_transform(
                    r#"<Inferno.Fragment key="foo" $HasNonKeyedChildren>{magic}</Inferno.Fragment>"#,
                    r#"createFragment(magic, 4, "foo");"#,
                );
            }
        }

        /// React.Fragment
        mod react_fragment {
            use super::*;

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment() {
                assert_transform(
                    "<React.Fragment>Test</React.Fragment>",
                    r#"createFragment([createTextVNode("Test")], 4);"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_dynamic_children() {
                assert_transform(
                    "<React.Fragment>{dynamic}</React.Fragment>",
                    "createFragment(dynamic, 0);",
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_keyed_children() {
                assert_transform(
                    r#"<React.Fragment><span key="ok">kk</span><div key="ok2">ok</div></React.Fragment>"#,
                    r#"createFragment([createVNode(1, "span", null, "kk", 16, null, "ok"), createVNode(1, "div", null, "ok", 16, null, "ok2")], 8);"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_non_keyed_children() {
                assert_transform(
                    "<React.Fragment><div>1</div><span>foo</span></React.Fragment>",
                    r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4);"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_non_keyed_children_2() {
                assert_transform(
                    r#"<React.Fragment key="foo"><div>1</div><span>foo</span></React.Fragment>"#,
                    r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4, "foo");"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_ignore_all_other_props() {
                assert_transform(
                    r#"<React.Fragment abc="foobar" id="test" key="foo"><div>1</div><span>foo</span></React.Fragment>"#,
                    r#"createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4, "foo");"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_non_keyed_children_3() {
                assert_transform(
                    r#"<React.Fragment key="foo" $HasKeyedChildren>{magic}</React.Fragment>"#,
                    r#"createFragment(magic, 8, "foo");"#,
                );
            }

            #[test]
            #[ignore = "swc-plugin-inferno only treats Fragment as a fragment, not member expressions ending in Fragment"]
            fn should_create_fragment_non_keyed_children_4() {
                assert_transform(
                    r#"<React.Fragment key="foo" $HasNonKeyedChildren>{magic}</React.Fragment>"#,
                    r#"createFragment(magic, 4, "foo");"#,
                );
            }
        }
    }
}
