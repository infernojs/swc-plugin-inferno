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
        assert_transform("<div>{a}</div>", r#"newVNode(1, "div", null, a);"#);
    }

    #[test]
    fn should_add_normalize_call_when_there_is_dynamic_and_static_children_mixed() {
        assert_transform(
            "<div>{a}<div>1</div></div>",
            r#"newVNode(1, "div", null, [a, newVNode(3, "div", null, "1")]);"#,
        );
    }

    #[test]
    fn should_not_add_normalize_call_when_all_children_are_known() {
        assert_transform(
            "<div><FooBar/><div>1</div></div>",
            r#"newVNode(5, "div", null, [newComponentVNode(0, FooBar), newVNode(3, "div", null, "1")]);"#,
        );
    }

    #[test]
    fn should_not_convert_text_to_create_vnode_when_its_within_component() {
        assert_transform(
            "<FooBar>1</FooBar>",
            r#"newComponentVNode(0, FooBar, {
  children: "1"
});"#,
        );
    }

    #[test]
    fn should_create_text_vnodes_when_there_is_no_normalization_needed_and_its_multiple_children() {
        assert_transform(
            "<div><FooBar/>foobar</div>",
            r#"newVNode(5, "div", null, [newComponentVNode(0, FooBar), newTextVNode("foobar")]);"#,
        );
    }

    #[test]
    fn should_create_text_vnodes_when_there_is_single_children() {
        assert_transform(
            "<div>foobar</div>",
            r#"newVNode(3, "div", null, "foobar");"#,
        );
    }

    #[test]
    fn should_create_text_vnodes_when_there_is_single_children_2() {
        assert_transform("<div>1</div>", r#"newVNode(3, "div", null, "1");"#);
    }

    #[test]
    fn should_not_normalize_component_prop_children() {
        assert_transform(
            "<Com>{a}</Com>",
            r"newComponentVNode(0, Com, {
  children: a
});",
        );
    }

    #[test]
    fn should_not_normalize_component_children_as_they_are_in_props() {
        assert_transform(
            "<Com>{a}{b}{c}</Com>",
            r"newComponentVNode(0, Com, {
  children: [a, b, c]
});",
        );
    }

    #[test]
    fn should_mark_parent_v_node_with_has_non_keyed_children_if_no_normalize_is_needed_and_all_children_are_non_keyed()
     {
        assert_transform(
            "<div><FooBar/><div>1</div></div>",
            r#"newVNode(5, "div", null, [newComponentVNode(0, FooBar), newVNode(3, "div", null, "1")]);"#,
        );
    }

    #[test]
    fn should_mark_parent_v_node_with_has_keyed_children_if_no_normalize_is_needed_and_all_children_are_keyed()
     {
        assert_transform(
            r#"<div><FooBar key="foo"/><div key="1">1</div></div>"#,
            r#"newVNode(33, "div", null, [newComponentVNode(0, FooBar, null, "foo"), newVNode(3, "div", null, "1", null, "1")]);"#,
        );
    }

    #[test]
    fn should_mark_parent_v_node_with_has_keyed_children_if_even_one_child_is_keyed_directly() {
        assert_transform(
            r#"<div><span></span><div key="1">1</div></div>"#,
            r#"newVNode(33, "div", null, [newVNode(17, "span"), newVNode(3, "div", null, "1", null, "1")]);"#,
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
            "<img $ChildFlag={1}>{foobar}</img>",
            r#"newVNode(17, "img", null, foobar);"#,
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
            r#"newVNode(3, "img", null, "foobar");"#,
        );
    }

    #[test]
    fn should_transform_br() {
        assert_transform("<br>foobar</br>", r#"newVNode(3, "br", null, "foobar");"#);
    }

    #[test]
    fn should_transform_media() {
        assert_transform(
            "<media>foobar</media>",
            r#"newVNode(3, "media", null, "foobar");"#,
        );
    }

    #[test]
    fn should_transform_textarea() {
        assert_transform(
            "<textarea>foobar</textarea>",
            r#"newVNode(2050, "textarea", null, "foobar");"#,
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
            r#"newVNode(33, "div", null, magic);"#,
        );
    }

    #[test]
    fn should_not_normalize_if_has_vnode_children_set() {
        assert_transform(
            "<div $HasVNodeChildren>{magic}</div>",
            r#"newVNode(9, "div", null, magic);"#,
        );
    }

    #[test]
    fn should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_dynamic()
     {
        assert_transform(
            "<div $HasTextChildren>{foobar}</div>",
            r#"newVNode(3, "div", null, foobar);"#,
        );
    }

    #[test]
    fn should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_hardcoded()
     {
        assert_transform(
            "<div $HasTextChildren>text</div>",
            r#"newVNode(3, "div", null, "text");"#,
        );
    }

    #[test]
    fn should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_hardcoded_2()
     {
        assert_transform(
            r#"<div $HasTextChildren>{"testing"}</div>"#,
            r#"newVNode(3, "div", null, "testing");"#,
        );
    }

    #[test]
    fn should_use_optimized_text_children_instead_create_text_vnode_for_element_single_child() {
        assert_transform("<div>text</div>", r#"newVNode(3, "div", null, "text");"#);
    }

    #[test]
    fn should_add_non_keyed_children_flag() {
        assert_transform(
            "<div $HasNonKeyedChildren>{test}</div>",
            r#"newVNode(5, "div", null, test);"#,
        );
    }

    #[test]
    fn should_throw_for_the_removed_re_create_flag() {
        assert_transform_error(
            "<div $ReCreate/>",
            "$ReCreate has been removed in Inferno 10. To re-create the element, change its key instead, for example key={version}.",
        );
    }

    #[test]
    fn should_be_possible_to_define_override_flags_runtime() {
        assert_transform(
            "<img $Flags={bool ? 1 : 2}>{expression}</img>",
            r#"newVNode(bool ? 1 : 2, "img", null, expression);"#,
        );
    }

    #[test]
    fn should_be_possible_to_define_override_flags_with_constant() {
        assert_transform(
            "<img $Flags={120}>foobar</img>",
            r#"newVNode(122, "img", null, "foobar");"#,
        );
    }

    #[test]
    fn should_be_possible_to_use_expression_for_flags() {
        assert_transform(
            "<ComponentA $Flags={magic}/>",
            "newComponentVNode(magic | 16, ComponentA);",
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
            r#"newComponentVNode(0, Child, {
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
            r#"newComponentVNode(0, Child, {
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
            r#"normalizeProps(newVNode(3, "div", null, "1", {
  ...props
}));"#,
        );
    }

    #[test]
    fn should_add_call_to_normalize_props_when_spread_operator_is_used_2() {
        assert_transform(
            r#"<div foo="bar" className="test" {...props}/>"#,
            r#"normalizeProps(newVNode(17, "div", "test", null, {
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
            r"newComponentVNode(0, FooBar, {
  children: [normalizeProps(newComponentVNode(0, BarFoo, {
    ...props
  })), newComponentVNode(0, NoNormalize)]
});",
        );
    }

    #[test]
    fn should_do_single_normalization_when_multiple_spread_operators_are_used() {
        assert_transform(
            "<FooBar><BarFoo {...magics} {...foobars} {...props}/><NoNormalize/></FooBar>",
            r"newComponentVNode(0, FooBar, {
  children: [normalizeProps(newComponentVNode(0, BarFoo, {
    ...magics,
    ...foobars,
    ...props
  })), newComponentVNode(0, NoNormalize)]
});",
        );
    }
}

