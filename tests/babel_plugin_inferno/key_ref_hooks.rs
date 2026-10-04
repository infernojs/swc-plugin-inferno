//! Ported from babel-plugin-inferno `tests/key-ref-hooks.test.js`: key, ref and onComponent hooks

use crate::helpers::*;

/// ref
mod ref_ {
    use super::*;

    #[test]
    fn should_pass_ref_to_an_element() {
        assert_transform(
            "<div ref={a} />",
            r#"newVNode(17, "div", null, null, null, null, a);"#,
        );
    }

    #[test]
    fn should_pass_ref_and_key_to_an_element() {
        assert_transform(
            r#"<div ref={a} key="k" />"#,
            r#"newVNode(17, "div", null, null, null, "k", a);"#,
        );
    }

    #[test]
    fn should_pass_ref_to_an_element_with_dynamic_children() {
        assert_transform(
            "<div ref={a}>{b}</div>",
            r#"newVNode(1, "div", null, b, null, null, a);"#,
        );
    }

    #[test]
    fn should_pass_ref_to_a_component() {
        assert_transform(
            "<Foo ref={r} />",
            "newComponentVNode(0, Foo, null, null, r);",
        );
    }

    #[test]
    fn should_pass_key_and_ref_to_a_component() {
        assert_transform(
            r#"<Foo key="k" ref={r} />"#,
            r#"newComponentVNode(0, Foo, null, "k", r);"#,
        );
    }

    #[test]
    fn should_pass_a_string_ref() {
        assert_transform(
            r#"<div ref="stringRef" />"#,
            r#"newVNode(17, "div", null, null, null, null, "stringRef");"#,
        );
    }

    #[test]
    fn should_omit_null_key_and_ref() {
        assert_transform("<div key={null} ref={null} />", r#"newVNode(17, "div");"#);
        assert_transform(
            "<Foo key={null} ref={null} />",
            "newComponentVNode(0, Foo);",
        );
    }

    #[test]
    fn should_pass_ref_and_other_props_to_a_component() {
        assert_transform(
            r#"<Component ref={ref} foo="56" />"#,
            r#"newComponentVNode(0, Component, {
  "foo": "56"
}, null, ref);"#,
        );
    }

    #[test]
    fn should_pass_every_argument_to_a_component() {
        assert_transform(
            r#"<Parent a="a" b={{b: "b"}} c={C} key="testKey" ref={testRef} />"#,
            r#"newComponentVNode(0, Parent, {
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
            r#"newVNode(17, "div", null, null, null, undefined);"#,
        );
    }

    #[test]
    fn should_pass_numeric_and_empty_string_keys() {
        assert_transform(
            r#"<div key={0}><a key=""/><b key="x"/></div>"#,
            r#"newVNode(33, "div", null, [newVNode(17, "a", null, null, null, ""), newVNode(17, "b", null, null, null, "x")], null, 0);"#,
        );
    }

    #[test]
    fn should_pass_an_object_key() {
        assert_transform(
            "<div key={obj} />",
            r#"newVNode(17, "div", null, null, null, obj);"#,
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
            r#"newVNode(33, "div", null, [newVNode(17, "span", null, null, null, "k"), newVNode(17, "span")]);"#,
        );
    }

    #[test]
    fn should_mark_a_single_keyed_child_as_a_vnode_child() {
        assert_transform(
            r#"<div><span key="k"/></div>"#,
            r#"newVNode(9, "div", null, newVNode(17, "span", null, null, null, "k"));"#,
        );
    }

    #[test]
    fn should_mark_duplicate_sibling_keys_as_keyed() {
        assert_transform(
            r#"<><a key="a"/><a key="a"/></>"#,
            r#"newFragment(288, [newVNode(17, "a", null, null, null, "a"), newVNode(17, "a", null, null, null, "a")]);"#,
        );
    }

    #[test]
    fn should_not_mark_children_as_keyed_when_normalization_is_needed() {
        assert_transform(
            r#"<div>{a}<span key="k"/></div>"#,
            r#"newVNode(1, "div", null, [a, newVNode(17, "span", null, null, null, "k")]);"#,
        );
    }

    #[test]
    fn should_not_inspect_keys_of_component_children() {
        assert_transform(
            r#"<Foo key="a"><Bar key="b"/><Bar key="c"/></Foo>"#,
            r#"newComponentVNode(0, Foo, {
  children: [newComponentVNode(0, Bar, null, "b"), newComponentVNode(0, Bar, null, "c")]
}, "a");"#,
        );
    }

    #[test]
    fn should_not_detect_keys_passed_through_spread() {
        assert_transform(
            r#"<div><Foo {...{key: "k"}}/><Foo {...{key: "j"}}/></div>"#,
            r#"newVNode(5, "div", null, [normalizeProps(newComponentVNode(0, Foo, {
  ...{
    key: "k"
  }
})), normalizeProps(newComponentVNode(0, Foo, {
  ...{
    key: "j"
  }
}))]);"#,
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
            r#"newComponentVNode(0, Foo, null, null, {
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
            r#"newComponentVNode(0, Foo.Bar, null, null, {
  "onComponentDidMount": a
});"#,
        );
    }

    #[test]
    fn should_keep_hooks_as_props_on_elements() {
        assert_transform(
            "<div onComponentDidMount={f} />",
            r#"newVNode(17, "div", null, null, {
  "onComponentDidMount": f
});"#,
        );
    }

    #[test]
    fn should_merge_ref_into_the_hooks_when_ref_comes_before_a_hook() {
        assert_transform(
            "<Foo ref={r} onComponentDidMount={m} />",
            r#"newComponentVNode(0, Foo, null, null, {
  ...r,
  "onComponentDidMount": m
});"#,
        );
    }

    #[test]
    fn should_merge_ref_into_the_hooks_when_ref_comes_after_the_hooks() {
        assert_transform(
            "<Foo onComponentDidMount={m} ref={r} />",
            r#"newComponentVNode(0, Foo, null, null, {
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
            r#"newComponentVNode(0, Foo, {
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
            r#"normalizeProps(newComponentVNode(0, Foo, {
  ...p
}, null, {
  "onComponentDidMount": m
}));"#,
        );
    }
}

/// valueless ref
mod valueless_ref {
    use super::*;

    const MESSAGE: &str = r#"Please provide an explicit ref value. Using "ref" as a shorthand for "ref={true}" is not allowed."#;

    #[test]
    fn should_reject_a_valueless_ref_on_an_element() {
        assert_transform_error("<div ref />", MESSAGE);
    }

    #[test]
    fn should_reject_a_valueless_ref_on_a_component() {
        assert_transform_error("<Foo ref />", MESSAGE);
    }

    #[test]
    fn should_reject_a_valueless_ref_next_to_component_hooks() {
        assert_transform_error("<Foo ref onComponentDidMount={m} />", MESSAGE);
    }

    #[test]
    fn should_reject_a_valueless_ref_on_a_generic_component() {
        assert_compile_error(
            Setup {
                lang: Lang::Tsx,
                ..Default::default()
            },
            "<Foo<string> ref />",
            MESSAGE,
        );
    }

    // babel-plugin-inferno prints its code frame. swc prints its own, which points at the same
    // code.
    #[test]
    fn should_point_the_valueless_ref_error_at_the_ref() {
        let error = compile(Setup::default(), "<ul>\n  <li ref>a</li>\n</ul>").unwrap_err();

        assert!(error.contains(MESSAGE), "{error}");
        assert!(
            error.contains(" 2 |   <li ref>a</li>\n   :       ^^^\n"),
            "{error}"
        );
    }
}
