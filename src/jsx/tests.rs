#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
};
use swc_core::common::chain;
use swc_core::common::input::StringInput;
use swc_core::ecma::transforms::base::fixer::fixer;
use swc_core::ecma::transforms::base::hygiene::hygiene;
use swc_core::ecma::transforms::base::resolver;
use swc_core::ecma::visit::FoldWith;
use swc_ecma_codegen::{Config, Emitter};
use swc_ecma_parser::{EsConfig, Parser, Syntax};
use swc_ecma_transforms_compat::es3::property_literals;
use swc_ecma_transforms_module::common_js::common_js;
use swc_ecma_transforms_testing::{parse_options, test, test_fixture, FixtureTestConfig, Tester};
use testing::NormalizedOutput;

use super::*;
use crate::{inferno, pure_annotations, PluginDiagnosticsEmitter};

test!(
    ::swc_ecma_parser::Syntax::Typescript(::swc_ecma_parser::TsConfig {
        tsx: true,
        ..Default::default()
    }),
    |t| {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        chain!(
            resolver(unresolved_mark, top_level_mark, false),
            jsx(
                Some(t.comments.clone()),
                Default::default(),
                unresolved_mark
            ),
            common_js(
                unresolved_mark,
                Default::default(),
                Default::default(),
                Some(t.comments.clone())
            )
        )
    },
    should_always_stick_the_create_vnode_ref_to_import_when_compiled_to_commonjs,
    r#"
import {
  Component,
  createTextVNode,
  createVNode,
  linkEvent,
  render,
} from 'inferno';

const Foo = class Clock extends Component {
  public render() {
    return (
      <Collapsible>
        <div>
          {[<p>Hello 0</p>, <p>Hello 1</p>]}
          <strong>Hello 2</strong>
        </div>
        <p>Hello 3</p>
      </Collapsible>
    );
  }
}
render(<Foo/>, null);
"#,
    r#"
            "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _inferno = require("inferno");
const Foo = class Clock extends _inferno.Component {
    public render() {
        return (0, _inferno.createComponentVNode)(2, Collapsible, {
            children: [
                (0, _inferno.createVNode)(1, "div", null, [
                    [
                        (0, _inferno.createVNode)(1, "p", null, "Hello 0", 16),
                        (0, _inferno.createVNode)(1, "p", null, "Hello 1", 16)
                    ],
                    (0, _inferno.createVNode)(1, "strong", null, "Hello 2", 16)
                ], 0),
                (0, _inferno.createVNode)(1, "p", null, "Hello 3", 16)
            ]
        });
    }
};
(0, _inferno.render)((0, _inferno.createComponentVNode)(2, Foo), null);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Typescript(::swc_ecma_parser::TsConfig {
        tsx: true,
        ..Default::default()
    }),
    |t| {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        chain!(
            resolver(unresolved_mark, top_level_mark, false),
            jsx(
                Some(t.comments.clone()),
                Default::default(),
                unresolved_mark
            ),
        )
    },
    should_always_stick_the_create_vnode_ref_to_import_when_compiled_to_esm,
    r#"
import {
  Component,
  createTextVNode,
  createVNode,
  linkEvent,
  render,
} from 'inferno';

const Foo = class Clock extends Component {
  public render() {
    return (
      <Collapsible>
        <div>
          {[<p>Hello 0</p>, <p>Hello 1</p>]}
          <strong>Hello 2</strong>
        </div>
        <p>Hello 3</p>
      </Collapsible>
    );
  }
}
render(<Foo/>, null);
"#,
    r#"
import { Component, createTextVNode, createVNode, linkEvent, render, createComponentVNode } from 'inferno';
const Foo = class Clock extends Component {
    public render() {
        return createComponentVNode(2, Collapsible, {
            children: [
                createVNode(1, "div", null, [
                    [
                        createVNode(1, "p", null, "Hello 0", 16),
                        createVNode(1, "p", null, "Hello 1", 16)
                    ],
                    createVNode(1, "strong", null, "Hello 2", 16)
                ], 0),
                createVNode(1, "p", null, "Hello 3", 16)
            ]
        });
    }
};
render(createComponentVNode(2, Foo), null);
"#
);

/*
 * REFs
 */

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_functional_component_hooks_to_refs,
    r#"
<Child
key={i}
onComponentDidAppear={childOnComponentDidAppear}
onComponentDidMount={childOnComponentDidMount}
>
{i}
</Child>
"#,
    r#"
import { createComponentVNode } from "inferno";
createComponentVNode(2, Child, {
  children: i
}, i, {
  onComponentDidAppear: childOnComponentDidAppear,
  onComponentDidMount: childOnComponentDidMount
});
"#
);

/*
 * Dynamic children
 */
test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_normalize_call_when_there_is_dynamic_children,
    r#"
<div>{a}</div>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "div", null, a, 0);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_normalize_call_when_there_is_dynamic_and_static_children_mixed,
    r#"
<div>{a}<div>1</div></div>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "div", null, [a, createVNode(1, "div", null, "1", 16)], 0);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_support_native_xlink_href,
    r#"
<svg focusable="false" className={'test'}>
    <use xlink:href="asd"/>
</svg>
"#,
    r#"
import { createVNode } from "inferno";
createVNode(32, "svg", 'test', createVNode(32, "use", null, null, 1, { "xlink:href": "asd" }), 2, { focusable: "false" });
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_not_add_normalize_call_when_all_children_are_known,
    r#"
<div><FooBar/><div>1</div></div>
"#,
    r#"
    import { createVNode, createComponentVNode } from "inferno";
createVNode(1, "div", null, [createComponentVNode(2, FooBar), createVNode(1, "div", null, "1", 16)], 4);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_not_convert_text_to_create_vnode_when_its_within_component,
    r#"
<FooBar>1</FooBar>
"#,
    r#"
    import { createComponentVNode } from "inferno";
createComponentVNode(2, FooBar, {
  children: "1"
});
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_create_text_vnodes_when_there_is_no_normalization_needed_and_its_multiple_children,
    r#"
<div><FooBar/>foobar</div>
"#,
    r#"
    import { createVNode, createComponentVNode, createTextVNode } from "inferno";
