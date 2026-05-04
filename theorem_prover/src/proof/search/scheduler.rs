//! Scheduling policy for choosing which matched rule to try next.
//!
//! Reusable quantifiers prefer currently visible branch terms, then a bounded fresh fallback.
//! Exhausting that bounded exploration leaves the branch open rather than refuted.

use crate::Sequent;
use crate::ast::Term;
use crate::proof::quantifier::{fresh_branch_term_name, visible_terms_in_sequent};
use crate::proof::rules::{Rule, RuleMatch, Side, find_applicable_rules};
use crate::proof::search::branch_state::BranchState;

/// Outcome of trying to schedule the next proof-search step.
pub(crate) enum ScheduleResult {
    Rules(Vec<ScheduledRule>),
    QuantifierExhausted,
    NoRules,
}

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
    max_fresh_terms_per_quantifier: usize,
) -> ScheduleResult {
    let rule_matches = find_applicable_rules(sequent);

    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if matches!(rule_match.rule, Rule::Id | Rule::TopR | Rule::BottomL) {
            scheduled.push(ScheduledRule::Standard(*rule_match));
        }
    }
    if !scheduled.is_empty() {
        return ScheduleResult::Rules(scheduled);
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
        return ScheduleResult::Rules(scheduled);
    }

    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if matches!(rule_match.rule, Rule::AndR | Rule::OrL | Rule::ImpliesL) {
            scheduled.push(ScheduledRule::Standard(*rule_match));
        }
    }
    if !scheduled.is_empty() {
        return ScheduleResult::Rules(scheduled);
    }

    let mut scheduled = Vec::new();
    let mut saw_reusable_quantifier = false;
    for rule_match in &rule_matches {
        if matches!(rule_match.rule, Rule::ForAllL | Rule::ExistsR) {
            saw_reusable_quantifier = true;
        }
        for next_rule in schedule_quantifier_instantiations(
            sequent,
            state,
            rule_match,
            max_fresh_terms_per_quantifier,
        ) {
            scheduled.push(next_rule);
        }
    }
    if !scheduled.is_empty() {
        return ScheduleResult::Rules(scheduled);
    }

    if saw_reusable_quantifier {
        ScheduleResult::QuantifierExhausted
    } else {
        ScheduleResult::NoRules
    }
}

/// Schedules current branch-term instantiations, then bounded fresh fallback.
fn schedule_quantifier_instantiations(
    sequent: &Sequent,
    state: &BranchState,
    rule_match: &RuleMatch,
    max_fresh_terms_per_quantifier: usize,
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
    for term in visible_terms_in_sequent(sequent) {
        if usage.used_terms.contains(&term) {
            continue;
        }

        scheduled.push(match rule_match.rule {
            Rule::ForAllL => ScheduledRule::ForAllL {
                rule_match: *rule_match,
                term,
                key: key.clone(),
                fresh_fallback: false,
            },
            Rule::ExistsR => ScheduledRule::ExistsR {
                rule_match: *rule_match,
                term,
                key: key.clone(),
                fresh_fallback: false,
            },
            _ => return Vec::new(),
        });
    }

    if !scheduled.is_empty() {
        return scheduled;
    }

    if usage.fresh_terms_used >= max_fresh_terms_per_quantifier {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Formula, Symbol, Var};
    use crate::proof::search::branch_state::{BranchState, record_quantifier_term};

    fn var(name: &str) -> Var {
        Var {
            name: name.to_owned(),
        }
    }

    fn variable(name: &str) -> Term {
        Term::Var(var(name))
    }

    fn constant(name: &str) -> Term {
        Term::Const(Symbol::User(name.to_owned()))
    }

    fn function(name: &str, args: Vec<Term>) -> Term {
        Term::Fun {
            name: Symbol::User(name.to_owned()),
            args,
        }
    }

    fn predicate(name: &str, args: Vec<Term>) -> Formula {
        Formula::predicate(name, args)
    }

    #[test]
    fn schedules_newly_visible_branch_terms_for_reusable_quantifiers() {
        let quantified = Formula::ForAll(
            vec![var("X")],
            Box::new(predicate("p", vec![variable("X")])),
        );
        let term_a = constant("a");
        let term_f_a = function("f", vec![term_a.clone()]);
        let sequent = Sequent {
            left: vec![
                quantified,
                predicate("p", vec![term_a.clone()]),
                predicate("p", vec![term_f_a.clone()]),
            ],
            right: vec![Formula::atom("goal")],
        };
        let mut state = BranchState::new();
        let key = quantified_occurrence_key(&sequent, Side::Left, 0)
            .expect("quantified formula should have a key");
        record_quantifier_term(&mut state, &key, &term_a, false);

        let result = schedule_next_rules(&sequent, &state, 1);

        let ScheduleResult::Rules(rules) = result else {
            panic!("expected a scheduled branch-term instantiation");
        };
        assert!(rules.iter().any(|rule| matches!(
            rule,
            ScheduledRule::ForAllL { term, .. } if term == &term_f_a
        )));
    }

    #[test]
    fn reports_quantifier_exhausted_when_terms_and_fresh_budget_are_spent() {
        let quantified = Formula::Exists(
            vec![var("X")],
            Box::new(predicate("p", vec![variable("X")])),
        );
        let term_w = constant("w");
        let sequent = Sequent {
            left: Vec::new(),
            right: vec![quantified, predicate("p", vec![term_w.clone()])],
        };
        let mut state = BranchState::new();
        let key = quantified_occurrence_key(&sequent, Side::Right, 0)
            .expect("quantified formula should have a key");
        record_quantifier_term(&mut state, &key, &term_w, true);

        let result = schedule_next_rules(&sequent, &state, 1);

        assert!(matches!(result, ScheduleResult::QuantifierExhausted));
    }
}
