//! Rule matching over the formulas currently visible in a sequent.

use crate::Sequent;
use crate::ast::Formula;
use crate::proof::rules::kinds::{Rule, RuleMatch, Side};

/// Finds every rule that can be applied immediately to the given sequent.
pub fn find_applicable_rules(sequent: &Sequent) -> Vec<RuleMatch> {
    let mut matches = Vec::new();

    for (index, left_formula) in sequent.left.iter().enumerate() {
        if sequent
            .right
            .iter()
            .any(|right_formula| right_formula == left_formula)
        {
            matches.push(RuleMatch {
                rule: Rule::Id,
                side: Side::Left,
                index,
            });
        }
    }

    for (index, formula) in sequent.left.iter().enumerate() {
        if let Some(rule) = left_rule(formula) {
            matches.push(RuleMatch {
                rule,
                side: Side::Left,
                index,
            });
        }
    }

    for (index, formula) in sequent.right.iter().enumerate() {
        if let Some(rule) = right_rule(formula) {
            matches.push(RuleMatch {
                rule,
                side: Side::Right,
                index,
            });
        }
    }

    matches
}

/// Maps a left-side formula to its corresponding left introduction rule, if any.
fn left_rule(formula: &Formula) -> Option<Rule> {
    match formula {
        Formula::False => Some(Rule::BottomL),
        Formula::And(_) => Some(Rule::AndL),
        Formula::Or(_) => Some(Rule::OrL),
        Formula::Implies(_, _) => Some(Rule::ImpliesL),
        Formula::Not(_) => Some(Rule::NotL),
        Formula::ForAll(_, _) => Some(Rule::ForAllL),
        Formula::Exists(_, _) => Some(Rule::ExistsL),
        _ => None,
    }
}

/// Maps a right-side formula to its corresponding right introduction rule, if any.
fn right_rule(formula: &Formula) -> Option<Rule> {
    match formula {
        Formula::True => Some(Rule::TopR),
        Formula::And(_) => Some(Rule::AndR),
        Formula::Or(_) => Some(Rule::OrR),
        Formula::Implies(_, _) => Some(Rule::ImpliesR),
        Formula::Not(_) => Some(Rule::NotR),
        Formula::ForAll(_, _) => Some(Rule::ForAllR),
        Formula::Exists(_, _) => Some(Rule::ExistsR),
        _ => None,
    }
}
