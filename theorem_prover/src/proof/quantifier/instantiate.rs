use crate::ast::{Formula, Symbol, Term};

pub(crate) fn instantiate_quantified_formula(
    vars: &[crate::ast::Var],
    body: &Formula,
    replacement_name: String,
    wrap_remaining: fn(Vec<crate::ast::Var>, Box<Formula>) -> Formula,
) -> Option<Formula> {
    let replacement = Term::Const(Symbol::User(replacement_name));
    instantiate_quantified_formula_with_term(vars, body, &replacement, wrap_remaining)
}

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
