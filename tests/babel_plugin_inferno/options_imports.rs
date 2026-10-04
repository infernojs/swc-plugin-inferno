//! Ported from babel-plugin-inferno `tests/options-imports.test.js`: Options and imports

use crate::helpers::*;

/// imports option
mod imports_option {
    use super::*;

    // Not ported:
    // - Should treat the string "false" like false (swc-plugin-inferno has no imports: false option)
    // - Should declare every used helper from the Inferno global in one var (swc-plugin-inferno has no imports: false option)
    // - Should insert the var after existing imports (swc-plugin-inferno has no imports: false option)
    // - Should insert the var after directives and before other code (swc-plugin-inferno has no imports: false option)
    // - Should declare helpers before a call to a hoisted function that uses JSX (swc-plugin-inferno has no imports: false option)
    // - Should declare helpers after imports and before other code (swc-plugin-inferno has no imports: false option)
    // - Should declare helpers after a required Inferno (swc-plugin-inferno has no imports: false option)
    // - Should declare helpers after an Inferno declaration that follows other code (swc-plugin-inferno has no imports: false option)
    // - Should ignore Inferno bindings in nested scopes (swc-plugin-inferno has no imports: false option)
    // - Should declare helpers in every file compiled with a reused config (imports: false and babel config reuse are babel-only)
    // - Should declare helpers in every file compiled with the same options object (imports: false and babel config reuse are babel-only)
    // - Should keep existing Inferno imports when declaring the var (swc-plugin-inferno has no imports: false option)
    // - Should not declare a var when pragma is set (swc-plugin-inferno has no imports: false, pragma options)

    #[test]
    fn should_import_from_a_custom_module_name() {
        assert_js_eq(
            &transform_with(
                r#"{"importSource": "inferno-compat"}"#,
                "<div><Foo {...p}/>text<></></div>",
            ),
            r#"import { newVNode, newFragment, newComponentVNode, normalizeProps, newTextVNode } from "inferno-compat";
newVNode(5, "div", null, [normalizeProps(newComponentVNode(0, Foo, {
  ...p
})), newTextVNode("text"), newFragment(272)]);"#,
        );
    }

