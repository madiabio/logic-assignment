//! Helpers for instantiating quantified formulas during proof search.

use crate::ast::{Formula, Symbol, Term};

/// Instantiates the leading quantified variable with a fresh constant name.
pub(crate) fn instantiate_quantified_formula(
    vars: &[crate::ast::Var],
    body: &Formula,
    replacement_name: String,
    wrap_remaining: fn(Vec<crate::ast::Var>, Box<Formula>) -> Formula,
) -> Option<Formula> {
    let replacement = Term::Const(Symbol::User(replacement_name));
    instantiate_quantified_formula_with_term(vars, body, &replacement, wrap_remaining)
}

/// Instantiates the leading quantified variable with a specific replacement term.
pub(crate) fn instantiate_quantified_formula_with_term(
    vars: &[crate::ast::Var],
    body: &Formula,
    replacement: &Term,
    wrap_remaining: fn(Vec<crate::ast::Var>, Box<Formula>) -> Formula,
) -> Option<Formula> {
    let (first_var, remaining_vars) = vars.split_first()?;
    let instantiated_body = body.substitute_var(&first_var.name, replacement);

    Some(if remaining_vars.is_empty() {
        instantiated_body
    } else {
        wrap_remaining(remaining_vars.to_vec(), Box::new(instantiated_body))
    })
}
