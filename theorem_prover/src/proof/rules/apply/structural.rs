//! Application of structural rules that close a branch immediately.

use crate::proof::rules::Rule;
use crate::proof::rules::apply::RuleApplication;

/// Applies structural rules such as identity, `⊤R`, and `⊥L`.
pub(crate) fn apply_structural(rule: Rule) -> RuleApplication {
    match rule {
        Rule::Id | Rule::TopR | Rule::BottomL => RuleApplication::Closed,
        _ => RuleApplication::NotImplemented,
    }
}