createVNode(1, "div", null, [createComponentVNode(2, FooBar), createTextVNode("foobar")], 4);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_create_text_vnodes_when_there_is_single_children,
    r#"
<div>foobar</div>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "div", null, "foobar", 16);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_create_text_vnodes_when_there_is_single_children_2,
    r#"
<div>1</div>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "div", null, "1", 16);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_not_normalize_component_prop_children,
    r#"
<Com>{a}</Com>
"#,
    r#"
    import { createComponentVNode } from "inferno";
createComponentVNode(2, Com, {
  children: a
});"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_not_normalize_component_children_as_they_are_in_props,
    r#"
<Com>{a}{b}{c}</Com>
"#,
    r#"
    import { createComponentVNode } from "inferno";
createComponentVNode(2, Com, {
  children: [a, b, c]
});"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_mark_parent_vnode_with_has_non_keyed_children_if_no_normalize_is_needed_and_all_children_are_non_keyed,
    r#"
<div><FooBar/><div>1</div></div>
"#,
    r#"
    import { createVNode, createComponentVNode } from "inferno";
createVNode(1, "div", null, [createComponentVNode(2, FooBar), createVNode(1, "div", null, "1", 16)], 4);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_mark_parent_vnode_with_has_keyed_children_if_no_normalize_is_needed_and_all_children_are_keyed,
    r#"
<div><FooBar key="foo"/><div key="1">1</div></div>
"#,
    r#"
    import { createVNode, createComponentVNode } from "inferno";
createVNode(1, "div", null, [createComponentVNode(2, FooBar, null, "foo"), createVNode(1, "div", null, "1", 16, null, "1")], 8);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_mark_parent_vnode_with_has_keyed_children_if_even_one_child_is_keyed_directly,
    r#"
<div><span></span><div key="1">1</div></div>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "div", null, [createVNode(1, "span"), createVNode(1, "div", null, "1", 16, null, "1")], 8);"#
);

/*
 * Dynamic ChildFlags
 */

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_be_possible_to_define_override_child_flags_runtime_for_dynamic_children,
    r#"
<img $ChildFlag={bool ? 1 : 2}>{expression}</img>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "img", null, expression, bool ? 1 : 2);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_be_possible_to_define_override_child_flags_runtime,
    r#"
<img $ChildFlag={1}>foobar</img>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "img", null, "foobar", 1);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_be_possible_to_use_expression_for_child_flags,
    r#"
<img $ChildFlag={magic}>foobar</img>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "img", null, "foobar", magic);"#
);

/*
 * different types
 */

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_img,
    r#"
<img>foobar</img>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "img", null, "foobar", 16);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_br,
    r#"
<br>foobar</br>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "br", null, "foobar", 16);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_media,
    r#"
<media>foobar</media>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "media", null, "foobar", 16);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_textarea,
    r#"
<textarea>foobar</textarea>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(128, "textarea", null, "foobar", 16);"#
);

/*
 * Special flags
 */

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_keyed_children_flag,
    r#"
<div $HasKeyedChildren>{magic}</div>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "div", null, magic, 8);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_not_normalize_if_has_vnode_children_set,
    r#"
<div $HasVNodeChildren>{magic}</div>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "div", null, magic, 2);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_dynamic,
    r#"
<div $HasTextChildren>{foobar}</div>
"#,
    r#"
    import { createVNode } from "inferno";
