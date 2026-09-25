//! Program scope bindings, like the `scope.hasBinding` and `scope.generateUidIdentifier`
//! calls of babel-plugin-inferno.

use super::{HELPER_COUNT, Helper};
use crate::program_bindings::{for_each_module_binding, for_each_script_binding};
use rustc_hash::FxHashSet;
use swc_core::{
    atoms::Atom,
    common::SyntaxContext,
    ecma::{
        ast::*,
        visit::{Visit, VisitWith, noop_visit_type},
    },
};

/// The context of each helper that the program declares itself, or `None`
pub(super) type HelperBindings = [Option<SyntaxContext>; HELPER_COUNT];

/// The helpers declared in the program scope of a module
pub(super) fn module_bindings(module: &Module) -> HelperBindings {
    let mut bindings = HelperBindings::default();
    for_each_module_binding(&module.body, |ident| add_helper(&mut bindings, ident));
    bindings
}

/// The helpers declared in the program scope of a script
pub(super) fn script_bindings(script: &Script) -> HelperBindings {
    let mut bindings = HelperBindings::default();
    for_each_script_binding(&script.body, |ident| add_helper(&mut bindings, ident));
    bindings
}

fn add_helper(bindings: &mut HelperBindings, ident: &Ident) {
    if let Some(helper) = Helper::from_name(&ident.sym) {
        // The first declaration wins
        bindings[helper as usize].get_or_insert(ident.ctxt);
    }
}

/// Every identifier name used in a script
pub(super) fn used_names(script: &Script) -> FxHashSet<Atom> {
    struct Names(FxHashSet<Atom>);

    impl Visit for Names {
        noop_visit_type!();

        fn visit_ident(&mut self, ident: &Ident) {
            self.0.insert(ident.sym.clone());
        }
    }

    let mut names = Names(FxHashSet::default());
    script.visit_with(&mut names);
    names.0
}

/// A name that is not used in the program, like `scope.generateUidIdentifier(name)` in babel:
/// `inferno` becomes `_inferno`, or `_inferno2` when that is taken.
pub(super) fn generate_uid(name: &str, used: &FxHashSet<Atom>) -> Atom {
    let name = to_identifier(name);
    let name = name
        .trim_start_matches('_')
        .trim_end_matches(|c: char| c.is_ascii_digit());

    let mut uid = Atom::from(format!("_{name}"));
    for i in 2.. {
        if !used.contains(&uid) {
            break;
        }
        uid = format!("_{name}{i}").into();
    }
    uid
}

/// `toIdentifier` of `@babel/types`: `inferno-compat` becomes `infernoCompat`
fn to_identifier(input: &str) -> String {
    let name: String = input
        .chars()
        .map(|c| if Ident::is_valid_continue(c) { c } else { '-' })
        .collect();
    let name = name.trim_start_matches(|c: char| c == '-' || c.is_ascii_digit());

    let mut camel_cased = String::with_capacity(name.len());
    let mut upper_next = false;
    for c in name.chars() {
        if c == '-' {
            upper_next = true;
        } else if upper_next {
            camel_cased.extend(c.to_uppercase());
            upper_next = false;
        } else {
            camel_cased.push(c);
        }
    }

    // babel prefixes a reserved word with `_` here, which generate_uid strips again
    camel_cased
}
