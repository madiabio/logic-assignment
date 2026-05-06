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

/// A scheduled proof rule ready for application during backward search.
///
/// This enum represents rule applications after scheduling has resolved any
/// necessary choices, particularly for quantifier rules that require selecting
/// an instantiation term.
///
/// Variants are split into:
///
/// - `Standard`: Rules that can be applied directly from the sequent structure
///   with no additional choices or state tracking.
/// - `ForAllL` / `ExistsR`: Quantifier rules that require an explicit term
///   instantiation and branch-local bookkeeping.
///
/// For quantifier variants:
///
/// - `rule_match`: The underlying structural match for the rule.
/// - `term`: The term chosen to instantiate the quantified variable.
/// - `key`: A unique identifier for the quantified occurrence, used to track
///   which terms have already been used on this branch.
/// - `fresh_fallback`: Indicates whether `term` is a fresh fallback (as opposed
///   to a reused existing term).
///
/// This separation allows the scheduler to enforce the quantifier instantiation
/// policy (reuse existing terms first, then optionally introduce fresh terms)
/// while keeping rule application logic simple and uniform downstream.
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

/// Selects the next batch of proof rules to try for a sequent.
///
/// Rules are scheduled in priority order:
///
/// 1. Closing rules: `Id`, `TopR`, `BottomL`.
/// 2. Invertible or single-premise rules, including `AndL`, `OrR`,
///    `ImpliesR`, negation rules, `ForAllR`, and `ExistsL`.
/// 3. Branching propositional rules: `AndR`, `OrL`, `ImpliesL`.
/// 4. Reusable quantifier rules: `ForAllL` and `ExistsR`, with candidate
///    instantiation terms chosen from the current [`BranchState`].
///
/// This function returns only the first non-empty priority group. That keeps
/// search focused and prevents lower-priority rules from being tried while a
/// higher-priority rule is still available.
///
/// If a reusable quantifier rule is applicable but no further instantiations
/// are allowed by the branch state or fresh-term budget, this returns
/// [`ScheduleResult::QuantifierExhausted`] instead of [`ScheduleResult::NoRules`].
///
/// # Parameters
///
/// - `sequent`: The current sequent being searched.
/// - `state`: Branch-local state used to avoid repeating quantifier
///   instantiations.
/// - `max_fresh_terms_per_quantifier`: Maximum number of fresh fallback terms
///   allowed for each reusable quantifier occurrence.
///
/// # Returns
///
/// A [`ScheduleResult`] containing the next rules to try, or indicating that no
/// rules remain or that quantifier exploration was exhausted.
pub(crate) fn schedule_next_rules(
    sequent: &Sequent,
    state: &BranchState,
    max_fresh_terms_per_quantifier: usize,
) -> ScheduleResult {
    // First collect every rule that is structurally applicable to this sequent.
    let rule_matches = find_applicable_rules(sequent);

    // Priority 1: closing rules.
    // These can immediately close a branch, so they should always be tried first.
    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if matches!(rule_match.rule, Rule::Id | Rule::TopR | Rule::BottomL) {
            scheduled.push(ScheduledRule::Standard(*rule_match));
        }
    }
    if !scheduled.is_empty() {
        return ScheduleResult::Rules(scheduled);
    }

    // Priority 2: non-branching / invertible rules.
    // These usually simplify the sequent without introducing search branching.
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

    // Priority 3: branching propositional rules.
    // These create multiple premises, so they are delayed until simpler rules
    // have been exhausted.
    let mut scheduled = Vec::new();
    for rule_match in &rule_matches {
        if matches!(rule_match.rule, Rule::AndR | Rule::OrL | Rule::ImpliesL) {
            scheduled.push(ScheduledRule::Standard(*rule_match));
        }
    }
    if !scheduled.is_empty() {
        return ScheduleResult::Rules(scheduled);
    }

    // Priority 4: reusable quantifier rules.
    // `ForAllL` and `ExistsR` require choosing an instantiation term, so they
    // go through special scheduling logic instead of being applied directly.
    let mut scheduled = Vec::new();
    let mut saw_reusable_quantifier = false;
    for rule_match in &rule_matches {
        if matches!(rule_match.rule, Rule::ForAllL | Rule::ExistsR) {
            saw_reusable_quantifier = true;
        }

        // Try unused visible terms first, then bounded fresh fallback terms.
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

    // If a reusable quantifier was present but produced no instantiations, then
    // search stopped because the quantifier budget/history was exhausted, not
    // because the branch is genuinely saturated.
    if saw_reusable_quantifier {
        ScheduleResult::QuantifierExhausted
    } else {
        ScheduleResult::NoRules
    }
}

