//! Ported from babel-plugin-inferno `tests/spread.test.js`: Spread attributes

use crate::helpers::*;

/// spread position
mod spread_position {
    use super::*;

    #[test]
    fn should_keep_a_spread_before_other_props() {
        assert_transform(
            "<Component {...x} y={2} z />",
            r#"normalizeProps(createComponentVNode(2, Component, {
  ...x,
  "y": 2,
  "z": true
}));"#,
        );
    }

    #[test]
    fn should_keep_a_spread_after_other_props() {
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
    fn should_keep_a_spread_between_other_props() {
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
    fn should_keep_the_same_spread_twice() {
        assert_transform(
            r#"<Component x={1} y="2" {...z} {...z}><Child /></Component>"#,
            r#"normalizeProps(createComponentVNode(2, Component, {
  "x": 1,
  "y": "2",
  ...z,
  ...z,
  children: createComponentVNode(2, Child)
}));"#,
        );
    }

    #[test]
    fn should_keep_a_sequence_expression_spread() {
        assert_transform(
            r#"<Component x="1" {...(z = { y: 2 }, z)} z={3}>Text</Component>"#,
            r#"normalizeProps(createComponentVNode(2, Component, {
  "x": "1",
  ...(z = {
    y: 2
  }, z),
  "z": 3,
  children: "Text"
}));"#,
        );
    }

    #[test]
    fn should_keep_a_null_spread() {
        assert_transform(
            "<div {...null} />",
            r#"normalizeProps(createVNode(1, "div", null, null, 1, {
  ...null
}));"#,
        );
    }

    #[test]
    fn should_keep_a_spread_of_a_jsx_element() {
        assert_transform(
            "<div {...<span/>} />",
            r#"normalizeProps(createVNode(1, "div", null, null, 1, {
  ...createVNode(1, "span")
}));"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno flattens an object literal spread containing __proto__, which sets the prototype of the props"]
    fn should_keep_an_object_literal_spread_containing_proto() {
        assert_transform(
            r#"<Foo {...{__proto__: a}} b="1" />"#,
            r#"normalizeProps(createComponentVNode(2, Foo, {
  ...{
    __proto__: a
  },
  "b": "1"
}));"#,
        );
    }

    // babel-plugin-inferno keeps the object literal spread; swc-plugin-inferno flattens it into the props,
    // which creates the same props.
    #[test]
    fn should_keep_a_comment_inside_a_spread() {
        assert_transform(
            r#"<div {.../*i18n*/{ id: "hello" }} />"#,
            r#"normalizeProps(createVNode(1, "div", null, null, 1, {
    id: "hello"
}));"#,
        );
    }
}

/// spread with special props
mod spread_with_special_props {
    use super::*;

    #[test]
    fn should_keep_class_name_key_and_ref_next_to_spreads() {
        assert_transform(
            r#"<div {...a} {...b} className="x" key="k" ref={r}>{c}</div>"#,
            r#"normalizeProps(createVNode(1, "div", "x", c, 0, {
  ...a,
  ...b
}, "k", r));"#,
        );
    }

    #[test]
    fn should_keep_key_next_to_a_spread_on_a_component() {
        assert_transform(
            r#"<Foo {...p} key="k"/>"#,
            r#"normalizeProps(createComponentVNode(2, Foo, {
  ...p
}, "k"));"#,
        );
    }

    #[test]
    fn should_keep_ref_next_to_a_spread_on_a_component() {
        assert_transform(
            "<Foo {...p} ref={r}/>",
            r"normalizeProps(createComponentVNode(2, Foo, {
  ...p
}, null, r));",
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno marks a children prop next to a spread as invalid children"]
    fn should_keep_a_children_prop_next_to_a_spread_on_an_element() {
        assert_transform(
            r#"<div {...p} children="x"/>"#,
            r#"normalizeProps(createVNode(1, "div", null, "x", 16, {
  ...p
}));"#,
        );
    }

    #[test]
    fn should_keep_dynamic_children_next_to_a_spread() {
        assert_transform(
            "<div {...p}>{a}{b}</div>",
            r#"normalizeProps(createVNode(1, "div", null, [a, b], 0, {
  ...p
}));"#,
        );
    }

    #[test]
    #[ignore = "swc-plugin-inferno moves the children prop after the spread, so the spread cannot override it"]
    fn should_keep_a_component_children_prop_next_to_a_spread() {
        assert_transform(
            "<Foo children={a} {...p} />",
            r#"normalizeProps(createComponentVNode(2, Foo, {
  "children": a,
  ...p
}));"#,
        );
    }

    #[test]
    fn should_prefer_jsx_children_over_a_children_prop_next_to_a_spread() {
        assert_transform(
            "<Foo {...p} children={a}>b</Foo>",
            r#"normalizeProps(createComponentVNode(2, Foo, {
  ...p,
  children: "b"
}));"#,
        );
    }

    #[test]
    fn should_keep_an_attribute_after_a_spread() {
        assert_transform(
            r#"<input {...props} type="radio" />"#,
            r#"normalizeProps(createVNode(64, "input", null, null, 1, {
  ...props,
  "type": "radio"
}));"#,
        );
    }

    #[test]
    fn should_keep_a_spread_of_a_conditional_object() {
        assert_transform(
            r#"<div {...(c ? {class: "x"} : {})} />"#,
            r#"normalizeProps(createVNode(1, "div", null, null, 1, {
  ...(c ? {
    class: "x"
  } : {})
}));"#,
        );
    }
}

/// other compilation targets
mod other_compilation_targets {
    use super::*;

    #[test]
    fn should_compile_spreads_with_object_spread_for_es5() {
        assert_js_eq(
            &transform_with("{}", r#"<div {...p} a="1"/>"#),
            r#"import { createVNode, normalizeProps } from "inferno";
normalizeProps(createVNode(1, "div", null, null, 1, {
  ...p,
  "a": "1"
}));"#,
        );
    }

    #[test]
    fn should_reference_inferno_helpers_through_the_module_namespace_for_common_js() {
        assert_js_eq(
            &transform_with("{}", "<div {...p}/>"),
            r#"import { createVNode, normalizeProps } from "inferno";
normalizeProps(createVNode(1, "div", null, null, 1, {
  ...p
}));"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    // babel-plugin-inferno keeps the object literal spread; swc-plugin-inferno flattens it into the props,
    // which creates the same props.
    #[test]
    fn should_flatten_an_object_literal_spread() {
        assert_transform(
            "<Foo {...{a: 1}} />",
            r"normalizeProps(createComponentVNode(2, Foo, {
    a: 1
}));",
        );
    }

    #[test]
    fn should_compile_a_key_before_a_spread_like_a_key_after_it() {
        assert_js_eq(
            &transform(r#"<Foo key="k" {...p}/>"#),
            &transform(r#"<Foo {...p} key="k"/>"#),
        );
        assert_transform(
            r#"<Foo key="k" {...p}/>"#,
            r#"normalizeProps(createComponentVNode(2, Foo, {
  ...p
}, "k"));"#,
        );
    }

    #[test]
    fn should_pass_class_name_before_a_spread_as_the_class_name_argument() {
        assert_transform(
            r#"<div className="x" {...p}/>"#,
            r#"normalizeProps(createVNode(1, "div", "x", null, 1, {
  ...p
}));"#,
        );
    }

    #[test]
    fn should_evaluate_class_name_before_the_other_props() {
        assert_transform(
            "<div onClick={a()} className={b()} key={c()} ref={d()} />",
            r#"createVNode(1, "div", b(), null, 1, {
  "onClick": a()
}, c(), d());"#,
        );
    }
}
