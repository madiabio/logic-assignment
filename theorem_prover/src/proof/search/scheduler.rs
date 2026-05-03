//! Scheduling policy for choosing which matched rule to try next.

use crate::Sequent;
use crate::ast::Term;
use crate::proof::quantifier::{fresh_branch_term_name, visible_terms_in_sequent};
use crate::proof::rules::{Rule, RuleMatch, Side, find_applicable_rules};
use crate::proof::search::branch_state::BranchState;

/// A rule application that is ready to run, possibly with a chosen quantifier term.
pub(crate) enum ScheduledRule {
    Standard(RuleMatch),
    ForAllL {
        rule_match: RuleMatch,
        term: Term,
        key: String,
        fresh_fallback: bool,
    },
    ExistsR {
        rule_match: RuleMatch,
        term: Term,
        key: String,
        fresh_fallback: bool,
    },
}

/// Selects the next batch of rules to try for the current sequent and branch state.
pub(crate) fn schedule_next_rules(
    sequent: &Sequent,
    state: &BranchState,
) -> Option<Vec<ScheduledRule>> {
    let rule_matches = find_applicable_rules(sequent);

    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if matches!(rule_match.rule, Rule::Id | Rule::TopR | Rule::BottomL) {
            scheduled.push(ScheduledRule::Standard(*rule_match));
        }
    }
    if !scheduled.is_empty() {
        return Some(scheduled);
    }

    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if matches!(
            rule_match.rule,
            Rule::AndL
                | Rule::OrR
                | Rule::ImpliesR
                | Rule::NotL
                | Rule::NotR
                | Rule::ForAllR
                | Rule::ExistsL
        ) {
            scheduled.push(ScheduledRule::Standard(*rule_match));
        }
    }
    if !scheduled.is_empty() {
        return Some(scheduled);
    }

    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if matches!(rule_match.rule, Rule::AndR | Rule::OrL | Rule::ImpliesL) {
            scheduled.push(ScheduledRule::Standard(*rule_match));
        }
    }
    if !scheduled.is_empty() {
        return Some(scheduled);
    }

    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if let Some(next_rule) = schedule_quantifier_reuse(sequent, state, rule_match) {
            scheduled.push(next_rule);
        }
    }
    if !scheduled.is_empty() {
        return Some(scheduled);
    }

    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if let Some(next_rule) = schedule_quantifier_fresh_fallback(sequent, state, rule_match) {
            scheduled.push(next_rule);
        }
    }
    if !scheduled.is_empty() {
        return Some(scheduled);
    }

    None
}

/// Schedules quantifier rules that can reuse a visible term not tried on this branch yet.
fn schedule_quantifier_reuse(
    sequent: &Sequent,
    state: &BranchState,
    rule_match: &RuleMatch,
) -> Option<ScheduledRule> {
    let key = quantified_occurrence_key(sequent, rule_match.side, rule_match.index)?;
    let usage = state
        .quantifier_usage
        .get(&key)
        .cloned()
        .unwrap_or_default();

    let term = visible_terms_in_sequent(sequent)
        .into_iter()
        .find(|term| !usage.used_terms.contains(&term.to_string()))?;

    Some(match rule_match.rule {
        Rule::ForAllL => ScheduledRule::ForAllL {
            rule_match: *rule_match,
            term,
            key,
            fresh_fallback: false,
        },
        Rule::ExistsR => ScheduledRule::ExistsR {
            rule_match: *rule_match,
            term,
            key,
            fresh_fallback: false,
        },
        _ => return None,
    })
}

/// Schedules one fresh-term fallback for a quantified occurrence when reuse is exhausted.
fn schedule_quantifier_fresh_fallback(
    sequent: &Sequent,
    state: &BranchState,
    rule_match: &RuleMatch,
) -> Option<ScheduledRule> {
    let key = quantified_occurrence_key(sequent, rule_match.side, rule_match.index)?;
    let usage = state
        .quantifier_usage
        .get(&key)
        .cloned()
        .unwrap_or_default();
    if usage.fresh_fallback_used {
        return None;
    }

    let term = Term::Const(crate::ast::Symbol::User(fresh_branch_term_name(sequent)));

    Some(match rule_match.rule {
        Rule::ForAllL => ScheduledRule::ForAllL {
            rule_match: *rule_match,
            term,
            key,
            fresh_fallback: true,
        },
        Rule::ExistsR => ScheduledRule::ExistsR {
            rule_match: *rule_match,
            term,
            key,
            fresh_fallback: true,
        },
        _ => return None,
    })
}

/// Builds a stable key for one quantified formula occurrence within a sequent.
fn quantified_occurrence_key(sequent: &Sequent, side: Side, index: usize) -> Option<String> {
    let formulas = match side {
        Side::Left => &sequent.left,
        Side::Right => &sequent.right,
    };

    let formula = formulas.get(index)?;
    if !matches!(
        formula,
        crate::ast::Formula::ForAll(_, _) | crate::ast::Formula::Exists(_, _)
    ) {
        return None;
    }

    let formula_text = formula.to_string();
    let ordinal = formulas[..=index]
        .iter()
        .filter(|candidate| candidate.to_string() == formula_text)
        .count();

    Some(format!("{side:?}:{ordinal}:{formula_text}"))
}
