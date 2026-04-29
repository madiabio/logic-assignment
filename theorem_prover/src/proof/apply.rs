// Implements the logic to apply a rule on a sequent.
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

        _ => RuleApplication::NotImplemented,
    }
}