/// Basic scenarios
mod basic_scenarios {
    use super::*;

    #[test]
    fn should_transform_div() {
        assert_transform("<div></div>", r#"newVNode(17, "div");"#);
    }

    #[test]
    fn should_transform_single_div() {
        assert_transform("<div>1</div>", r#"newVNode(3, "div", null, "1");"#);
    }

    #[test]
    fn test_to_verify_stripping_imports_work() {
        assert_transform("<div>1</div>", r#"newVNode(3, "div", null, "1");"#);
    }

    #[test]
    fn class_name_should_be_in_third_parameter_as_string_when_its_element() {
        assert_transform(
            r#"<div className="first second">1</div>"#,
            r#"newVNode(3, "div", "first second", "1");"#,
        );
    }

    #[test]
    fn class_name_should_be_in_fifth_parameter_as_string_when_its_component() {
        assert_transform(
            r#"<UnknownClass className="first second">1</UnknownClass>"#,
            r#"newComponentVNode(0, UnknownClass, {
  "className": "first second",
  children: "1"
});"#,
        );
    }

    #[test]
    fn jsxmember_expressions_should_work() {
        assert_transform(
            "<Components.Unknown>1</Components.Unknown>",
            r#"newComponentVNode(0, Components.Unknown, {
  children: "1"
});"#,
        );
    }

    #[test]
    fn class_should_be_in_third_parameter_as_variable() {
        assert_transform(
            "<div class={variable}>1</div>",
            r#"newVNode(3, "div", variable, "1");"#,
        );
    }

    #[test]
    fn should_call_create_vnode_twice_and_text_children() {
        assert_transform(
            r"<div>
          <div>single</div>
        </div>",
            r#"newVNode(9, "div", null, newVNode(3, "div", null, "single"));"#,
        );
    }

    #[test]
    fn events_should_be_in_props() {
        assert_transform(
            r#"<div id="test" onClick={func} class={variable}>1</div>"#,
            r#"newVNode(3, "div", variable, "1", {
  "id": "test",
  "onClick": func
});"#,
        );
    }

    #[test]
    fn should_transform_input_and_html_for_correctly() {
        assert_transform(
            r#"<label htmlFor={id}><input id={id} name={name} value={value} onChange={onChange} onInput={onInput} onKeyup={onKeyup} onFocus={onFocus} onClick={onClick} type="number" pattern="[0-9]+([,.][0-9]+)?" inputMode="numeric" min={minimum}/></label>"#,
            r#"newVNode(9, "label", null, newVNode(528, "input", null, null, {
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
}), {
  "for": id
});"#,
        );
    }

    #[test]
    fn should_transform_accept_charset_correctly() {
        assert_transform(
            r#"<form acceptCharset="ISO-8859-1"/>"#,
            r#"newVNode(17, "form", null, null, {
  "accept-charset": "ISO-8859-1"
});"#,
        );
    }

    #[test]
    fn should_lower_case_f_e_col_span() {
        assert_transform(
            r#"<td colSpan="5"/>"#,
            r#"newVNode(17, "td", null, null, {
  "colspan": "5"
});"#,
        );
    }

    #[test]
    fn should_transform_on_double_click_to_native_html_event() {
        assert_transform(
            "<div onDoubleClick={foobar}></div>",
            r#"newVNode(17, "div", null, null, {
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
            r#"newVNode(131089, "div", null, null, {
  "contentEditable": true
});"#,
        );
        assert_transform(
            r#"<span contenteditable="false"></span>"#,
            r#"newVNode(131089, "span", null, null, {
  "contenteditable": "false"
});"#,
        );
        assert_transform(
            "<div contenteditable></div>",
            r#"newVNode(131089, "div", null, null, {
  "contenteditable": true
});"#,
        );
        assert_transform(
            "<div contentEditable={logic}></div>",
            r#"newVNode(131089, "div", null, null, {
  "contentEditable": logic
});"#,
        );
        assert_transform(
            r#"<div contentEditable="true"></div>"#,
            r#"newVNode(131089, "div", null, null, {
  "contentEditable": "true"
});"#,
        );
    }
}

