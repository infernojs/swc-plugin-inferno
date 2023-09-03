use swc_common::{comments::SingleThreadedComments, sync::Lrc, FileName, Mark, SourceMap};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{Parser, StringInput};
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_testing::Tester;
use swc_ecma_visit::FoldWith;

use super::*;

fn parse(
    tester: &mut Tester,
    src: &str,
) -> Result<(Module, Lrc<SourceMap>, Lrc<SingleThreadedComments>), ()> {
    let syntax = ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
        jsx: true,
        ..Default::default()
    });
    let source_map = Lrc::new(SourceMap::default());
    let source_file = source_map.new_source_file(FileName::Anon, src.into());

    let comments = Lrc::new(SingleThreadedComments::default());
    let module = {
        let mut p = Parser::new(syntax, StringInput::from(&*source_file), Some(&comments));
        let res = p
            .parse_module()
            .map_err(|e| e.into_diagnostic(tester.handler).emit());

        for e in p.take_errors() {
            e.into_diagnostic(tester.handler).emit()
        }

        res?
    };

    Ok((module, source_map, comments))
}

fn emit(
    source_map: Lrc<SourceMap>,
    comments: Lrc<SingleThreadedComments>,
    program: &Module,
) -> String {
    let mut src_map_buf = vec![];
    let mut buf = vec![];
    {
        let writer = Box::new(JsWriter::new(
            source_map.clone(),
            "\n",
            &mut buf,
            Some(&mut src_map_buf),
        ));
        let mut emitter = Emitter {
            cfg: Default::default(),
            comments: Some(&comments),
            cm: source_map,
            wr: writer,
        };
        emitter.emit_module(program).unwrap();
    }

    String::from_utf8(buf).unwrap()
}

fn run_test(input: &str, expected: &str) {
    Tester::run(|tester| {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        let (actual, actual_sm, actual_comments) = parse(tester, input)?;
        let actual = actual
            .fold_with(&mut resolver(unresolved_mark, top_level_mark, false))
            .fold_with(&mut crate::inferno(
                actual_sm.clone(),
                Some(&actual_comments),
                Default::default(),
                top_level_mark,
                unresolved_mark,
            ));

        let actual_src = emit(actual_sm, actual_comments, &actual);

        let (expected, expected_sm, expected_comments) = parse(tester, expected)?;
        let expected_src = emit(expected_sm, expected_comments, &expected);

        if actual_src != expected_src {
            println!(">>>>> Orig <<<<<\n{}", input);
            println!(">>>>> Code <<<<<\n{}", actual_src);
            panic!(
                r#"assertion failed: `(left == right)`
    {}"#,
                ::testing::diff(&actual_src, &expected_src),
            );
        }

        Ok(())
    });
}

macro_rules! test {
    ($test_name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $test_name() {
            run_test($input, $expected)
        }
    };
}

test!(
    forward_ref,
    r#"
  import {forwardRef} from 'inferno';
  const Comp = forwardRef((props, ref) => null);
  "#,
    r#"
  import {forwardRef} from 'inferno';
  const Comp = /*#__PURE__*/ forwardRef((props, ref) => null);
  "#
);

test!(
    create_element,
    r#"
  import Inferno from 'inferno';
  Inferno.createVNode(1, "div");
  "#,
    r#"
  import Inferno from 'inferno';
  /*#__PURE__*/ Inferno.createVNode(1, "div");
  "#
);

test!(
    create_element_jsx,
    r#"
  import Inferno from 'inferno';
  const x = <div />;
  "#,
    r#"
    import { createVNode } from "inferno";
    import Inferno from 'inferno';
  const x = /*#__PURE__*/ createVNode(1, "div");
  "#
);

test!(
    create_element_fragment_jsx,
    r#"
  import Inferno from 'inferno';
  const x = <><div /></>;
  "#,
    r#"
    import { createVNode, createFragment } from "inferno";
    import Inferno from 'inferno';
    const x = /*#__PURE__*/ createFragment([
        /*#__PURE__*/ createVNode(1, "div")
    ], 4);
  "#
);

test!(
    clone_element,
    r#"
  import Inferno from 'inferno';
  Inferno.directClone(Inferno.createVNode(1, "div"));
  "#,
    r#"
  import Inferno from 'inferno';
  /*#__PURE__*/ Inferno.directClone(/*#__PURE__*/ Inferno.createVNode(1, "div"));
  "#
);

test!(
    create_ref,
    r#"
  import Inferno from 'inferno';
  Inferno.createRef();
  "#,
    r#"
  import Inferno from 'inferno';
  /*#__PURE__*/ Inferno.createRef();
  "#
);

test!(
    create_portal,
    r#"
  import * as Inferno from 'inferno';

  const Portal = Inferno.createPortal(Inferno.createVNode(1, "div"), document.getElementById('test'));
  "#,
    r#"
  import * as Inferno from 'inferno';
  
  const Portal = /*#__PURE__*/Inferno.createPortal( /*#__PURE__*/Inferno.createVNode(1, "div"), document.getElementById('test'));
  "#
);

test!(
    non_pure_ported_noop_hooks,
    r#"
  import {useState} from 'inferno';
  useState(2);
  "#,
    r#"
  import {useState} from 'inferno';
  useState(2);
  "#
);

test!(
    non_pure_inferno_dom,
    r#"
  import Inferno from 'inferno';
  Inferno.render(Inferno.createVNode(1, "div"));
  "#,
    r#"
  import Inferno from 'inferno';
  Inferno.render(/*#__PURE__*/Inferno.createVNode(1, "div"));
  "#
);

test!(
    non_inferno_named,
    r#"
  import {createElement} from 'foo';
  createElement('hi');
  "#,
    r#"
  import {createElement} from 'foo';
  createElement('hi');
  "#
);

test!(
    non_inferno_namespace,
    r#"
  import * as foo from 'foo';
  foo.createElement('hi');
  "#,
    r#"
  import * as foo from 'foo';
  foo.createElement('hi');
  "#
);

test!(
    non_inferno_default,
    r#"
  import foo from 'foo';
  foo.createElement('hi');
  "#,
    r#"
  import foo from 'foo';
  foo.createElement('hi');
  "#
);
