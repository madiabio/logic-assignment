//! Application of propositional connective rules.

use crate::ast::Formula;
use crate::proof::rules::apply::RuleApplication;
use crate::proof::sequent::Sequent;

/// Applies `∧L` by expanding the selected conjunction on the left.
pub(crate) fn apply_and_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::And(items)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left[..index].iter().cloned());
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

/// Applies `∧R` by splitting the selected conjunction on the right into two branches.
pub(crate) fn apply_and_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::And(items)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

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

/// Applies `∨L` by branching on the selected disjunction on the left.
pub(crate) fn apply_or_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Or(items)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

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

/// Applies `∨R` by expanding the selected disjunction on the right in place.
pub(crate) fn apply_or_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Or(items)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    let mut right = Vec::with_capacity(sequent.right.len() + 1);
    right.extend(sequent.right[..index].iter().cloned());
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

/// Applies `→R` by moving the antecedent to the left and keeping the consequent on the right.
pub(crate) fn apply_implies_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Implies(left_formula, right_formula)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left.iter().cloned());
    left.push((**left_formula).clone());

    let mut right = Vec::with_capacity(sequent.right.len());
    right.extend(sequent.right[..index].iter().cloned());
    right.push((**right_formula).clone());
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent { left, right }])
}

/// Applies `→L` by branching over the implication's antecedent and consequent.
pub(crate) fn apply_implies_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Implies(left_formula, right_formula)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let mut left_without_implication = Vec::with_capacity(sequent.left.len().saturating_sub(1));
    left_without_implication.extend(sequent.left[..index].iter().cloned());
    left_without_implication.extend(sequent.left[index + 1..].iter().cloned());

    let mut first_right = Vec::with_capacity(sequent.right.len() + 1);
    first_right.extend(sequent.right.iter().cloned());
    first_right.push((**left_formula).clone());

    let mut second_left = Vec::with_capacity(left_without_implication.len() + 1);
    second_left.extend(left_without_implication.iter().cloned());
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

/// Applies `¬L` by moving the negated formula to the right.
pub(crate) fn apply_not_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Not(inner)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len().saturating_sub(1));
    left.extend(sequent.left[..index].iter().cloned());
    left.extend(sequent.left[index + 1..].iter().cloned());

    let mut right = Vec::with_capacity(sequent.right.len() + 1);
    right.extend(sequent.right.iter().cloned());
    right.push((**inner).clone());

    RuleApplication::Premises(vec![Sequent { left, right }])
}

/// Applies `¬R` by moving the negated formula to the left.
pub(crate) fn apply_not_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Not(inner)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left.iter().cloned());
    left.push((**inner).clone());

    let mut right = Vec::with_capacity(sequent.right.len().saturating_sub(1));
    right.extend(sequent.right[..index].iter().cloned());
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent { left, right }])
}