createVNode(1, "div", null, foobar, 16);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_hardcoded,
    r#"
    <div $HasTextChildren>{"testing"}</div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, "testing", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_set_has_text_children_flag_and_not_create_text_vnode_when_has_text_children_is_used_hardcoded2,
    r#"
    <div $HasTextChildren>text</div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, "text", 16);
    "#);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_use_optimized_text_children_instead_create_text_vnode_for_element_single_child,
    r#"
    <div>text</div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, "text", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_non_keyed_children_flag,
    r#"
    <div $HasNonKeyedChildren>{test}</div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, test, 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_re_create_flag,
    r#"
    <div $ReCreate/>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(2049, "div");
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_be_possible_to_define_override_flags_runtime,
    r#"
    <img $Flags={bool ? 1 : 2}>{expression}</img>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(bool ? 1 : 2, "img", null, expression, 0);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_be_possible_to_define_override_flags_with_constant,
    r#"
    <img $Flags={120}>foobar</img>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(120, "img", null, "foobar", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_be_possible_to_use_expression_for_flags,
    r#"
    <ComponentA $Flags={magic}/>
    "#,
    r#"
    import { createComponentVNode } from "inferno";
    createComponentVNode(magic, ComponentA);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_call_to_normalize_props_when_spread_operator_is_used,
    r#"
    <div {...props}>1</div>
    "#,
    r#"
    import {createVNode, normalizeProps} from "inferno";
    normalizeProps(createVNode(1, "div", null, "1", 16, {...props}));
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_call_to_normalize_props_when_spread_operator_is_used_2,
    r#"
    <div foo="bar" className="test" {...props}/>
    "#,
    r#"
    import { createVNode, normalizeProps } from "inferno";
    normalizeProps(createVNode(1, "div", "test", null, 1, {foo: "bar", ...props}));
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_call_to_normalize_props_when_spread_operator_is_used_inside_children_for_component,
    r#"
    <FooBar><BarFoo {...props}/><NoNormalize/></FooBar>
    "#,
    r#"
    import { createComponentVNode, normalizeProps } from "inferno";
    createComponentVNode(2, FooBar, {children: [normalizeProps(createComponentVNode(2, BarFoo, {  ...props})), createComponentVNode(2, NoNormalize)]});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_do_single_normalization_when_multiple_spread_operators_are_used,
    r#"
    <FooBar><BarFoo {...magics} {...foobars} {...props}/><NoNormalize/></FooBar>
    "#,
    r#"
    import { createComponentVNode, normalizeProps } from "inferno";
    createComponentVNode(2, FooBar, {children: [normalizeProps(createComponentVNode(2, BarFoo, {  ...magics,  ...foobars,  ...props})), createComponentVNode(2, NoNormalize)]});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_div,
    r#"
    <div></div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div");
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_single_div,
    r#"
    <div>1</div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, "1", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    test_to_verify_stripping_imports_work,
    r#"
    <div>1</div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, "1", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    class_name_should_be_in_fifth_parameter_as_string_when_its_component,
    r#"
    <UnknownClass className="first second">1</UnknownClass>
    "#,
    r#"
    import { createComponentVNode } from "inferno";
    createComponentVNode(2, UnknownClass, {className: "first second",children: "1"});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    jsxmember_expressions_should_work,
    r#"
    <Components.Unknown>1</Components.Unknown>
    "#,
    r#"
    import { createComponentVNode } from "inferno";
    createComponentVNode(2, Components.Unknown, {children: "1"});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_call_create_vnode_twice_and_text_children,
    r#"<div>
          <div>single</div>
        </div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, createVNode(1, "div", null, "single", 16), 2);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    events_should_be_in_props,
    r#"
    <div id="test" onClick={func} class={variable}>1</div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", variable, "1", 16, {id: "test", onClick: func});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_input_and_htmlfor_correctly,
    r#"
    <label htmlFor={id}><input id={id} name={name} value={value} onChange={onChange} onInput={onInput} onKeyup={onKeyup} onFocus={onFocus} onClick={onClick} type="number" pattern="[0-9]+([,\.][0-9]+)?" inputMode="numeric" min={minimum}/></label>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "label", null, createVNode(64, "input", null, null, 1, {id: id,name: name,value: value,onChange: onChange,onInput: onInput,onKeyup: onKeyup,onFocus: onFocus,onClick: onClick,type: "number",pattern: "[0-9]+([,\\.][0-9]+)?",inputMode: "numeric",min: minimum}), 2, {"for": id});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_double_click_to_native_html_event,
    r#"
    <div onDoubleClick={foobar}></div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, null, 1, {onDblClick: foobar});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    class_name_should_be_in_third_parameter_as_string_when_its_element,
    r#"
    <div className="first second">1</div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", "first second", "1", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_xlink_href,
    r#"
    <svg><use xlinkHref="tester"></use></svg>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(32, "svg", null, createVNode(32, "use", null, null, 1, {"xlink:href": "tester"}), 2);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_stroke_width,
    r#"
    <svg><rect strokeWidth="1px"></rect></svg>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(32, "svg", null, createVNode(32, "rect", null, null, 1, {"stroke-width": "1px"}), 2);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    contenteditbale_1,
    r#"
    <div contentEditable></div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(4097, "div", null, null, 1, {contentEditable: true});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    contenteditbale_2,
    r#"
    <span contenteditable="false"></span>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(4097, "span", null, null, 1, {contenteditable: "false"});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    contenteditbale_3,
    r#"
    <div contenteditable></div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(4097, "div", null, null, 1, {contenteditable: true});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    contenteditbale_4,
    r#"
    <div contentEditable={logic}></div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(4097, "div", null, null, 1, {contentEditable: logic});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    contenteditbale_5,
    r#"
    <div contentEditable="true"></div>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(4097, "div", null, null, 1, {contentEditable: "true"});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_transform_fill_opacity_2,
    r#"
      <svg><rect fillOpacity="1"></rect></svg>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(32, "svg", null, createVNode(32, "rect", null, null, 1, {"fill-opacity": "1"}), 2);
    "#
);

// TODO: How to verify errors
// test!(
//     ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
//         jsx: true,
//         ..Default::default()
//     }),
//     |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
//     Element_should_prefer_child_element_over_children_props,
//     r#"
//     <div children="ab">test</div>
//     "#,
//     r#"
//     import { createVNode } from "inferno";
//     createVNode(1, "div", null, "test", 16);
//     "#);

// TODO: How to verify errors
// test!(
//     ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
//         jsx: true,
//         ..Default::default()
//     }),
//     |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
//     Element_should_use_prop_if_no_children_exists,
//     r#"
//     <div children="ab"/>
//     "#,
//     r#"
//   x 'children' property is not supported for regular elements. Use nesting
// instead.    ,-[input.js:1:1]
//  1 |
//  2 |     <div children="ab">test</div>
//    :          ^^^^^^^^
//  3 |
//    `----
//     "#);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_not_fail_if_create_vnode_is_already_imported,
    r#"
  import {createVNode} from "inferno"; var foo = <div/>;
