//! Test cases ported from babel-plugin-inferno (`tests.js` and `tests/*.test.js`).
//!
//! Each module ports one test file, and each nested module one of its `describe`
//! blocks. The expected code is babel-plugin-inferno's, compared after
//! normalization (see [`helpers::assert_js_eq`]), so formatting differences
//! between babel and swc do not matter.
//!
//! Where swc-plugin-inferno differs from babel-plugin-inferno:
//!
//! - When both produce code that behaves the same, or the difference is
//!   intended, the test expects swc-plugin-inferno's code and a comment says
//!   what babel-plugin-inferno generates.
//! - When swc-plugin-inferno behaves differently, the test keeps
//!   babel-plugin-inferno's expectation and is `#[ignore]`d with the reason.
//!   Run them with `cargo test --test babel_plugin_inferno -- --ignored`.
//! - Tests of babel-only options and behaviour are listed as "Not ported".

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
mod whitespace_text;
