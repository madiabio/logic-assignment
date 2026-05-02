// Implements the logic to apply a rule on a sequent.
use crate::ast::Formula;
use crate::proof::rules::Rule;
use crate::proof::rules::RuleMatch;
use crate::proof::sequent::Sequent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleApplication {
    Closed,
    Premises(Vec<Sequent>),
    NotImplemented,
}

pub fn apply_rule(sequent: &Sequent, rule_match: &RuleMatch) -> RuleApplication {
    match rule_match.rule {
        // All of these rules close a branch.
        Rule::Id | Rule::TopR | Rule::BottomL => RuleApplication::Closed,
        Rule::AndL => apply_and_l(sequent, rule_match.index),
        Rule::OrR => apply_or_r(sequent, rule_match.index),
        Rule::ImpliesR => apply_implies_r(sequent, rule_match.index),
        Rule::NotR => apply_not_r(sequent, rule_match.index),

        _ => RuleApplication::NotImplemented,
    }
}

fn apply_and_l(sequent: &Sequent, index: usize) -> RuleApplication {
    // TODO: Check the NotImplemented stuff, it probs shouldnt be returning NotImplemented
    let Some(Formula::And(items)) = sequent.left.get(index) else {
        return RuleApplication::NotImplemented;
    };

    if items.len() < 2 {
        return RuleApplication::NotImplemented;
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

fn apply_or_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Or(items)) = sequent.right.get(index) else {
        return RuleApplication::NotImplemented;
    };

    if items.len() < 2 {
        return RuleApplication::NotImplemented;
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
        return RuleApplication::NotImplemented;
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

fn apply_not_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Not(inner)) = sequent.right.get(index) else {
        return RuleApplication::NotImplemented;
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
