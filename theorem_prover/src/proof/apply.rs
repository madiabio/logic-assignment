// Implements the logic to apply a rule on a sequent.
use crate::ast::{Formula, Symbol, Term};
use crate::proof::quantifier::{
    fresh_branch_term_name, fresh_eigenconstant_name, instantiate_quantified_formula,
    instantiate_quantified_formula_with_term, visible_terms_in_sequent,
};
use crate::proof::rules::Rule;
use crate::proof::rules::RuleMatch;
use crate::proof::sequent::Sequent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleApplication {
    Closed,
    Premises(Vec<Sequent>),
    NotImplemented,
    Error,
}

pub fn apply_rule(sequent: &Sequent, rule_match: &RuleMatch) -> RuleApplication {
    match rule_match.rule {
        // All of these rules close a branch.
        Rule::Id | Rule::TopR | Rule::BottomL => RuleApplication::Closed,

        // These are not branch closing, and don't create a new branch
        Rule::AndL => apply_and_l(sequent, rule_match.index),
        Rule::OrR => apply_or_r(sequent, rule_match.index),
        Rule::ImpliesR => apply_implies_r(sequent, rule_match.index),
        Rule::NotL => apply_not_l(sequent, rule_match.index),
        Rule::NotR => apply_not_r(sequent, rule_match.index),
        Rule::ForAllL => apply_forall_l(sequent, rule_match.index),
        Rule::ForAllR => apply_forall_r(sequent, rule_match.index),
        Rule::ExistsL => apply_exists_l(sequent, rule_match.index),
        Rule::ExistsR => apply_exists_r(sequent, rule_match.index),

        // These are branch closing, and create new branch
        Rule::AndR => apply_and_r(sequent, rule_match.index),
        Rule::OrL => apply_or_l(sequent, rule_match.index),
        Rule::ImpliesL => apply_implies_l(sequent, rule_match.index),
    }
}

fn apply_forall_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(term) = visible_terms_in_sequent(sequent)
        .into_iter()
        .next()
        .or_else(|| Some(Term::Const(Symbol::User(fresh_branch_term_name(sequent)))))
    else {
        return RuleApplication::Error;
    };

    apply_forall_l_with_term(sequent, index, &term)
}

fn apply_and_l(sequent: &Sequent, index: usize) -> RuleApplication {
    // TODO: Check the NotImplemented stuff, it probs shouldnt be returning NotImplemented
    let Some(Formula::And(items)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left[..index].iter().cloned());

    // This prover keeps conjunctions in a binary step form, so an n-ary
    // conjunction is peeled from the front and the remainder stays grouped.
    left.push(items[0].clone());
    if items.len() == 2 {
        left.push(items[1].clone());
    } else {
        left.push(Formula::And(items[1..].to_vec()));
    }

    left.extend(sequent.left[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left,
        right: sequent.right.clone(),
    }])
}

fn apply_and_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::And(items)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    // Gamma |- A /\ B, Delta branches into Gamma |- A, Delta and
    // Gamma |- B, Delta. For n-ary conjunctions, keep the tail grouped so
    // repeated AndR applications continue shrinking the formula.
    let mut first_right = Vec::with_capacity(sequent.right.len());
    first_right.extend(sequent.right[..index].iter().cloned());
    first_right.push(items[0].clone());
    first_right.extend(sequent.right[index + 1..].iter().cloned());

    let mut second_right = Vec::with_capacity(sequent.right.len());
    second_right.extend(sequent.right[..index].iter().cloned());
    if items.len() == 2 {
        second_right.push(items[1].clone());
    } else {
        second_right.push(Formula::And(items[1..].to_vec()));
    }
    second_right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![
        Sequent {
            left: sequent.left.clone(),
            right: first_right,
        },
        Sequent {
            left: sequent.left.clone(),
            right: second_right,
        },
    ])
}

