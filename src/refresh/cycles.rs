//! Custom hooks of a module that call each other.
//!
//! The refresh runtime computes the key of a signature from the keys of the custom hooks it
//! lists, and recurses forever when hooks list each other in a cycle, a hook calling itself
//! included. The calls that close a cycle are left out of the hooks arrays; the signature keys
//! keep them, so editing such a call still changes the signature.

use super::hook::{hooks_called_in, hooks_called_in_expr};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::ecma::{
    ast::*,
    visit::{Visit, VisitWith, noop_visit_type},
};

/// The calls between the hooks of a module that close a cycle
pub(crate) struct HookCycles {
    /// (caller, callee), by the names of the functions
    calls: FxHashSet<(Id, Id)>,
    /// The other names of functions: the name of a function expression -> its variable
    aliases: FxHashMap<Id, Id>,
}

impl HookCycles {
    pub(crate) fn find(items: &[ModuleItem]) -> Self {
        let mut graph = Graph::default();
        for item in items {
            item.visit_with(&mut graph);
        }
        graph.cycles()
    }

    /// Whether `function` calling `hook` closes a cycle
    pub(crate) fn contains(&self, function: &Id, hook: &Id) -> bool {
        !self.calls.is_empty()
            && self
                .calls
                .contains(&(name(&self.aliases, function), name(&self.aliases, hook)))
    }
}

fn name(aliases: &FxHashMap<Id, Id>, function: &Id) -> Id {
    aliases.get(function).unwrap_or(function).clone()
}

/// The named functions of a module and the functions they call as hooks
#[derive(Default)]
struct Graph {
    /// Functions that call hooks, in source order
    functions: Vec<Id>,
    hooks: FxHashMap<Id, Vec<Id>>,
    aliases: FxHashMap<Id, Id>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visited {
    /// On the path of the depth-first search
    Active,
    Done,
}

impl Graph {
    fn add(&mut self, function: Id, hooks: Vec<Id>) {
        // A function that calls no hooks is on no cycle
        if !hooks.is_empty() {
            self.functions.push(function.clone());
            self.hooks.insert(function, hooks);
        }
    }

    fn add_function(&mut self, name: Id, function: &Function) {
        if let Some(body) = &function.body {
            self.add(name, hooks_called_in(&body.stmts));
        }
    }

    fn add_arrow(&mut self, name: Id, arrow: &ArrowExpr) {
        let hooks = match &*arrow.body {
            ArrowFunctionBody::FunctionBody(body) => hooks_called_in(&body.stmts),
            ArrowFunctionBody::Expr(expr) => hooks_called_in_expr(expr),
            #[cfg(swc_ast_unknown)]
            _ => Vec::new(),
        };
        self.add(name, hooks);
    }

    /// The calls that close a cycle: the calls to a function on the path of a depth-first search
    /// that starts from the functions in source order
    fn cycles(self) -> HookCycles {
        let mut visited = FxHashMap::default();
        let mut calls = FxHashSet::default();

        for function in &self.functions {
            if !visited.contains_key(function) {
                self.visit(function, &mut visited, &mut calls);
            }
        }

        HookCycles {
            calls,
            aliases: self.aliases,
        }
    }

    fn visit(
        &self,
        function: &Id,
        visited: &mut FxHashMap<Id, Visited>,
        calls: &mut FxHashSet<(Id, Id)>,
    ) {
        visited.insert(function.clone(), Visited::Active);

        for hook in self.hooks.get(function).into_iter().flatten() {
            let hook = name(&self.aliases, hook);

            match visited.get(&hook) {
                Some(Visited::Active) => {
                    calls.insert((function.clone(), hook));
                }
                Some(Visited::Done) => {}
                None if self.hooks.contains_key(&hook) => self.visit(&hook, visited, calls),
                None => {}
            }
        }

        visited.insert(function.clone(), Visited::Done);
    }
}

impl Visit for Graph {
    noop_visit_type!();

    fn visit_fn_decl(&mut self, declaration: &FnDecl) {
        self.add_function(declaration.ident.to_id(), &declaration.function);
        declaration.visit_children_with(self);
    }

    fn visit_fn_expr(&mut self, expr: &FnExpr) {
        if let Some(ident) = &expr.ident {
            self.add_function(ident.to_id(), &expr.function);
        }
        expr.visit_children_with(self);
    }

    fn visit_var_declarator(&mut self, declarator: &VarDeclarator) {
        // A function assigned to a variable is known by the variable's name, which fast refresh
        // registers its signature with
        if let (Pat::Ident(binding), Some(init)) = (&declarator.name, &declarator.init) {
            match &**init {
                Expr::Fn(FnExpr { ident, function }) => {
                    if let Some(ident) = ident {
                        self.aliases.insert(ident.to_id(), binding.to_id());
                    }
                    self.add_function(binding.to_id(), function);
                    function.visit_children_with(self);
                    return;
                }
                Expr::Arrow(arrow) => {
                    self.add_arrow(binding.to_id(), arrow);
                    arrow.visit_children_with(self);
                    return;
                }
                _ => {}
            }
        }
        declarator.visit_children_with(self);
    }
}
