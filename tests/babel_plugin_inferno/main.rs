//! Test cases ported from babel-plugin-inferno (`tests.js` and `tests/*.test.js`).
//!
//! Each module ports one test file, and each nested module one of its `describe`
//! blocks. The expected code is babel-plugin-inferno's, compared after
//! normalization (see [`helpers::assert_js_eq`]), so formatting differences
//! between babel and swc do not matter.
//!
//! swc-plugin-inferno generates the same code as babel-plugin-inferno. The few
//! tests that expect something else say why in a comment: swc's parser and
//! TypeScript transform differ from babel's, swc renames clashing bindings
//! (hygiene), and swc-plugin-inferno adds pure annotations and rejects unknown
//! options. Errors are compared by message, and babel's code frames in the
//! format of swc's diagnostics. Tests of babel-only options and behaviour are
//! listed as "Not ported".

mod helpers;

mod attribute_tables;
mod attributes;
mod babel_parity;
mod bench_fixtures;
mod entities_strings;
mod expression_children;
mod fragments;
mod key_ref_hooks;
mod options_imports;
mod oxc_parity;
mod parser_errors;
mod positions;
mod preact_parity;
mod react_parity;
mod source_maps;
mod special_flags;
mod spread;
mod svg_attributes;
mod tag_names;
mod transforms;
mod tsx;
mod useless_flags;
mod whitespace_text;
