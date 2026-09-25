//! Ported from babel-plugin-inferno `tests/key-ref-hooks.test.js`: key, ref and onComponent hooks

use crate::helpers::*;

/// ref
mod ref_ {
    use super::*;

    #[test]
    fn should_pass_ref_to_an_element() {
        assert_transform(
            "<div ref={a} />",
            r#"createVNode(1, "div", null, null, 1, null, null, a);"#,
        );
    }

    #[test]
    fn should_pass_ref_and_key_to_an_element() {
        assert_transform(
            r#"<div ref={a} key="k" />"#,
            r#"createVNode(1, "div", null, null, 1, null, "k", a);"#,
        );
    }

    #[test]
    fn should_pass_ref_to_an_element_with_dynamic_children() {
        assert_transform(
            "<div ref={a}>{b}</div>",
            r#"createVNode(1, "div", null, b, 0, null, null, a);"#,
        );
    }

    #[test]
    fn should_pass_ref_to_a_component() {
        assert_transform(
            "<Foo ref={r} />",
            "createComponentVNode(2, Foo, null, null, r);",
        );
    }

    #[test]
    fn should_pass_key_and_ref_to_a_component() {
        assert_transform(
            r#"<Foo key="k" ref={r} />"#,
            r#"createComponentVNode(2, Foo, null, "k", r);"#,
        );
    }

    #[test]
    fn should_pass_a_string_ref() {
        assert_transform(
            r#"<div ref="stringRef" />"#,
            r#"createVNode(1, "div", null, null, 1, null, null, "stringRef");"#,
        );
    }

    #[test]
    fn should_pass_null_key_and_ref() {
        assert_transform(
            "<div key={null} ref={null} />",
            r#"createVNode(1, "div", null, null, 1, null, null, null);"#,
        );
    }

    #[test]
    fn should_pass_ref_and_other_props_to_a_component() {
        assert_transform(
            r#"<Component ref={ref} foo="56" />"#,
            r#"createComponentVNode(2, Component, {
  "foo": "56"
}, null, ref);"#,
        );
    }

    #[test]
    fn should_pass_every_argument_to_a_component() {
        assert_transform(
            r#"<Parent a="a" b={{b: "b"}} c={C} key="testKey" ref={testRef} />"#,
            r#"createComponentVNode(2, Parent, {
  "a": "a",
  "b": {
    b: "b"
  },
  "c": C
}, "testKey", testRef);"#,
        );
    }
}

/// key values
mod key_values {
    use super::*;

    #[test]
    fn should_pass_an_undefined_key() {
        assert_transform(
            "<div key={undefined} />",
            r#"createVNode(1, "div", null, null, 1, null, undefined);"#,
        );
    }

    #[test]
    fn should_pass_numeric_and_empty_string_keys() {
        assert_transform(
            r#"<div key={0}><a key=""/><b key="x"/></div>"#,
            r#"createVNode(1, "div", null, [createVNode(1, "a", null, null, 1, null, ""), createVNode(1, "b", null, null, 1, null, "x")], 8, null, 0);"#,
        );
    }

    #[test]
    fn should_pass_an_object_key() {
        assert_transform(
            "<div key={obj} />",
            r#"createVNode(1, "div", null, null, 1, null, obj);"#,
        );
    }

    #[test]
    fn should_reject_a_valueless_key_on_an_element() {
        assert_transform_error(
            "<div key />",
            r#"Please provide an explicit key value. Using "key" as a shorthand for "key={true}" is not allowed."#,
        );
    }

    #[test]
    fn should_reject_a_valueless_key_on_a_component() {
        assert_transform_error(
            "<Foo key />",
            r#"Please provide an explicit key value. Using "key" as a shorthand for "key={true}" is not allowed."#,
        );
    }

    #[test]
    fn should_reject_a_valueless_key_inside_an_array_babel_should_disallow_valueless_key() {
        assert_transform_error(
            "[<div key></div>]",
            r#"Please provide an explicit key value. Using "key" as a shorthand for "key={true}" is not allowed."#,
        );
    }

    #[test]
    fn should_point_the_valueless_key_error_at_the_key() {
        assert_transform_error(
            r"<ul>
  <li key>a</li>
</ul>",
            r" 2 |   <li key>a</li>
   :       ^^^",
        );
    }
}

/// keyed children
mod keyed_children {
    use super::*;

    #[test]
    fn should_mark_mixed_keyed_and_unkeyed_children_as_keyed() {
        assert_transform(
            r#"<div><span key="k"/><span/></div>"#,
            r#"createVNode(1, "div", null, [createVNode(1, "span", null, null, 1, null, "k"), createVNode(1, "span")], 8);"#,
        );
    }

