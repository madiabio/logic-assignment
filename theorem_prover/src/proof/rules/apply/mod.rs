use crate::proof::rules::{Rule, RuleMatch};
use crate::proof::sequent::Sequent;

pub mod connective;
pub mod quantifier;
pub mod structural;

pub(crate) use quantifier::{apply_exists_r_with_term, apply_forall_l_with_term};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleApplication {
    Closed,
    Premises(Vec<Sequent>),
    NotImplemented,
    Error,
}

pub fn apply_rule(sequent: &Sequent, rule_match: &RuleMatch) -> RuleApplication {
    match rule_match.rule {
        Rule::Id | Rule::TopR | Rule::BottomL => structural::apply_structural(rule_match.rule),
        Rule::AndL => connective::apply_and_l(sequent, rule_match.index),
        Rule::OrR => connective::apply_or_r(sequent, rule_match.index),
        Rule::ImpliesR => connective::apply_implies_r(sequent, rule_match.index),
        Rule::NotL => connective::apply_not_l(sequent, rule_match.index),
        Rule::NotR => connective::apply_not_r(sequent, rule_match.index),
        Rule::ForAllL => quantifier::apply_forall_l(sequent, rule_match.index),
        Rule::ForAllR => quantifier::apply_forall_r(sequent, rule_match.index),
        Rule::ExistsL => quantifier::apply_exists_l(sequent, rule_match.index),
        Rule::ExistsR => quantifier::apply_exists_r(sequent, rule_match.index),
        Rule::AndR => connective::apply_and_r(sequent, rule_match.index),
        Rule::OrL => connective::apply_or_l(sequent, rule_match.index),
        Rule::ImpliesL => connective::apply_implies_l(sequent, rule_match.index),
    }
}
