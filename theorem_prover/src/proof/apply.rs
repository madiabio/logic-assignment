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

        _ => RuleApplication::NotImplemented,
    }
}

fn apply_and_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::And(items)) = sequent.left.get(index) else {
        return RuleApplication::NotImplemented;
    };

    if items.len() < 2 {
        return RuleApplication::NotImplemented;
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
