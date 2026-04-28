use crate::ast::Formula;
use crate::sequent::Sequent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Id,
    TopR,
    BottomL,
    AndL,
    AndR,
    OrL,
    OrR,
    ImpliesL,
    ImpliesR,
    NotL,
    NotR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleMatch {
    pub rule: Rule,
    pub side: Side,
    pub index: usize,
}

pub fn find_applicable_rules(sequent: &Sequent) -> Vec<RuleMatch> {
    let mut matches = Vec::new();

    if let Some(index) = sequent.left.iter().position(|left_formula| {
        sequent
            .right
            .iter()
            .any(|right_formula| right_formula == left_formula)
    }) {
        matches.push(RuleMatch {
            rule: Rule::Id,
            side: Side::Left,
            index,
        });
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

fn left_rule(formula: &Formula) -> Option<Rule> {
    match formula {
        Formula::False => Some(Rule::BottomL),
        Formula::And(_) => Some(Rule::AndL),
        Formula::Or(_) => Some(Rule::OrL),
        Formula::Implies(_, _) => Some(Rule::ImpliesL),
        Formula::Not(_) => Some(Rule::NotL),
        _ => None,
    }
}

fn right_rule(formula: &Formula) -> Option<Rule> {
    match formula {
        Formula::True => Some(Rule::TopR),
        Formula::And(_) => Some(Rule::AndR),
        Formula::Or(_) => Some(Rule::OrR),
        Formula::Implies(_, _) => Some(Rule::ImpliesR),
        Formula::Not(_) => Some(Rule::NotR),
        _ => None,
    }
}