// Not ported from Pragma option:
// - Should replace newVNode to pragma option value (swc-plugin-inferno has no imports: false, pragma options)

// Not ported from defineAllArguments option:
// - Should replace newVNode to pragma option value (swc-plugin-inferno has no imports: false, defineAllArguments options)

/// SVG attributes React syntax support
mod svg_attributes_react_syntax_support {
    use super::*;

    #[test]
    fn should_support_native_xlink_href() {
        assert_transform(
            r##"<svg><use xlink:href="#tester"></use></svg>"##,
            r##"newVNode(72, "svg", null, newVNode(80, "use", null, null, {
  "xlink:href": "#tester"
}));"##,
        );
    }

    #[test]
    fn should_transform_xlink_href_to_xlink_href() {
        assert_transform(
            r##"<svg><use xlinkHref="#tester"></use></svg>"##,
            r##"newVNode(72, "svg", null, newVNode(80, "use", null, null, {
  "xlink:href": "#tester"
}));"##,
        );
    }

    #[test]
    fn should_transform_stroke_width_to_stroke_width() {
        assert_transform(
            r#"<svg><rect strokeWidth="1px"></rect></svg>"#,
            r#"newVNode(72, "svg", null, newVNode(80, "rect", null, null, {
  "stroke-width": "1px"
}));"#,
        );
    }

    #[test]
    fn should_transform_stroke_width_to_stroke_width_2() {
        assert_transform(
            r#"<svg><rect fillOpacity="1"></rect></svg>"#,
            r#"newVNode(72, "svg", null, newVNode(80, "rect", null, null, {
  "fill-opacity": "1"
}));"#,
        );
    }

    #[test]
    fn should_not_transform_stroke_with_or_other_svg_attributes_if_they_are_used_in_component() {
        assert_transform(
            r#"<Foobar strokeWidth="1px" fillOpacity="1"/>"#,
            r#"newComponentVNode(0, Foobar, {
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
            r#"newVNode(5, "div", null, [newTextVNode("Okay"), newVNode(3, "span", null, "foo")]);"#,
        );
    }

    #[test]
    fn should_create_text_vnode_when_text_node_is_under_short_syntax_fragment() {
        assert_transform(
            "<>Okay<span>foo</span></>",
            r#"newFragment(260, [newTextVNode("Okay"), newVNode(3, "span", null, "foo")]);"#,
        );
    }

    #[test]
    fn should_not_wrap_dynamic_value() {
        assert_transform("<>{magic}</>", "newFragment(256, magic);");
    }

    #[test]
    fn should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_short_syntax_fragment()
     {
        assert_transform(
            "<><>Text</></>",
            r#"newFragment(260, [newFragment(260, [newTextVNode("Text")])]);"#,
        );
    }

    #[test]
    fn should_always_short_syntax_fragment() {
        assert_transform(
            "<><><div>Text</div></></>",
            r#"newFragment(260, [newFragment(260, [newVNode(3, "div", null, "Text")])]);"#,
        );
    }

    #[test]
    fn should_handle_many_dynamic_children_short_syntax() {
        assert_transform(
            "<><>{Frag}Text{Wohoo}</></>",
            r#"newFragment(260, [newFragment(256, [Frag, newTextVNode("Text"), Wohoo])]);"#,
        );
    }

    #[test]
    fn should_handle_many_dynamic_and_non_dynamic_children_short_syntax() {
        assert_transform(
            "<><><span></span>Text{Wohoo}</></>",
            r#"newFragment(260, [newFragment(256, [newVNode(17, "span"), newTextVNode("Text"), Wohoo])]);"#,
        );
    }

    #[test]
    fn should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_long_syntax_fragment()
     {
        assert_transform(
            "<Fragment><Fragment>Text</Fragment></Fragment>",
            r#"newFragment(260, [newFragment(260, [newTextVNode("Text")])]);"#,
        );
    }

    #[test]
    fn should_create_text_vnode_when_text_node_is_under_large_syntax_fragment() {
        assert_transform(
            "<Fragment>Okay<span>foo</span></Fragment>",
            r#"newFragment(260, [newTextVNode("Okay"), newVNode(3, "span", null, "foo")]);"#,
        );
    }

    #[test]
    fn should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_long_syntax_fragment_2()
     {
        assert_transform(
            "<Fragment><Fragment>Text</Fragment></Fragment>",
            r#"newFragment(260, [newFragment(260, [newTextVNode("Text")])]);"#,
        );
    }

    #[test]
    fn should_always_long_syntax_fragment() {
        assert_transform(
            "<Fragment><Fragment><div>Text</div></Fragment></Fragment>",
            r#"newFragment(260, [newFragment(260, [newVNode(3, "div", null, "Text")])]);"#,
        );
    }

    #[test]
    fn should_handle_many_dynamic_children_long_syntax() {
        assert_transform(
            "<Fragment><Fragment>{Frag}Text{Wohoo}</Fragment></Fragment>",
            r#"newFragment(260, [newFragment(256, [Frag, newTextVNode("Text"), Wohoo])]);"#,
        );
    }

    #[test]
    fn should_handle_many_dynamic_and_non_dynamic_children_long_syntax() {
        assert_transform(
            "<Fragment><Fragment><span></span>Text{Wohoo}</Fragment></Fragment>",
            r#"newFragment(260, [newFragment(256, [newVNode(17, "span"), newTextVNode("Text"), Wohoo])]);"#,
        );
    }
}