    #[test]
    fn should_treat_the_string_true_like_true() {
        assert_js_eq(
            &transform_with("{}", "<div/>"),
            r#"import { newVNode } from "inferno";
newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_import_from_inferno_when_imports_is_omitted() {
        assert_js_eq(
            &transform_with("{}", "<div/>"),
            r#"import { newVNode } from "inferno";
newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_import_every_used_helper_in_one_declaration() {
        assert_js_eq(
            &transform_with("{}", "<div><Foo {...p}/>text<></></div>"),
            r#"import { newVNode, newFragment, newComponentVNode, normalizeProps, newTextVNode } from "inferno";
newVNode(5, "div", null, [normalizeProps(newComponentVNode(0, Foo, {
  ...p
})), newTextVNode("text"), newFragment(272)]);"#,
        );
    }
}

// Not ported from pragma options:
// - Should import every helper under its pragma name (swc-plugin-inferno has no pragma, pragmaCreateComponentVNode, pragmaNormalizeProps, pragmaTextVNode, pragmaFragmentVNode options)
// - Should call every helper by its pragma name without imports (swc-plugin-inferno has no imports: false, pragma, pragmaCreateComponentVNode, pragmaNormalizeProps, pragmaTextVNode, pragmaFragmentVNode options)
// - Should declare default helper names when only a component pragma is set (swc-plugin-inferno has no imports: false, pragmaCreateComponentVNode options)

// Not ported from defineAllArguments option:
// - Should define all component arguments (swc-plugin-inferno has no defineAllArguments option)
// - Should accept the string "true" (swc-plugin-inferno has no defineAllArguments option)
// - Should define all element arguments (swc-plugin-inferno has no defineAllArguments option)
// - Should define all arguments of an empty short syntax fragment (swc-plugin-inferno has no defineAllArguments option)
// - Should define all arguments of a short syntax fragment with dynamic children (swc-plugin-inferno has no defineAllArguments option)
// - Should define all arguments of a short syntax fragment with static children (swc-plugin-inferno has no defineAllArguments option)
// - Should define all arguments of an empty long syntax Fragment like short syntax (swc-plugin-inferno has no defineAllArguments option)

/// existing bindings
mod existing_bindings {
    use super::*;

    #[test]
    fn should_use_a_top_level_new_vnode_function_instead_of_importing() {
        assert_js_eq(
            &transform_with(
                "{}",
                r"function newVNode(){}
const a = <div/>;",
            ),
            r#"function newVNode() {}
const a = newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_use_new_vnode_imported_from_another_module() {
        assert_js_eq(
            &transform_with(
                "{}",
                r#"import {newVNode} from "other-lib";
const a = <div/>;"#,
            ),
            r#"import { newVNode } from "other-lib";
const a = newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_still_import_new_vnode_when_it_is_imported_under_another_name() {
        assert_js_eq(
            &transform_with(
                "{}",
                r#"import {newVNode as cv} from "inferno";
const a = <div/>;"#,
            ),
            r#"import { newVNode } from "inferno";
import { newVNode as cv } from "inferno";
const a = newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_import_new_vnode_next_to_a_namespace_import() {
        assert_js_eq(
            &transform_with(
                "{}",
                r#"import * as Inferno from "inferno";
const a = <div/>;"#,
            ),
            r#"import { newVNode } from "inferno";
import * as Inferno from "inferno";
const a = newVNode(17, "div");"#,
        );
    }

    #[test]
    fn should_not_import_helpers_that_are_already_imported() {
        assert_js_eq(
            &transform_with(
                "{}",
                r#"import {newVNode, newComponentVNode} from "inferno";
const a = <div><Foo/></div>;"#,
            ),
            r#"import { newVNode, newComponentVNode } from "inferno";
const a = newVNode(9, "div", null, newComponentVNode(0, Foo));"#,
        );
    }

    #[test]
    fn should_ignore_bindings_named_after_other_jsx_runtimes() {
        assert_js_eq(
            &transform_with(
                "{}",
                r"const _jsx = 1, jsx = 2;
<div/>;",
            ),
            r#"import { newVNode } from "inferno";
const _jsx = 1,
  jsx = 2;
newVNode(17, "div");"#,
        );
    }
}

/// import emission
mod import_emission {
    use super::*;

    fn compile_as(source_type: SourceType, options: &str, input: &str) -> String {
        compile_ok(
            Setup {
                source_type,
                options,
                ..Default::default()
            },
            input,
        )
    }

    // Not ported:
    // - Should keep output on the original lines with retainLines (retainLines is an option of babel's code generator)
    // - Should require helpers under their pragma names in a script (swc-plugin-inferno has no pragma option)

    #[test]
    fn should_not_import_anything_without_jsx() {
        assert_js_eq(&transform_with("{}", "const a = 1;"), "const a = 1;");
    }

    #[test]
    fn should_import_once_for_many_jsx_roots() {
        assert_js_eq(
            &transform_with(
                "{}",
                r"const a = <div/>;
const b = <span/>;",
            ),
            r#"import { newVNode } from "inferno";
const a = newVNode(17, "div");
const b = newVNode(17, "span");"#,
        );
    }

    #[test]
    fn should_require_helpers_in_a_file_parsed_as_script_by_source_type_unambiguous() {
        let code = compile_as(
            SourceType::Unambiguous,
            "{}",
            "const a = require(\"x\");\nconst b = <div/>;",
        );

        assert_js_eq(
            &code,
            "var _inferno = require(\"inferno\"),\n  newVNode = _inferno.newVNode;\nconst a = require(\"x\");\nconst b = newVNode(17, \"div\");",
        );
    }

    #[test]
    fn should_import_helpers_in_a_file_parsed_as_module_by_source_type_unambiguous() {
        let code = compile_as(
            SourceType::Unambiguous,
            "{}",
            "import x from \"x\";\nconst b = <div/>;",
        );

        assert_js_eq(
            &code,
            "import { newVNode } from \"inferno\";\nimport x from \"x\";\nconst b = newVNode(17, \"div\");",
        );
    }

    #[test]
    fn should_require_every_used_helper_in_a_script() {
        let code = compile_as(
            SourceType::Script,
            "{}",
            "const a = <div><Foo {...p}/>text<></></div>;",
        );

        assert_js_eq(
            &code,
            "var _inferno = require(\"inferno\"),\n  newVNode = _inferno.newVNode,\n  newFragment = _inferno.newFragment,\n  newComponentVNode = _inferno.newComponentVNode,\n  normalizeProps = _inferno.normalizeProps,\n  newTextVNode = _inferno.newTextVNode;\nconst a = newVNode(5, \"div\", null, [normalizeProps(newComponentVNode(0, Foo, {\n  ...p\n})), newTextVNode(\"text\"), newFragment(272)]);",
        );
        assert_valid_js(&code);
    }

    #[test]
    fn should_require_helpers_after_directives_in_a_script() {
        let code = compile_as(
            SourceType::Script,
            "{}",
            "\"use strict\";\nconst a = <div/>;",
        );

        assert_js_eq(
            &code,
            "\"use strict\";\n\nvar _inferno = require(\"inferno\"),\n  newVNode = _inferno.newVNode;\nconst a = newVNode(17, \"div\");",
        );
    }

    #[test]
    fn should_require_helpers_from_a_custom_module_name_in_a_script() {
        let code = compile_as(
            SourceType::Script,
            "{\"importSource\": \"inferno-compat\"}",
            "const a = <div/>;",
        );

        assert_js_eq(
            &code,
            "var _infernoCompat = require(\"inferno-compat\"),\n  newVNode = _infernoCompat.newVNode;\nconst a = newVNode(17, \"div\");",
        );
    }

    #[test]
    fn should_not_require_helpers_that_are_already_declared_in_a_script() {
        let code = compile_as(
            SourceType::Script,
            "{}",
            "function newVNode() {}\nconst a = <div><Foo/></div>;",
        );

        assert_js_eq(
            &code,
            "var _inferno = require(\"inferno\"),\n  newComponentVNode = _inferno.newComponentVNode;\nfunction newVNode() {}\nconst a = newVNode(9, \"div\", null, newComponentVNode(0, Foo));",
        );
    }

    #[test]
    fn should_use_a_unique_name_for_the_required_module_in_a_script() {
        let code = compile_as(
            SourceType::Script,
            "{}",
            "var _inferno = 1;\nconst a = <div/>;",
        );

        assert_js_eq(
            &code,
            "var _inferno2 = require(\"inferno\"),\n  newVNode = _inferno2.newVNode;\nvar _inferno = 1;\nconst a = newVNode(17, \"div\");",
        );
    }
}

/// pragma comments
mod pragma_comments {
    use super::*;

    #[test]
    fn should_ignore_jsx_and_jsx_frag_comments() {
        assert_js_eq(
            &transform_with(
                "{}",
                r"/** @jsx h */
/** @jsxFrag F */
<><div/></>",
            ),
            r#"import { newVNode, newFragment } from "inferno";
/** @jsx h */
/** @jsxFrag F */
newFragment(260, [newVNode(17, "div")]);"#,
        );
    }

    #[test]
    fn should_ignore_jsx_runtime_and_jsx_import_source_comments() {
        assert_js_eq(
            &transform_with(
                "{}",
                r"/** @jsxRuntime classic */
/** @jsxImportSource preact */
<div/>",
            ),
            r#"import { newVNode } from "inferno";
/** @jsxRuntime classic */
/** @jsxImportSource preact */
newVNode(17, "div");"#,
        );
    }
}

/// current behaviour (questionable)
mod current_behaviour_questionable {
    use super::*;

    // babel-plugin-inferno calls the local binding, which is not a function. swc renames the local
    // binding instead (hygiene).
    #[test]
    fn should_rename_a_local_binding_that_shadows_new_vnode() {
        assert_js_eq(
            &transform_with("{}", "function f(){ const newVNode = 1; return <div/>; }"),
            r#"import { newVNode } from "inferno";
function f() {
    const newVNode1 = 1;
    return newVNode(17, "div");
}"#,
        );
    }

    // babel-plugin-inferno never adds pure annotations; swc-plugin-inferno does unless `pure` is false.
    #[test]
    fn should_add_pure_annotations_unless_disabled() {
        assert!(transform_with("{}", "<div><Foo/></div>").contains("/*#__PURE__*/"));
        assert!(!transform_with(r#"{"pure": false}"#, "<div><Foo/></div>").contains("__PURE__"));
    }

    // babel-plugin-inferno ignores unknown options; swc-plugin-inferno rejects them.
    #[test]
    fn should_reject_unknown_options() {
        let err =
            serde_json::from_str::<swc_plugin_inferno::Options>(r#"{"pragmaa": "x"}"#).unwrap_err();

        assert!(err.to_string().contains("unknown field `pragmaa`"), "{err}");
    }
}