    #[test]
    fn should_mark_a_single_keyed_child_as_a_vnode_child() {
        assert_transform(
            r#"<div><span key="k"/></div>"#,
            r#"createVNode(1, "div", null, createVNode(1, "span", null, null, 1, null, "k"), 2);"#,
        );
    }

    #[test]
    fn should_mark_duplicate_sibling_keys_as_keyed() {
        assert_transform(
            r#"<><a key="a"/><a key="a"/></>"#,
            r#"createFragment([createVNode(1, "a", null, null, 1, null, "a"), createVNode(1, "a", null, null, 1, null, "a")], 8);"#,
        );
    }

    #[test]
    fn should_not_mark_children_as_keyed_when_normalization_is_needed() {
        assert_transform(
            r#"<div>{a}<span key="k"/></div>"#,
            r#"createVNode(1, "div", null, [a, createVNode(1, "span", null, null, 1, null, "k")], 0);"#,
        );
    }

    #[test]
    fn should_not_inspect_keys_of_component_children() {
        assert_transform(
            r#"<Foo key="a"><Bar key="b"/><Bar key="c"/></Foo>"#,
            r#"createComponentVNode(2, Foo, {
  children: [createComponentVNode(2, Bar, null, "b"), createComponentVNode(2, Bar, null, "c")]
}, "a");"#,
        );
    }

    #[test]
    fn should_not_detect_keys_passed_through_spread() {
        assert_transform(
            r#"<div><Foo {...{key: "k"}}/><Foo {...{key: "j"}}/></div>"#,
            r#"createVNode(1, "div", null, [normalizeProps(createComponentVNode(2, Foo, {
  ...{
    key: "k"
  }
})), normalizeProps(createComponentVNode(2, Foo, {
  ...{
    key: "j"
  }
}))], 4);"#,
        );
    }
}

/// onComponent hooks
mod on_component_hooks {
    use super::*;

    #[test]
    fn should_move_every_on_component_hook_into_ref() {
        assert_transform(
            "<Foo onComponentWillMount={a} onComponentWillUnmount={b} onComponentShouldUpdate={c} onComponentWillUpdate={d} onComponentDidUpdate={e} />",
            r#"createComponentVNode(2, Foo, null, null, {
  "onComponentWillMount": a,
  "onComponentWillUnmount": b,
  "onComponentShouldUpdate": c,
  "onComponentWillUpdate": d,
  "onComponentDidUpdate": e
});"#,
        );
    }

    #[test]
    fn should_move_hooks_into_ref_for_member_expression_components() {
        assert_transform(
            "<Foo.Bar onComponentDidMount={a} />",
            r#"createComponentVNode(2, Foo.Bar, null, null, {
  "onComponentDidMount": a
});"#,
        );
    }

    #[test]
    fn should_keep_hooks_as_props_on_elements() {
        assert_transform(
            "<div onComponentDidMount={f} />",
            r#"createVNode(1, "div", null, null, 1, {
  "onComponentDidMount": f
});"#,
        );
    }

    #[test]
    fn should_merge_ref_into_the_hooks_when_ref_comes_before_a_hook() {
        assert_transform(
            "<Foo ref={r} onComponentDidMount={m} />",
            r#"createComponentVNode(2, Foo, null, null, {
  ...r,
  "onComponentDidMount": m
});"#,
        );
    }

    #[test]
    fn should_merge_ref_into_the_hooks_when_ref_comes_after_the_hooks() {
        assert_transform(
            "<Foo onComponentDidMount={m} ref={r} />",
            r#"createComponentVNode(2, Foo, null, null, {
  ...r,
  "onComponentDidMount": m
});"#,
        );
    }

    #[test]
    fn should_compile_ref_and_hooks_the_same_in_any_order() {
        assert_js_eq(
            &transform("<Foo ref={r} onComponentDidMount={m} />"),
            &transform("<Foo onComponentDidMount={m} ref={r} />"),
        );
    }

    #[test]
    fn should_merge_ref_with_several_hooks_key_and_children() {
        assert_transform(
            "<Foo key={i} ref={r} onComponentDidAppear={a} onComponentDidMount={b}>{i}</Foo>",
            r#"createComponentVNode(2, Foo, {
  children: i
}, i, {
  ...r,
  "onComponentDidAppear": a,
  "onComponentDidMount": b
});"#,
        );
    }

    #[test]
    fn should_move_hooks_next_to_spread_props() {
        assert_transform(
            "<Foo {...p} onComponentDidMount={m} />",
            r#"normalizeProps(createComponentVNode(2, Foo, {
  ...p
}, null, {
  "onComponentDidMount": m
}));"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    #[test]
    fn should_pass_true_as_ref_for_a_valueless_ref() {
        assert_transform(
            "<div ref />",
            r#"createVNode(1, "div", null, null, 1, null, null, true);"#,
        );
    }
}
