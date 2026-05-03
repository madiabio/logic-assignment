use crate::ast::{Formula, Symbol, Term};
use crate::proof::quantifier::{
    fresh_branch_term_name, fresh_eigenconstant_name, instantiate_quantified_formula,
    instantiate_quantified_formula_with_term, visible_terms_in_sequent,
};
use crate::proof::rules::apply::RuleApplication;
use crate::proof::sequent::Sequent;

pub(crate) fn apply_forall_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(term) = visible_terms_in_sequent(sequent)
        .into_iter()
        .next()
        .or_else(|| Some(Term::Const(Symbol::User(fresh_branch_term_name(sequent)))))
    else {
        return RuleApplication::Error;
    };

    apply_forall_l_with_term(sequent, index, &term)
}

pub(crate) fn apply_forall_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::ForAll(vars, body)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let Some(replacement_formula) = instantiate_quantified_formula(
        vars,
        body,
        fresh_eigenconstant_name(sequent),
        Formula::ForAll,
    ) else {
        return RuleApplication::Error;
    };

    let mut right = Vec::with_capacity(sequent.right.len());
    right.extend(sequent.right[..index].iter().cloned());
    right.push(replacement_formula);
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left: sequent.left.clone(),
        right,
    }])
}

pub(crate) fn apply_forall_l_with_term(
    sequent: &Sequent,
    index: usize,
    term: &Term,
) -> RuleApplication {
    let Some(Formula::ForAll(vars, body)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let Some(instantiated_formula) =
        instantiate_quantified_formula_with_term(vars, body, term, Formula::ForAll)
    else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left[..=index].iter().cloned());
    left.push(instantiated_formula);
    left.extend(sequent.left[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left,
        right: sequent.right.clone(),
    }])
}

pub(crate) fn apply_exists_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Exists(vars, body)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let Some(replacement_formula) = instantiate_quantified_formula(
        vars,
        body,
        fresh_eigenconstant_name(sequent),
        Formula::Exists,
    ) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len());
    left.extend(sequent.left[..index].iter().cloned());
    left.push(replacement_formula);
    left.extend(sequent.left[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left,
        right: sequent.right.clone(),
    }])
}

pub(crate) fn apply_exists_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(term) = visible_terms_in_sequent(sequent)
        .into_iter()
        .next()
        .or_else(|| Some(Term::Const(Symbol::User(fresh_branch_term_name(sequent)))))
    else {
        return RuleApplication::Error;
    };

    apply_exists_r_with_term(sequent, index, &term)
}

pub(crate) fn apply_exists_r_with_term(
    sequent: &Sequent,
    index: usize,
    term: &Term,
) -> RuleApplication {
    let Some(Formula::Exists(vars, body)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let Some(instantiated_formula) =
        instantiate_quantified_formula_with_term(vars, body, term, Formula::Exists)
    else {
        return RuleApplication::Error;
    };

    let mut right = Vec::with_capacity(sequent.right.len() + 1);
    right.extend(sequent.right[..=index].iter().cloned());
    right.push(instantiated_formula);
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left: sequent.left.clone(),
        right,
    }])
}
