use crate::proof::rules::Rule;
use crate::proof::rules::apply::RuleApplication;

pub(crate) fn apply_structural(rule: Rule) -> RuleApplication {
    match rule {
        Rule::Id | Rule::TopR | Rule::BottomL => RuleApplication::Closed,
        _ => RuleApplication::NotImplemented,
    }
}