"#,
    r#"
    import { createVNode } from "inferno";
    var foo = createVNode(1, "div");
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_not_wrap_dynamic_value,
    r#"
      <>{magic}</>
    "#,
    r#"
    import {createFragment} from "inferno";
    createFragment(magic, 0);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_short_syntax_fragment,
    r#"
      <><>Text</></>
    "#,
    r#"
    import {createTextVNode, createFragment} from "inferno";
    createFragment([createFragment([createTextVNode("Text")], 4)], 4);
    "#);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_always_short_syntax_fragment,
    r#"
      <><><div>Text</div></></>
    "#,
    r#"
    import {createVNode, createFragment} from "inferno";
    createFragment([createFragment([createVNode(1, "div", null, "Text", 16)], 4)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_handle_many_dynamic_children_short_syntax,
    r#"
      <><>{Frag}Text{Wohoo}</></>
    "#,
    r#"
    import {createTextVNode, createFragment} from "inferno";
    createFragment([createFragment([Frag, createTextVNode("Text"), Wohoo], 0)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_create_text_vnode_when_there_are_siblings,
    r#"
    <div>Okay<span>foo</span></div>
    "#,
    r#"
    import { createVNode, createTextVNode } from "inferno";
    createVNode(1, "div", null, [createTextVNode("Okay"), createVNode(1, "span", null, "foo", 16)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_handle_many_dynamic_and_non_dynamic_children_short_syntax,
    r#"
      <><><span></span>Text{Wohoo}</></>
    "#,
    r#"
    import { createVNode, createTextVNode, createFragment } from "inferno";
    createFragment([createFragment([createVNode(1, "span"), createTextVNode("Text"), Wohoo], 0)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_long_syntax_fragment_2,
      r#"
      <Fragment><Fragment>Text</Fragment></Fragment>
    "#,
    r#"
    import { createTextVNode, createFragment } from "inferno";
    createFragment([createFragment([createTextVNode("Text")], 4)], 4);
    "#);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_create_text_vnode_when_text_node_is_under_large_syntax_fragment,
    r#"
      <Fragment>Okay<span>foo</span></Fragment>
    "#,
    r#"
    import { createVNode, createTextVNode, createFragment } from "inferno";
    createFragment([createTextVNode("Okay"), createVNode(1, "span", null, "foo", 16)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_always_keep_text_node_as_children_even_if_there_is_one_when_parent_is_long_syntax_fragment,
      r#"
      <Fragment><Fragment>Text</Fragment></Fragment>
    "#,
    r#"
    import { createTextVNode, createFragment } from "inferno";
    createFragment([createFragment([createTextVNode("Text")], 4)], 4);
    "#);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_always_long_syntax_fragment,
    r#"
      <Fragment><Fragment><div>Text</div></Fragment></Fragment>
    "#,
    r#"
    import { createVNode, createFragment } from "inferno";
    createFragment([createFragment([createVNode(1, "div", null, "Text", 16)], 4)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_handle_many_dynamic_children_long_syntax,
    r#"
      <Fragment><Fragment>{Frag}Text{Wohoo}</Fragment></Fragment>
    "#,
    r#"
    import { createTextVNode, createFragment } from "inferno";
    createFragment([createFragment([Frag, createTextVNode("Text"), Wohoo], 0)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_handle_many_dynamic_and_non_dynamic_children_long_syntax,
    r#"
      <Fragment><Fragment><span></span>Text{Wohoo}</Fragment></Fragment>
    "#,
    r#"
    import { createVNode, createTextVNode, createFragment } from "inferno";
    createFragment([createFragment([createVNode(1, "span"), createTextVNode("Text"), Wohoo], 0)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_create_text_vnode_when_text_node_is_under_short_syntax_fragment,
    r#"
      <>Okay<span>foo</span></>
    "#,
    r#"
    import { createVNode, createTextVNode, createFragment } from "inferno";
    createFragment([createTextVNode("Okay"), createVNode(1, "span", null, "foo", 16)], 4);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragments_syntax_should_create_empty_create_fragment,
    r#"
      <></>
    "#,
    r#"
    import { createFragment } from "inferno";
    createFragment();
      "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragments_syntax_should_create_fragment,
    r#"
      <>Test</>
    "#,
    r#"
    import { createTextVNode, createFragment } from "inferno";
    createFragment([createTextVNode("Test")], 4);
      "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragments_syntax_should_create_fragment_dynamic_children,
    r#"
      <>{dynamic}</>
    "#,
    r#"
    import { createFragment } from "inferno";
    createFragment(dynamic, 0);
      "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragments_syntax_should_create_fragment_keyed_children,
    r#"
      <><span key="ok">kk</span><div key="ok2">ok</div></>
    "#,
    r#"
    import { createVNode, createFragment } from "inferno";
    createFragment([createVNode(1, "span", null, "kk", 16, null, "ok"), createVNode(1, "div", null, "ok", 16, null, "ok2")], 8);
      "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragments_syntax_should_create_fragment_non_keyed_children,
    r#"
      <><div>1</div><span>foo</span></>
    "#,
    r#"
    import { createVNode, createFragment } from "inferno";
    createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4);
      "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_empty_create_fragment1,
    r#"
      <Fragment></Fragment>
    "#,
    r#"
      import { createFragment } from "inferno";
      createFragment();
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_empty_create_fragment2,
    r#"
      <Fragment/>
    "#,
    r#"
      import { createFragment } from "inferno";
      createFragment();
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_fragment,
    r#"
      <Fragment>Test</Fragment>
    "#,
    r#"
    import { createTextVNode, createFragment } from "inferno";
    createFragment([createTextVNode("Test")], 4);
        "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_fragment_dynamic_children,
    r#"
      <Fragment>{dynamic}</Fragment>
        "#,
    r#"
        import { createFragment } from "inferno";
        createFragment(dynamic, 0);
        "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_fragment_keyed_children_2,
    r#"
      <Fragment><span key="ok">kk</span><div key="ok2">ok</div></Fragment>
    "#,
    r#"
    import { createVNode, createFragment } from "inferno";
    createFragment([createVNode(1, "span", null, "kk", 16, null, "ok"), createVNode(1, "div", null, "ok", 16, null, "ok2")], 8);
        "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_fragment_non_keyed_children_3,
    r#"
      <Fragment><div>1</div><span>foo</span></Fragment>
    "#,
    r#"
    import { createVNode, createFragment } from "inferno";
    createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4);
        "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_fragment_non_keyed_children_2,
    r#"
      <Fragment key="foo"><div>1</div><span>foo</span></Fragment>
    "#,
    r#"
    import { createVNode, createFragment } from "inferno";
    createFragment([createVNode(1, "div", null, "1", 16), createVNode(1, "span", null, "foo", 16)], 4, "foo");
        "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_fragment_keyed_children,
    r#"
      <Fragment key="foo" $HasKeyedChildren>{magic}</Fragment>
    "#,
    r#"
    import { createFragment } from "inferno";
    createFragment(magic, 8, "foo");
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    fragment_long_should_create_fragment_non_keyed_children,
    r#"
      <Fragment key="foo" $HasNonKeyedChildren>{magic}</Fragment>
    "#,
    r#"
    import { createFragment } from "inferno";
    createFragment(magic, 4, "foo");
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_add_import_to_create_vnode_component_but_not_to_create_vnode_if_create_vnode_is_already_declared,
    r#"
      import {createVNode} from "inferno"; var foo = <FooBar/>;
    "#,
    r#"
    import { createVNode, createComponentVNode } from "inferno";
    var foo = createComponentVNode(2, FooBar);
    "#);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    component_should_prefer_child_element_over_children_props,
    r#"
    <Com children="ab">test</Com>
    "#,
    r#"
    import { createComponentVNode } from "inferno";
    createComponentVNode(2, Com, {children: "test"});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    component_should_prefer_prop_over_empty_children,
    r#"
    <Com children="ab"></Com>
    "#,
    r#"
    import { createComponentVNode } from "inferno";
    createComponentVNode(2, Com, {children: "ab"});
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    component_should_use_prop_if_no_children_exists,
    r#"
    <Com children="ab"/>
    "#,
    r#"
    import { createComponentVNode } from "inferno";
    createComponentVNode(2, Com, {children: "ab"});
    "#
);

// This could be optimized to have HasVNodeChildren set,
// but I'm not sure if anybody writes code like this
test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_prefer_xml_children_over_props,
    r#"
    <foo children={<span>b</span>}></foo>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "foo", null, createVNode(1, "span", null, "b", 16));
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    should_convert_jsx_attributes_to_vnodes,
    r#"
    <foo aasd={<span>b</span>}></foo>
    "#,
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "foo", null, null, 1, {
        aasd: createVNode(1, "span", null, "b", 16)
    });
    "#
);

// TODO: How to verify errors
// test!(
//     ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
//         jsx: true,
//         ..Default::default()
//     }),
//     |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
//     should_prefer_xml_children_over_props_null,
//     r#"
//     <foo children={null}></foo>
//     "#,
//     r#"
//     import { createVNode } from "inferno";
//     createVNode(1, "foo");
//     "#);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    component_array_empty_children,
    r#"
    <Com>{[]}</Com>
    "#,
    r#"
    import {createComponentVNode} from "inferno"
    createComponentVNode(2, Com, {
        children: []
    });
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    component_should_create_vnode_for_prop,
    r#"
    <Com asd={<div>1</div>}/>
    "#,
    r#"
    import { createVNode, createComponentVNode } from "inferno";
    createComponentVNode(2, Com, {asd: createVNode(1, "div", null, "1", 16)});
    "#
);

fn tr(t: &mut Tester, options: Options, top_level_mark: Mark) -> impl Fold {
    let unresolved_mark = Mark::new();
    // Create a handler wired with plugin's diagnostic emitter, set it for global context.
    let handler = swc_core::common::errors::Handler::with_emitter(
        true,
        false,
        Box::new(PluginDiagnosticsEmitter),
    );
    let _ = HANDLER.inner.set(handler);

    chain!(
        resolver(unresolved_mark, top_level_mark, false),
        jsx(Some(t.comments.clone()), options, unresolved_mark),
    )
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureOptions {
    #[serde(flatten)]
    options: Options,

    #[serde(default = "true_by_default")]
    pure: bool,

    #[serde(default)]
    throws: Option<String>,

    #[serde(default, alias = "useBuiltIns")]
    use_builtins: bool,
}

fn true_by_default() -> bool {
    true
}

fn fixture_tr(t: &mut Tester, options: FixtureOptions) -> impl Fold {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    let handler = swc_core::common::errors::Handler::with_emitter(
        true,
        false,
        Box::new(PluginDiagnosticsEmitter),
    );
    let _ = HANDLER.inner.set(handler);

    chain!(
        resolver(unresolved_mark, top_level_mark, false),
        inferno(
            t.cm.clone(),
            Some(t.comments.clone()),
            options.options,
            top_level_mark,
            unresolved_mark
        ),
        pure_annotations(Some(t.comments.clone()))
    )
}

fn integration_tr(t: &mut Tester, options: FixtureOptions) -> impl Fold {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    let handler = swc_core::common::errors::Handler::with_emitter(
        true,
        false,
        Box::new(PluginDiagnosticsEmitter),
    );
    let _ = HANDLER.inner.set(handler);

    chain!(
        resolver(unresolved_mark, top_level_mark, false),
        inferno(
            t.cm.clone(),
            Some(t.comments.clone()),
            options.options,
            top_level_mark,
            unresolved_mark
        )
    )
}

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_add_appropriate_newlines,
    r#"
<Component
  {...props}
  sound="moo" />
"#,
    r#"
import { createComponentVNode, normalizeProps } from "inferno";
normalizeProps(createComponentVNode(2, Component, {
    ...props,
    sound: "moo"
}));
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_arrow_functions,
    r#"
var foo = function () {
  return () => <this />;
};

var bar = function () {
  return () => <this.foo />;
};
"#,
    r#"
import { createComponentVNode } from "inferno";
var foo = function() {
    return ()=>createComponentVNode(2, this);
};
var bar = function() {
    return ()=>createComponentVNode(2, this.foo);
};
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_concatenates_adjacent_string_literals,
    r#"
var x =
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
  </div>
  "#,
    r#"
import { createVNode, createTextVNode } from "inferno";
var x = createVNode(1, "div", null, [
    createTextVNode("foo"),
    createTextVNode("bar"),
    createTextVNode("baz"),
    createVNode(1, "div", null, "buz bang", 16),
    createTextVNode("qux"),
    null,
    createTextVNode("quack")
], 0);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_dont_coerce_expression_containers,
    r#"
<Text>
  To get started, edit index.ios.js!!!{"\n"}
  Press Cmd+R to reload
</Text>
"#,
    r#"import { createComponentVNode, createTextVNode } from "inferno";
createComponentVNode(2, Text, {
    children: [
        createTextVNode("To get started, edit index.ios.js!!!"),
        createTextVNode("\n"),
        createTextVNode("Press Cmd+R to reload")
    ]
});
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_noop_ported_honor_custom_jsx_comment_if_jsx_pragma_option_set,
    r#"/** @jsx dom */

<Foo></Foo>;

var profile = <div>
  <img src="avatar.png" className="profile" />
  <h3>{[user.firstName, user.lastName].join(" ")}</h3>
</div>;"#,
    r#"
import { createVNode, createComponentVNode } from "inferno";
createComponentVNode(2, Foo);
var profile = createVNode(1, "div", null, [
    createVNode(1, "img", "profile", null, 1, {
        src: "avatar.png"
    }),
    createVNode(1, "h3", null, [
        user.firstName,
        user.lastName
    ].join(" "), 0)
], 4);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_noop_honor_custom_jsx_comment,
    r#"
/** @jsx dom */

<Foo></Foo>;

var profile = <div>
  <img src="avatar.png" className="profile" />
  <h3>{[user.firstName, user.lastName].join(" ")}</h3>
</div>;
"#,
    r#"
import { createVNode, createComponentVNode } from "inferno";
createComponentVNode(2, Foo);
var profile = createVNode(1, "div", null, [
    createVNode(1, "img", "profile", null, 1, {
        src: "avatar.png"
    }),
    createVNode(1, "h3", null, [
        user.firstName,
        user.lastName
    ].join(" "), 0)
], 4);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(
        t,
        Options {
            ..Default::default()
        },
        Mark::fresh(Mark::root())
    ),
    ported_noop_honor_custom_jsx_pragma_option,
    r#"

<Foo></Foo>;

var profile = <div>
  <img src="avatar.png" className="profile" />
  <h3>{[user.firstName, user.lastName].join(" ")}</h3>
</div>;"#,
    r#"
import { createVNode, createComponentVNode } from "inferno";
createComponentVNode(2, Foo);
var profile = createVNode(1, "div", null, [
    createVNode(1, "img", "profile", null, 1, {
        src: "avatar.png"
    }),
    createVNode(1, "h3", null, [
        user.firstName,
        user.lastName
    ].join(" "), 0)
], 4);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_noop_jsx_with_retainlines_option,
    r#"var div = <div>test</div>;"#,
    r#"import { createVNode } from "inferno";
var div = createVNode(1, "div", null, "test", 16);"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_noop_jsx_without_retainlines_option,
    r#"var div = <div>test</div>;"#,
    r#"import { createVNode } from "inferno";
var div = createVNode(1, "div", null, "test", 16);"#
);

test!(
    // This is not worth optimization if Inferno does not have support for static vNodes trees
    // cloning the element runtime is more expensive than creating new and adds extra memory
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_noop_optimisation_ported_constant_elements,
    r#"
import {Component} from "inferno";

class App extends Component {
  render() {
    const navbarHeader = <div className="navbar-header">
      <a className="navbar-brand" href="/">
        <img src="/img/logo/logo-96x36.png" />
      </a>
    </div>;

    return <div>
      <nav className="navbar navbar-default">
        <div className="container">
          {navbarHeader}
        </div>
      </nav>
    </div>;
  }
}
"#,
    r#"
import { Component, createVNode } from "inferno";
class App extends Component {
    render() {
        const navbarHeader = createVNode(1, "div", "navbar-header", createVNode(1, "a", "navbar-brand", createVNode(1, "img", null, null, 1, {
            src: "/img/logo/logo-96x36.png"
        }), 2, {
            href: "/"
        }), 2);
        return createVNode(1, "div", null, createVNode(1, "nav", "navbar navbar-default", createVNode(1, "div", "container", navbarHeader, 0), 2), 2);
    }
}"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| chain!(
        tr(t, Default::default(), Mark::fresh(Mark::root())),
        property_literals(),
    ),
    ported_should_add_quotes_es3,
    r#"var es3 = <F aaa new const var default foo-bar/>;"#,
    r#"
import { createComponentVNode } from "inferno";
var es3 = createComponentVNode(2, F, {
  aaa: true,
  "new": true,
  "const": true,
  "var": true,
  "default": true,
  "foo-bar": true
});
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_allow_constructor_as_prop,
    r#"<Component constructor="foo" />;"#,
    r#"
import { createComponentVNode } from "inferno";
createComponentVNode(2, Component, {
    constructor: "foo"
});
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_allow_deeper_js_namespacing,
    r#"<Namespace.DeepNamespace.Component />;"#,
    r#"
import { createComponentVNode } from "inferno";
createComponentVNode(2, Namespace.DeepNamespace.Component);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_allow_elements_as_attributes,
    r#"<div attr=<div /> />"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "div", null, null, 1, {
    attr: createVNode(1, "div")
});"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_allow_js_namespacing,
    r#"<Namespace.Component />;"#,
    r#"
import { createComponentVNode } from "inferno";
createComponentVNode(2, Namespace.Component);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_allow_nested_fragments,
    r#"
<div>
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
</div>
"#,
    r#"
import { createVNode, createFragment } from "inferno";
createVNode(1, "div", null, createFragment([
    createFragment([
        createVNode(1, "span", null, "Hello", 16),
        createVNode(1, "span", null, "world", 16)
    ], 4),
    createFragment([
        createVNode(1, "span", null, "Goodbye", 16),
        createVNode(1, "span", null, "world", 16)
    ], 4)
], 4), 2);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_noop_should_allow_no_pragmafrag_if_frag_unused,
    r#"
/** @jsx dom */

<div>no fragment is used</div>
"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "div", null, "no fragment is used", 16);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_noop_should_allow_pragmafrag_and_frag,
    r#"
/** @jsx dom */
/** @jsxFrag DomFrag */

<></>
"#,
    r#"
import { createFragment } from "inferno";
createFragment();
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_avoid_wrapping_in_extra_parens_if_not_needed,
    r#"
var x = <div>
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
</Composite>;
"#,
    r#"
import { createVNode, createComponentVNode } from "inferno";
var x = createVNode(1, "div", null, createComponentVNode(2, Component), 2);
var x = createVNode(1, "div", null, props.children, 0);
var x = createComponentVNode(2, Composite, {
    children: props.children
});
var x = createComponentVNode(2, Composite, {
    children: createComponentVNode(2, Composite2)
});"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_convert_simple_tags,
    r#"var x = <div></div>;"#,
    r#"
    import { createVNode } from "inferno";
    var x = createVNode(1, "div");
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_convert_simple_text,
    r#"var x = <div>text</div>;"#,
    r#"
    import { createVNode } from "inferno";
    var x = createVNode(1, "div", null, "text", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_escape_xhtml_jsxattribute,
    r#"
<div id="wôw" />;
<div id="\w" />;
<div id="w &lt; w" />;
"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "div", null, null, 1, {
    id: "w\xf4w"
});
createVNode(1, "div", null, null, 1, {
    id: "\\w"
});
createVNode(1, "div", null, null, 1, {
    id: "w < w"
});"#,
    ok_if_code_eq
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_escape_xhtml_jsxtext_1,
    r"
<div>wow</div>;
<div>wôw</div>;

<div>w & w</div>;
<div>w &amp; w</div>;

<div>w &nbsp; w</div>;
<div>this should parse as unicode: {'\u00a0 '}</div>;

<div>w &lt; w</div>;
",
    r#"
import { createVNode, createTextVNode } from "inferno";
createVNode(1, "div", null, "wow", 16);
createVNode(1, "div", null, "w\xf4w", 16);
createVNode(1, "div", null, "w & w", 16);
createVNode(1, "div", null, "w & w", 16);
createVNode(1, "div", null, "w \xa0 w", 16);
createVNode(1, "div", null, [
    createTextVNode("this should parse as unicode: "),
    createTextVNode('\u00a0 ')
], 0);
createVNode(1, "div", null, "w < w", 16);"#,
    ok_if_code_eq
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_escape_xhtml_jsxtext_2,
    r"
<div>this should not parse as unicode: \u00a0</div>;
",
    r#"
import { createVNode } from "inferno";
createVNode(1, "div", null, "this should not parse as unicode: \\u00a0", 16);
"#,
    ok_if_code_eq
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_escape_unicode_chars_in_attribute,
    r#"<Bla title="Ú"/>"#,
    r#"import { createComponentVNode } from "inferno";
createComponentVNode(2, Bla, {
    title: "\xda"
});"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_escape_xhtml_jsxtext_3,
    r#"
<div>this should parse as nbsp:   </div>;
"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "div", null, "this should parse as nbsp:   ", 16);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_handle_attributed_elements,
    r#"
var HelloMessage = Inferno.createClass({
  render: function() {
    return <div>Hello {this.props.name}</div>;
  }
});

Inferno.render(<HelloMessage name={
  <span>
    Sebastian
  </span>
} />, mountNode);
"#,
    r#"
import { createVNode, createComponentVNode, createTextVNode } from "inferno";
var HelloMessage = Inferno.createClass({
    render: function() {
        return createVNode(1, "div", null, [
            createTextVNode("Hello "),
            this.props.name
        ], 0);
    }
});
Inferno.render(createComponentVNode(2, HelloMessage, {
    name: createVNode(1, "span", null, "Sebastian", 16)
}), mountNode);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_handle_has_own_property_correctly,
    r#"<hasOwnProperty>testing</hasOwnProperty>;"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "hasOwnProperty", null, "testing", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_have_correct_comma_in_nested_children,
    r#"
var x = <div>
  <div><br /></div>
  <Component>{foo}<br />{bar}</Component>
  <br />
</div>;
"#,
    r#"
import { createVNode, createComponentVNode } from "inferno";
var x = createVNode(1, "div", null, [
    createVNode(1, "div", null, createVNode(1, "br"), 2),
    createComponentVNode(2, Component, {
        children: [
            foo,
            createVNode(1, "br"),
            bar
        ]
    }),
    createVNode(1, "br")
], 4);
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_insert_commas_after_expressions_before_whitespace,
    r#"
var x =
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
    }
    attr4="baz">
  </div>
"#,
    r#"
import { createVNode } from "inferno";
var x = createVNode(1, "div", null, null, 1, {
    attr1: "foo" + "bar",
    attr2: "foo" + "bar" + "baz" + "bug",
    attr3: "foo" + "bar" + "baz" + "bug",
    attr4: "baz"
});
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_not_add_quotes_to_identifier_names,
    r#"var e = <F aaa new const var default foo-bar/>;"#,
    r#"
import { createComponentVNode } from "inferno";
var e = createComponentVNode(2, F, {
  aaa: true,
  new: true,
  const: true,
  var: true,
  default: true,
  "foo-bar": true
});
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_not_mangle_expressioncontainer_attribute_values,
    r#"<button data-value={"a value\n  with\nnewlines\n   and spaces"}>Button</button>;"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "button", null, "Button", 16, {
    "data-value": "a value\n  with\nnewlines\n   and spaces"
});"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_not_strip_nbsp_even_coupled_with_other_whitespace,
    r#"<div>&nbsp; </div>;"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "div", null, "\xa0 ", 16);
    "#,
    ok_if_code_eq
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_not_strip_tags_with_a_single_child_of_nbsp,
    r#"<div>&nbsp;</div>;"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "div", null, "\xa0", 16);
    "#,
    ok_if_code_eq
);

test!(
    // Comments are currently stripped out
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_noop_should_properly_handle_comments_between_props,
    r#"
var x = (
  <div
/* a multi-line
comment */
    attr1="foo">
<span // a double-slash comment
      attr2="bar"
    />
  </div>
);
"#,
    r#"
    import { createVNode } from "inferno";
var x = createVNode(1, "div", null, createVNode(1, "span", null, null, 1, {
    attr2: "bar"
}), 2, {
    attr1: "foo"
});
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_quote_jsx_attributes,
    r#"<button data-value='a value'>Button</button>;"#,
    r#"
import { createVNode } from "inferno";
createVNode(1, "button", null, "Button", 16, {
    "data-value": "a value"
});
"#
);

// TODO: Namespaces disabled
// test!(
//     ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
//         jsx: true,
//         ..Default::default()
//     }),
//     |t| tr(
//         t,
//         Options {
//             pragma: Some("h".into()),
//             throw_if_namespace: false.into(),
//             ..Default::default()
//         },
//         Mark::fresh(Mark::root())
//     ),
//     ported_should_support_xml_namespaces_if_flag,
//     r#"<f:image n:attr />;"#,
//     r#"h("f:image", {
//   "n:attr": true
// });"#
// );

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_should_transform_known_hyphenated_tags,
    r#"<font-face />;"#,
    r#"
import { createVNode } from "inferno";
createVNode(32, "font-face");
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_wraps_props_in_ported_spread_for_first_spread_attributes,
    r#"
<Component { ... x } y
={2 } z />
"#,
    r#"
import { createComponentVNode, normalizeProps } from "inferno";
normalizeProps(createComponentVNode(2, Component, {
    ...x,
    y: 2,
    z: true
}));
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_wraps_props_in_ported_spread_for_last_spread_attributes,
    r#"<Component y={2} z { ... x } />"#,
    r#"
import { createComponentVNode, normalizeProps } from "inferno";
normalizeProps(createComponentVNode(2, Component, {
    y: 2,
    z: true,
    ...x
}));
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_wraps_props_in_ported_spread_for_middle_spread_attributes,
    r#"<Component y={2} { ... x } z />"#,
    r#"
import { createComponentVNode, normalizeProps } from "inferno";
normalizeProps(createComponentVNode(2, Component, {
    y: 2,
    ...x,
    z: true
}));"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    ported_attribute_html_entity_quote,
    r#"<Component text="Hello &quot;World&quot;" />"#,
    r#"
import { createComponentVNode } from "inferno";
createComponentVNode(2, Component, {
    text: 'Hello "World"'
});"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    use_builtins_assignment,
    r#"var div = <Component {...props} foo="bar" />"#,
    r#"
import { createComponentVNode, normalizeProps } from "inferno";
var div = normalizeProps(createComponentVNode(2, Component, {
    ...props,
    foo: "bar"
}));"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    use_spread_assignment,
    r#"<Component y={2} { ...x } z />"#,
    r#"
import { createComponentVNode, normalizeProps } from "inferno";
normalizeProps(createComponentVNode(2, Component, {
    y: 2,
    ...x,
    z: true
}));
"#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    issue_229,
    r#"
    const a = <>test</>
    const b = <div>test</div>
    "#,
    r#"
    import { createVNode, createTextVNode, createFragment } from "inferno";
    const a = createFragment([
        createTextVNode("test")
    ], 4);
    const b = createVNode(1, "div", null, "test", 16);
    "#
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| {
        let top_level_mark = Mark::fresh(Mark::root());
        chain!(
            tr(t, Default::default(), top_level_mark),
            common_js(
                top_level_mark,
                Default::default(),
                Default::default(),
                Some(t.comments.clone())
            )
        )
    },
    issue_351,
    "import Inferno from 'inferno';

<div />;",
    "\"use strict\";
Object.defineProperty(exports, \"__esModule\", {
    value: true
});
var _inferno = _interop_require_wildcard(require(\"inferno\"));
(0, _inferno.createVNode)(1, \"div\");"
);

test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    issue_481,
    "<span> {foo}</span>;",
    r#"
import { createVNode, createTextVNode } from "inferno";
createVNode(1, "span", null, [
    createTextVNode(" "),
    foo
], 0);
    "#
);

// https://github.com/swc-project/swc/issues/517
test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| {
        let top_level_mark = Mark::fresh(Mark::root());
        chain!(
            tr(t, Default::default(), top_level_mark),
            common_js(
                Mark::fresh(Mark::root()),
                Default::default(),
                Default::default(),
                Some(t.comments.clone())
            )
        )
    },
    issue_517,
    "import Inferno from 'inferno';
<div style='white-space: pre'>Hello World</div>;",
    r#"
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _inferno = _interop_require_wildcard(require("inferno"));
(0, _inferno.createVNode)(1, "div", null, "Hello World", 16, {
    style: "white-space: pre"
});
    "#
);

#[test]
fn jsx_text() {
    assert_eq!(jsx_text_to_str(" ".into()), *" ");
    assert_eq!(jsx_text_to_str("Hello world".into()), *"Hello world");
    //    assert_eq!(jsx_text_to_str(" \n".into()), *" ");
}

// https://github.com/swc-project/swc/issues/542
test!(
    ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| tr(t, Default::default(), Mark::fresh(Mark::root())),
    issue_542,
    "let page = <p>Click <em>New melody</em> listen to a randomly generated melody</p>",
    r#"
import { createVNode, createTextVNode } from "inferno";
let page = createVNode(1, "p", null, [
    createTextVNode("Click "),
    createVNode(1, "em", null, "New melody", 16),
    createTextVNode(" listen to a randomly generated melody")
], 4);
"#
);

test!(
    Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |t| {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        chain!(
            resolver(unresolved_mark, top_level_mark, false),
            jsx(
                Some(t.comments.clone()),
                Default::default(),
                unresolved_mark
            )
        )
    },
    issue_4956,
    "
    <div title=\"\u{2028}\"/>
    ",
    r#"
    import { createVNode } from "inferno";
    createVNode(1, "div", null, null, 1, {
      title: "\u2028"
    });
  "#
);

#[testing::fixture("tests/jsx/fixture/**/input.js")]
fn fixture(input: PathBuf) {
    let mut output = input.with_file_name("output.js");
    if !output.exists() {
        output = input.with_file_name("output.mjs");
    }

    test_fixture(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        &|t| {
            let options = parse_options(input.parent().unwrap());
            fixture_tr(t, options)
        },
        &input,
        &output,
        FixtureTestConfig {
            allow_error: true,
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/integration/fixture/**/input.js")]
fn integration(input: PathBuf) {
    let mut output = input.with_file_name("output.js");
    if !output.exists() {
        output = input.with_file_name("output.mjs");
    }

    test_fixture(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        &|t| {
            let options = parse_options(input.parent().unwrap());
            integration_tr(t, options)
        },
        &input,
        &output,
        FixtureTestConfig {
            allow_error: true,
            ..Default::default()
        },
    );
}

#[testing::fixture("tests/script/**/input.js")]
fn script(input: PathBuf) {
    let output = input.with_file_name("output.js");

    let options = parse_options(input.parent().unwrap());

    let input = fs::read_to_string(&input).unwrap();

    test_script(&input, &output, options);
}

fn test_script(src: &str, output: &Path, options: Options) {
    Tester::run(|tester| {
        let fm = tester
            .cm
            .new_source_file(FileName::Real("input.js".into()), src.into());

        let syntax = Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        });

        let mut parser = Parser::new(syntax, StringInput::from(&*fm), Some(&tester.comments));

        let script = parser.parse_script().unwrap();

        let top_level_mark = Mark::new();
        let unresolved_mark = Mark::new();

        let script = script.fold_with(&mut chain!(
            resolver(Mark::new(), top_level_mark, false),
            inferno(
                tester.cm.clone(),
                Some(&tester.comments),
                options,
                top_level_mark,
                unresolved_mark,
            ),
            hygiene(),
            fixer(Some(&tester.comments))
        ));

        let mut buf = vec![];

        let mut emitter = Emitter {
            cfg: Config::default()
                .with_ascii_only(true)
                .with_omit_last_semi(true),
            cm: tester.cm.clone(),
            wr: Box::new(swc_ecma_codegen::text_writer::JsWriter::new(
                tester.cm.clone(),
                "\n",
                &mut buf,
                None,
            )),
            comments: Some(&tester.comments),
        };

        // println!("Emitting: {:?}", module);
        emitter.emit_script(&script).unwrap();

        let s = String::from_utf8_lossy(&buf).to_string();
        assert!(NormalizedOutput::new_raw(s).compare_to_file(output).is_ok());

        Ok(())
    })
}
