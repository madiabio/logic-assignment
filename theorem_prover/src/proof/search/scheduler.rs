//! Scheduling policy for choosing which matched rule to try next.
//!
//! Use frozen baseline visible terms first, then allow exactly one fresh fallback per quantified
//! occurrence, and never promote generated terms into the candidate pool.

use crate::Sequent;
use crate::ast::Term;
use crate::proof::quantifier::fresh_branch_term_name;
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
        for next_rule in schedule_quantifier_instantiations(sequent, state, rule_match) {
            scheduled.push(next_rule);
        }
    }
    if !scheduled.is_empty() {
        return Some(scheduled);
    }

    None
}

/// Schedules frozen-baseline instantiations, then one fresh fallback per quantified occurrence.
fn schedule_quantifier_instantiations(
    sequent: &Sequent,
    state: &BranchState,
    rule_match: &RuleMatch,
) -> Vec<ScheduledRule> {
    let Some(key) = quantified_occurrence_key(sequent, rule_match.side, rule_match.index) else {
        return Vec::new();
    };
    let usage = state
        .quantifier_usage
        .get(&key)
        .cloned()
        .unwrap_or_default();

    let mut scheduled = Vec::new();
    for term in state.baseline_terms.iter() {
        if usage.used_baseline_terms.contains(term) {
            continue;
        }

        scheduled.push(match rule_match.rule {
            Rule::ForAllL => ScheduledRule::ForAllL {
                rule_match: *rule_match,
                term: term.clone(),
                key: key.clone(),
                fresh_fallback: false,
            },
            Rule::ExistsR => ScheduledRule::ExistsR {
                rule_match: *rule_match,
                term: term.clone(),
                key: key.clone(),
                fresh_fallback: false,
            },
            _ => return Vec::new(),
        });
    }

    if !scheduled.is_empty() {
        return scheduled;
    }

    if usage.fresh_used {
        return Vec::new();
    }

    let term = Term::Const(crate::ast::Symbol::User(fresh_branch_term_name(sequent)));
    scheduled.push(match rule_match.rule {
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
        _ => return Vec::new(),
    });

    scheduled
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
