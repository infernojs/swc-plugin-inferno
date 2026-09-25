//! Ported from babel-plugin-inferno `tests/tsx.test.js`: TSX with @babel/preset-typescript

use crate::helpers::*;

/// type syntax
mod type_syntax {
    use super::*;

    // Not ported:
    // - Should drop function type arguments (swc_ecma_parser rejects function type arguments on JSX tags)

    #[test]
    fn should_drop_type_arguments_of_components() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx(
                "{}",
                "<Foo<string> bar={x as any} baz={y!} />",
            )),
            r#"createComponentVNode(2, Foo, {
  "bar": x,
  "baz": y
});"#,
        );
    }

    #[test]
    fn should_drop_type_arguments_of_components_with_children() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx(
                "{}",
                r"<C<number>></C>;
<C<number>/>;",
            )),
            r"createComponentVNode(2, C);
createComponentVNode(2, C);",
        );
    }

    #[test]
    fn should_strip_as_expressions_in_ref_and_children() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx(
                "{}",
                "<div ref={r as any}>{(v as number)}</div>",
            )),
            r#"createVNode(1, "div", null, v, 0, null, null, r);"#,
        );
    }

    #[test]
    fn should_strip_as_and_non_null_expressions_in_children() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx("{}", "<div>{(v as number)}{w!}</div>")),
            r#"createVNode(1, "div", null, [v, w], 0);"#,
        );
    }

    #[test]
    fn should_strip_as_expressions_in_spreads() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx("{}", "<Foo {...(p as Props)} />")),
            r"normalizeProps(createComponentVNode(2, Foo, {
  ...p
}));",
        );
    }

    #[test]
    fn should_strip_parameter_types_of_event_handlers() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx(
                "{}",
                "<div onClick={(e: MouseEvent) => f(e)} />",
            )),
            r#"createVNode(1, "div", null, null, 1, {
  "onClick": e => f(e)
});"#,
        );
    }

    #[test]
    fn should_strip_satisfies_in_keys() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx("{}", "<div key={k satisfies string} />")),
            r#"createVNode(1, "div", null, null, 1, null, k);"#,
        );
    }

    #[test]
    fn should_compile_jsx_in_a_generic_arrow_function() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx(
                "{}",
                "export const f = <T,>(x: T) => <div>{x as any}</div>;",
            )),
            r#"export const f = x => createVNode(1, "div", null, x, 0);"#,
        );
    }

    // swc inlines the enum member.
    #[test]
    fn should_compile_jsx_next_to_an_enum() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx(
                "{}",
                r"enum E { A }
export const a = <div data-e={E.A}/>;",
            )),
            r#"var E = function(E) {
    E[E["A"] = 0] = "A";
    return E;
}(E || {});
export const a = createVNode(1, "div", null, null, 1, {
    "data-e": 0
});"#,
        );
    }

    // @babel/preset-typescript compiles the namespace differently.
    #[test]
    fn should_compile_jsx_inside_a_namespace() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx(
                "{}",
                "namespace N { export const el = <div/>; }",
            )),
            r#"(function(N) {
    N.el = createVNode(1, "div");
})(N || (N = {}));
var N;"#,
        );
    }

    #[test]
    fn should_compile_jsx_after_a_generic_class() {
        assert_js_eq(
            &strip_inferno_import(&transform_tsx(
                "{}",
                r"class C extends D<T> {}
<C/>;",
            )),
            r"class C extends D {}
createComponentVNode(2, C);",
        );
    }
}

/// import elision
mod import_elision {
    use super::*;

    // Not ported:
    // - Should keep an Inferno namespace import with imports false (swc-plugin-inferno has no imports: false option)
    // - Should keep an Inferno default import with imports false (swc-plugin-inferno has no imports: false option)

    #[test]
    fn should_keep_an_import_that_is_only_used_as_a_jsx_tag() {
        assert_js_eq(
            &transform_tsx(
                "{}",
                r#"import Foo from "./Foo";
export const a = <Foo/>;"#,
            ),
            r#"import { createComponentVNode } from "inferno";
import Foo from "./Foo";
export const a = createComponentVNode(2, Foo);"#,
        );
    }

    #[test]
    fn should_remove_type_imports_and_keep_component_imports() {
        assert_js_eq(
            &transform_tsx(
                "{}",
                r#"import { Foo } from "./Foo";
import type { P } from "./P";
export const a = <Foo<P> x={1 as number} y={z!} w={q satisfies P}/>;"#,
            ),
            r#"import { createComponentVNode } from "inferno";
import { Foo } from "./Foo";
export const a = createComponentVNode(2, Foo, {
  "x": 1,
  "y": z,
  "w": q
});"#,
        );
    }

    #[test]
    fn should_keep_an_existing_create_vnode_import() {
        assert_js_eq(
            &transform_tsx(
                "{}",
                r#"import { createVNode } from "inferno";
export const a = <div/>;"#,
            ),
            r#"import { createVNode } from "inferno";
export const a = createVNode(1, "div");"#,
        );
    }

    #[test]
    fn should_keep_an_existing_create_vnode_import_with_only_remove_type_imports() {
        assert_js_eq(
            &compile_ok(
                Setup {
                    lang: Lang::Tsx,
                    typescript: r#"{"verbatimModuleSyntax": true}"#,
                    ..Default::default()
                },
                r#"import { createVNode } from "inferno";
export const a = <div/>;"#,
            ),
            r#"import { createVNode } from "inferno";
export const a = createVNode(1, "div");"#,
        );
    }

    // @babel/preset-typescript keeps the import because React is its default JSX pragma.
    #[test]
    fn should_drop_an_unused_react_namespace_import() {
        assert_js_eq(
            &transform_tsx(
                "{}",
                r#"import * as React from "react";
export const a = <div/>;"#,
            ),
            r#"import { createVNode } from "inferno";
export const a = createVNode(1, "div");"#,
        );
    }

    // @babel/preset-typescript keeps imports of the jsxPragma. swc runs its TypeScript transform after
    // the plugin, when no JSX is left to use the pragma.
    #[test]
    fn should_drop_an_unused_jsx_pragma_import() {
        assert_js_eq(
            &compile_ok(
                Setup {
                    lang: Lang::Tsx,
                    jsx_pragma: Some("h"),
                    ..Default::default()
                },
                r#"import { h } from "preact";
export const a = <div/>;"#,
            ),
            r#"import { createVNode } from "inferno";
export const a = createVNode(1, "div");"#,
        );
    }
}