/// Schedules instantiations for a reusable quantifier rule.
///
/// This handles the special search policy for `ForAllL` and `ExistsR`.
/// Unlike most rules, these require choosing a term to instantiate the
/// quantified variable with.
///
/// Scheduling proceeds in two stages:
///
/// 1. Try each currently visible branch term that has not already been used for
///    this quantified occurrence.
/// 2. If no visible terms remain, schedule one fresh fallback term if the
///    per-quantifier fresh-term budget allows it.
///
/// The quantified occurrence is identified using [`quantified_occurrence_key`],
/// and prior usage is read from [`BranchState::quantifier_usage`]. This prevents
/// repeatedly trying the same instantiation on the same branch.
///
/// # Parameters
///
/// - `sequent`: The current sequent being searched.
/// - `state`: Branch-local state tracking previous quantifier instantiations.
/// - `rule_match`: The matched quantifier rule occurrence to schedule.
/// - `max_fresh_terms_per_quantifier`: Maximum number of fresh fallback terms
///   allowed for this quantified occurrence.
///
/// # Returns
///
/// A list of scheduled `ForAllL` or `ExistsR` applications. Returns an empty
/// list if `rule_match` is not a reusable quantifier rule, the occurrence cannot
/// be keyed, or no further instantiations are allowed.
fn schedule_quantifier_instantiations(
    sequent: &Sequent,
    state: &BranchState,
    rule_match: &RuleMatch,
    max_fresh_terms_per_quantifier: usize,
) -> Vec<ScheduledRule> {
    // Build a stable key identifying *this specific quantified occurrence*.
    // If we can't (e.g. not actually a quantifier), nothing to schedule.
    let Some(key) = quantified_occurrence_key(sequent, rule_match.side, rule_match.index) else {
        return Vec::new();
    };

    // Look up how this occurrence has been used on the current branch:
    // - which terms have already been tried
    // - how many fresh terms have been introduced
    let usage = state
        .quantifier_usage
        .get(&key)
        .cloned()
        .unwrap_or_default();

    let mut scheduled = Vec::new();

    // --- Phase 1: reuse existing (visible) terms in the sequent ---
    // Try every term currently visible in the branch, but only once per occurrence.
    for term in visible_terms_in_sequent(sequent) {
        // Skip terms we've already used for this occurrence
        if usage.used_terms.contains(&term) {
            continue;
        }

        // Schedule a quantifier rule with this existing term
        scheduled.push(match rule_match.rule {
            Rule::ForAllL => ScheduledRule::ForAllL {
                rule_match: *rule_match,
                term,
                key: key.clone(),
                fresh_fallback: false, // this is a reused term
            },
            Rule::ExistsR => ScheduledRule::ExistsR {
                rule_match: *rule_match,
                term,
                key: key.clone(),
                fresh_fallback: false, // this is a reused term
            },
            // Not a reusable quantifier rule → nothing to do here
            _ => return Vec::new(),
        });
    }

    // If we found at least one unused existing term, return those.
    // We do NOT consider fresh terms yet (priority: reuse first).
    if !scheduled.is_empty() {
        return scheduled;
    }

    // --- Phase 2: fresh fallback ---
    // No existing terms left → optionally introduce a fresh term.
    // But only if we haven't exceeded the per-quantifier budget.
    if usage.fresh_terms_used >= max_fresh_terms_per_quantifier {
        // Budget exhausted → no more instantiations possible for this occurrence
        return Vec::new();
    }

    // Generate a new constant symbol (like w) and wraps it as a term so it can be used as a fresh witness in quantifier instantiation.
    let term = Term::Const(crate::ast::Symbol::User(fresh_branch_term_name(sequent)));

    // Schedule exactly one fresh instantiation
    scheduled.push(match rule_match.rule {
        Rule::ForAllL => ScheduledRule::ForAllL {
            rule_match: *rule_match,
            term,
            key,
            fresh_fallback: true, // indicates this is a fresh-term fallback
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

/// Constructs a stable identifier for a quantified formula occurrence in a sequent.
///
/// This key uniquely identifies a *specific occurrence* of a quantified formula
/// (`∀` or `∃`) within the sequent, rather than just the formula shape. It is
/// used to track branch-local quantifier instantiation history (e.g. which terms
/// have already been used for this occurrence).
///
/// The key is composed of:
///
/// - The side of the sequent (`Left` or `Right`)
/// - The ordinal occurrence of this formula among syntactically identical formulas
///   up to and including `index`
/// - The string representation of the formula
///
/// This ensures that multiple identical quantified formulas in the same sequent
/// are distinguished by position, while remaining stable across recursive search
/// steps where the sequent structure is preserved.
///
/// # Parameters
///
/// - `sequent`: The current sequent containing the formula.
/// - `side`: Which side of the sequent (`Γ` or `Δ`) to inspect.
/// - `index`: The index of the formula within that side.
///
/// # Returns
///
/// - `Some(String)` containing a unique key if the formula at the given position
///   is quantified (`∀` or `∃`).
/// - `None` if the index is out of bounds or the formula is not quantified.
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