/// Imports
mod imports {
    use super::*;

    #[test]
    fn should_not_fail_if_new_vnode_is_already_imported() {
        assert_js_eq(
            &transform_with(
                "{}",
                r#"import {newVNode} from "inferno"; var foo = <div/>;"#,
            ),
            r#"import { newVNode } from "inferno";
var foo = newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_add_import_to_new_component_vnode_but_not_to_new_vnode_if_new_vnode_is_already_delcared()
     {
        assert_js_eq(
            &transform_with(
                "{}",
                r#"import {newVNode} from "inferno"; var foo = <FooBar/>;"#,
            ),
            r#"import { newComponentVNode } from "inferno";
import { newVNode } from "inferno";
var foo = newComponentVNode(0, FooBar);"#,
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
            r#"newVNode(3, "div", null, "test");"#,
        );
    }

    #[test]
    fn element_should_prefer_prop_over_empty_children() {
        assert_transform(
            r#"<div children="ab"></div>"#,
            r#"newVNode(3, "div", null, "ab");"#,
        );
    }

    #[test]
    fn element_should_use_prop_if_no_children_exists() {
        assert_transform(
            r#"<div children="ab"/>"#,
            r#"newVNode(3, "div", null, "ab");"#,
        );
    }

    #[test]
    fn component_should_prefer_child_element_over_children_props() {
        assert_transform(
            r#"<Com children="ab">test</Com>"#,
            r#"newComponentVNode(0, Com, {
  children: "test"
});"#,
        );
    }

