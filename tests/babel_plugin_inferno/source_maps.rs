//! Ported from babel-plugin-inferno `tests/source-maps.test.js`: Source maps

use crate::helpers::*;

const INPUT: &str = r#"const x = 1;

const el = (
  <div className="a">
    <Foo bar={x} />
    text
  </div>
);
const f = (
  <>
    <b {...p} />
  </>
);"#;

fn compile() -> Compiled {
    compile_with(Setup::default(), INPUT, noop_pass()).unwrap()
}

fn at(line: usize, column: usize) -> Option<Position> {
    Some(Position { line, column })
}

#[test]
fn should_map_an_element_call_to_its_opening_tag() {
    assert_eq!(
        original_position(&compile(), r#"createVNode(1, "div""#),
        at(4, 2)
    );
}

#[test]
fn should_map_a_component_call_to_its_opening_tag() {
    assert_eq!(
        original_position(&compile(), "createComponentVNode(2"),
        at(5, 4)
    );
}

#[test]
fn should_map_a_text_vnode_to_its_jsx_text() {
    assert_eq!(original_position(&compile(), "createTextVNode("), at(5, 19));
}

#[test]
fn should_map_a_fragment_call_to_its_opening_tag() {
    assert_eq!(original_position(&compile(), "createFragment("), at(10, 2));
}

#[test]
fn should_map_normalize_props_and_the_call_it_wraps_to_the_opening_tag() {
    let compiled = compile();

    assert_eq!(original_position(&compiled, "normalizeProps("), at(11, 4));
    assert_eq!(
        original_position(&compiled, r#"createVNode(1, "b""#),
        at(11, 4)
    );
}

/// The input position mapped from exactly where `needle` starts, without falling back to an
/// earlier mapping on the line like `original_position`
fn exact_position(compiled: &Compiled, needle: &str) -> Option<Position> {
    let (line, column) = compiled.code.lines().enumerate().find_map(|(i, line)| {
        line.find(needle)
            .map(|byte| (i + 1, line[..byte].chars().count()))
    })?;

    compiled
        .mappings
        .iter()
        .find(|m| m.generated.line == line && m.generated.column == column)
        .map(|m| m.original)
}

// babel-plugin-inferno maps both normalizeProps and the call it wraps to the opening tag. The
// wrapped call has a mapping of its own, where its annotated expression starts.
#[test]
fn should_map_the_call_wrapped_by_normalize_props_itself() {
    assert_eq!(
        exact_position(&compile(), r#"/*#__PURE__*/ createVNode(1, "b""#),
        at(11, 4)
    );
}