fn apply_or_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Or(items)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    // Gamma, A \/ B |- Delta branches into Gamma, A |- Delta and
    // Gamma, B |- Delta. For n-ary disjunctions, keep the tail grouped so
    // repeated OrL applications continue shrinking the formula.
    let mut first_left = Vec::with_capacity(sequent.left.len());
    first_left.extend(sequent.left[..index].iter().cloned());
    first_left.push(items[0].clone());
    first_left.extend(sequent.left[index + 1..].iter().cloned());

    let mut second_left = Vec::with_capacity(sequent.left.len());
    second_left.extend(sequent.left[..index].iter().cloned());
    if items.len() == 2 {
        second_left.push(items[1].clone());
    } else {
        second_left.push(Formula::Or(items[1..].to_vec()));
    }
    second_left.extend(sequent.left[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![
        Sequent {
            left: first_left,
            right: sequent.right.clone(),
        },
        Sequent {
            left: second_left,
            right: sequent.right.clone(),
        },
    ])
}

fn apply_or_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Or(items)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    let mut right = Vec::with_capacity(sequent.right.len() + 1);
    right.extend(sequent.right[..index].iter().cloned());

    // Mirror AndL on the right: split off one disjunct and keep the tail as a
    // single formula so repeated rule applications continue reducing it.
    right.push(items[0].clone());
    if items.len() == 2 {
        right.push(items[1].clone());
    } else {
        right.push(Formula::Or(items[1..].to_vec()));
    }

    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left: sequent.left.clone(),
        right,
    }])
}

fn apply_implies_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Implies(left_formula, right_formula)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left.iter().cloned());
    // Gamma |- A -> B becomes Gamma, A |- B.
    left.push((**left_formula).clone());

    let mut right = Vec::with_capacity(sequent.right.len());
    right.extend(sequent.right[..index].iter().cloned());
    right.push((**right_formula).clone());
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent { left, right }])
}

fn apply_implies_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Implies(left_formula, right_formula)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let mut left_without_implication = Vec::with_capacity(sequent.left.len().saturating_sub(1));
    left_without_implication.extend(sequent.left[..index].iter().cloned());
    left_without_implication.extend(sequent.left[index + 1..].iter().cloned());

    let mut first_right = Vec::with_capacity(sequent.right.len() + 1);
    first_right.extend(sequent.right.iter().cloned());
    // Gamma, A -> B |- Delta becomes Gamma |- Delta, A.
    first_right.push((**left_formula).clone());

    let mut second_left = Vec::with_capacity(left_without_implication.len() + 1);
    second_left.extend(left_without_implication.iter().cloned());
    // The second branch keeps Gamma and adds the consequent B on the left.
    second_left.push((**right_formula).clone());

    RuleApplication::Premises(vec![
        Sequent {
            left: left_without_implication,
            right: first_right,
        },
        Sequent {
            left: second_left,
            right: sequent.right.clone(),
        },
    ])
}

fn apply_not_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Not(inner)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len().saturating_sub(1));
    left.extend(sequent.left[..index].iter().cloned());
    left.extend(sequent.left[index + 1..].iter().cloned());

    let mut right = Vec::with_capacity(sequent.right.len() + 1);
    right.extend(sequent.right.iter().cloned());
    // Gamma, ~A |- Delta becomes Gamma |- Delta, A.
    right.push((**inner).clone());

    RuleApplication::Premises(vec![Sequent { left, right }])
}

fn apply_not_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Not(inner)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left.iter().cloned());
    // Gamma |- ~A becomes Gamma, A |- by moving the negated formula left.
    left.push((**inner).clone());

    let mut right = Vec::with_capacity(sequent.right.len().saturating_sub(1));
    right.extend(sequent.right[..index].iter().cloned());
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent { left, right }])
}

fn apply_forall_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::ForAll(vars, body)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let Some(replacement_formula) = instantiate_quantified_formula(
        vars,
        body,
        fresh_eigenconstant_name(sequent), // get arbitraty constant 'a' that doesnt occur in the conclusion
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

fn apply_exists_l(sequent: &Sequent, index: usize) -> RuleApplication {
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

fn apply_exists_r(sequent: &Sequent, index: usize) -> RuleApplication {
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