    #[test]
    fn component_should_prefer_prop_over_empty_children() {
        assert_transform(
            r#"<Com children="ab"></Com>"#,
            r#"newComponentVNode(0, Com, {
  "children": "ab"
});"#,
        );
    }

    #[test]
    fn component_should_use_prop_if_no_children_exists() {
        assert_transform(
            r#"<Com children="ab"/>"#,
            r#"newComponentVNode(0, Com, {
  "children": "ab"
});"#,
        );
    }

    #[test]
    fn component_array_empty_children() {
        assert_transform("<Com>{[]}</Com>", "newComponentVNode(0, Com);");
    }

    #[test]
    fn component_should_create_v_node_for_children() {
        assert_transform(
            "<Com children={<div>1</div>}/>",
            r#"newComponentVNode(0, Com, {
  "children": newVNode(3, "div", null, "1")
});"#,
        );
    }

    #[test]
    fn should_prefer_xml_children_over_props() {
        assert_transform(
            "<foo children={<span>b</span>}></foo>",
            r#"newVNode(9, "foo", null, newVNode(3, "span", null, "b"));"#,
        );
    }

    #[test]
    fn should_prefer_xml_children_over_props_null() {
        assert_transform("<foo children={null}></foo>", r#"newVNode(17, "foo");"#);
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
            assert_transform("<></>", "newFragment(272);");
        }

        #[test]
        fn should_create_fragment() {
            assert_transform("<>Test</>", r#"newFragment(260, [newTextVNode("Test")]);"#);
        }

        #[test]
        fn should_create_fragment_dynamic_children() {
            assert_transform("<>{dynamic}</>", "newFragment(256, dynamic);");
        }

        #[test]
        fn should_create_fragment_keyed_children() {
            assert_transform(
                r#"<><span key="ok">kk</span><div key="ok2">ok</div></>"#,
                r#"newFragment(288, [newVNode(3, "span", null, "kk", null, "ok"), newVNode(3, "div", null, "ok", null, "ok2")]);"#,
            );
        }

        #[test]
        fn should_create_fragment_non_keyed_children() {
            assert_transform(
                "<><div>1</div><span>foo</span></>",
                r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")]);"#,
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
                assert_transform("<Fragment></Fragment>", "newFragment(272);");
                assert_transform("<Fragment/>", "newFragment(272);");
            }

            #[test]
            fn should_create_fragment() {
                assert_transform(
                    "<Fragment>Test</Fragment>",
                    r#"newFragment(260, [newTextVNode("Test")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_dynamic_children() {
                assert_transform(
                    "<Fragment>{dynamic}</Fragment>",
                    "newFragment(256, dynamic);",
                );
            }

            #[test]
            fn should_create_fragment_keyed_children() {
                assert_transform(
                    r#"<Fragment><span key="ok">kk</span><div key="ok2">ok</div></Fragment>"#,
                    r#"newFragment(288, [newVNode(3, "span", null, "kk", null, "ok"), newVNode(3, "div", null, "ok", null, "ok2")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children() {
                assert_transform(
                    "<Fragment><div>1</div><span>foo</span></Fragment>",
                    r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_2() {
                assert_transform(
                    r#"<Fragment key="foo"><div>1</div><span>foo</span></Fragment>"#,
                    r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")], "foo");"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_3() {
                assert_transform(
                    r#"<Fragment key="foo" $HasKeyedChildren>{magic}</Fragment>"#,
                    r#"newFragment(288, magic, "foo");"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_4() {
                assert_transform(
                    r#"<Fragment key="foo" $HasNonKeyedChildren>{magic}</Fragment>"#,
                    r#"newFragment(260, magic, "foo");"#,
                );
            }
        }

        /// Inferno.Fragment
        mod inferno_fragment {
            use super::*;

            #[test]
            fn should_create_fragment() {
                assert_transform(
                    "<Inferno.Fragment>Test</Inferno.Fragment>",
                    r#"newFragment(260, [newTextVNode("Test")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_dynamic_children() {
                assert_transform(
                    "<Inferno.Fragment>{dynamic}</Inferno.Fragment>",
                    "newFragment(256, dynamic);",
                );
            }

            #[test]
            fn should_create_fragment_keyed_children() {
                assert_transform(
                    r#"<Inferno.Fragment><span key="ok">kk</span><div key="ok2">ok</div></Inferno.Fragment>"#,
                    r#"newFragment(288, [newVNode(3, "span", null, "kk", null, "ok"), newVNode(3, "div", null, "ok", null, "ok2")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children() {
                assert_transform(
                    "<Inferno.Fragment><div>1</div><span>foo</span></Inferno.Fragment>",
                    r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_2() {
                assert_transform(
                    r#"<Inferno.Fragment key="foo"><div>1</div><span>foo</span></Inferno.Fragment>"#,
                    r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")], "foo");"#,
                );
            }

            #[test]
            fn should_ignore_all_other_props() {
                assert_transform(
                    r#"<Inferno.Fragment abc="foobar" id="test" key="foo"><div>1</div><span>foo</span></Inferno.Fragment>"#,
                    r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")], "foo");"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_3() {
                assert_transform(
                    r#"<Inferno.Fragment key="foo" $HasKeyedChildren>{magic}</Inferno.Fragment>"#,
                    r#"newFragment(288, magic, "foo");"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_4() {
                assert_transform(
                    r#"<Inferno.Fragment key="foo" $HasNonKeyedChildren>{magic}</Inferno.Fragment>"#,
                    r#"newFragment(260, magic, "foo");"#,
                );
            }
        }

        /// React.Fragment
        mod react_fragment {
            use super::*;

            #[test]
            fn should_create_fragment() {
                assert_transform(
                    "<React.Fragment>Test</React.Fragment>",
                    r#"newFragment(260, [newTextVNode("Test")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_dynamic_children() {
                assert_transform(
                    "<React.Fragment>{dynamic}</React.Fragment>",
                    "newFragment(256, dynamic);",
                );
            }

            #[test]
            fn should_create_fragment_keyed_children() {
                assert_transform(
                    r#"<React.Fragment><span key="ok">kk</span><div key="ok2">ok</div></React.Fragment>"#,
                    r#"newFragment(288, [newVNode(3, "span", null, "kk", null, "ok"), newVNode(3, "div", null, "ok", null, "ok2")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children() {
                assert_transform(
                    "<React.Fragment><div>1</div><span>foo</span></React.Fragment>",
                    r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")]);"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_2() {
                assert_transform(
                    r#"<React.Fragment key="foo"><div>1</div><span>foo</span></React.Fragment>"#,
                    r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")], "foo");"#,
                );
            }

            #[test]
            fn should_ignore_all_other_props() {
                assert_transform(
                    r#"<React.Fragment abc="foobar" id="test" key="foo"><div>1</div><span>foo</span></React.Fragment>"#,
                    r#"newFragment(260, [newVNode(3, "div", null, "1"), newVNode(3, "span", null, "foo")], "foo");"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_3() {
                assert_transform(
                    r#"<React.Fragment key="foo" $HasKeyedChildren>{magic}</React.Fragment>"#,
                    r#"newFragment(288, magic, "foo");"#,
                );
            }

            #[test]
            fn should_create_fragment_non_keyed_children_4() {
                assert_transform(
                    r#"<React.Fragment key="foo" $HasNonKeyedChildren>{magic}</React.Fragment>"#,
                    r#"newFragment(260, magic, "foo");"#,
                );
            }
        }
    }
}
