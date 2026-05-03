// Backwards Search Proof API
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use log::warn;

use crate::Sequent;
use crate::ast::Term;
use crate::proof::apply::{
    RuleApplication, apply_exists_r_with_term, apply_forall_l_with_term, apply_rule,
};
use crate::proof::quantifier::{fresh_branch_term_name, visible_terms_in_sequent};
use crate::proof::rules::{Rule, RuleMatch, Side, find_applicable_rules};

const DEFAULT_PROVE_TIMEOUT: Duration = Duration::from_secs(1);

// Public proof-search configuration. This is intentionally small for now, but
// it gives the API a stable place to grow future search controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProofOptions {
    pub timeout: Duration,
}

impl Default for ProofOptions {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_PROVE_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofStatus {
    NotImplemented,
    Provable,
    NotProvable,
    Timeout,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofResult {
    pub status: ProofStatus,
}

// Internal branch result used while searching. We keep this separate from the
// public status so recursive search can combine partial outcomes before
// exposing one final ProofStatus at the API boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchOutcome {
    Provable,
    NotProvable,
    Timeout,
    NotImplemented,
    Error,
}

impl SearchOutcome {
    fn into_status(self) -> ProofStatus {
        match self {
            SearchOutcome::Provable => ProofStatus::Provable,
            SearchOutcome::Timeout => ProofStatus::Timeout,
            SearchOutcome::NotImplemented => ProofStatus::NotImplemented,
            SearchOutcome::Error => ProofStatus::Error,
            SearchOutcome::NotProvable => ProofStatus::NotProvable,
        }
    }

    fn merge(self, other: Self) -> Self {
        if self.priority() >= other.priority() {
            self
        } else {
            other
        }
    }

    fn priority(self) -> u8 {
        match self {
            SearchOutcome::Provable => 4,
            SearchOutcome::Timeout => 3,
            SearchOutcome::NotImplemented => 2,
            SearchOutcome::Error => 1,
            SearchOutcome::NotProvable => 0,
        }
    }
}

// public API
pub fn prove(sequent: &Sequent, options: ProofOptions) -> ProofResult {
    let deadline = Instant::now() + options.timeout;

    ProofResult {
        status: backwards_search(sequent, deadline, &BranchState::default()).into_status(),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BranchState {
    quantifier_usage: BTreeMap<String, QuantifierUsage>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct QuantifierUsage {
    used_terms: BTreeSet<String>,
    fresh_fallback_used: bool,
}

enum ScheduledRule {
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

// Backward search follows Algorithm 2's deterministic priority order. A step
// either closes the branch immediately or reduces the goal into premise
// sequents, which are then proved recursively.
fn backwards_search(sequent: &Sequent, deadline: Instant, state: &BranchState) -> SearchOutcome {
    if Instant::now() >= deadline {
        warn!("Proof search timed out.");
        return SearchOutcome::Timeout;
    }

    let Some(scheduled_rules) = schedule_next_rules(sequent, state) else {
        return SearchOutcome::NotProvable;
    };

    let mut best = SearchOutcome::NotProvable;

    for scheduled_rule in scheduled_rules {
        let mut next_state = state.clone();
        let application = match &scheduled_rule {
            ScheduledRule::Standard(rule_match) => apply_rule(sequent, rule_match),
            ScheduledRule::ForAllL {
                rule_match,
                term,
                key,
                fresh_fallback,
            } => {
                record_quantifier_term(&mut next_state, key, term, *fresh_fallback);
                apply_forall_l_with_term(sequent, rule_match.index, term)
            }
            ScheduledRule::ExistsR {
                rule_match,
                term,
                key,
                fresh_fallback,
            } => {
                record_quantifier_term(&mut next_state, key, term, *fresh_fallback);
                apply_exists_r_with_term(sequent, rule_match.index, term)
            }
        };

        let outcome = match application {
            RuleApplication::Closed => SearchOutcome::Provable,
            RuleApplication::NotImplemented => {
                warn!("Not implemented rule.");
                SearchOutcome::NotImplemented
            }
            RuleApplication::Premises(premises) => prove_premises(&premises, deadline, &next_state),
            RuleApplication::Error => {
                warn!("Error.");
                SearchOutcome::Error
            }
        };

        match outcome {
            SearchOutcome::Provable | SearchOutcome::Timeout => return outcome,
            other => {
                best = best.merge(other);
            }
        }
    }

    best
}

// A branching rule succeeds only if every premise succeeds. The first premise
// that fails determines the whole rule outcome, so we can stop exploring that
// branch as soon as one premise is not provable.
fn prove_premises(premises: &[Sequent], deadline: Instant, state: &BranchState) -> SearchOutcome {
    for premise in premises {
        let outcome = backwards_search(premise, deadline, state);

        if outcome != SearchOutcome::Provable {
            return outcome;
        }
    }

    SearchOutcome::Provable
}

fn schedule_next_rules(sequent: &Sequent, state: &BranchState) -> Option<Vec<ScheduledRule>> {
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

fn schedule_quantifier_reuse(
    sequent: &Sequent,
    state: &BranchState,
    rule_match: &RuleMatch,
) -> Option<ScheduledRule> {
    let key = quantified_occurrence_key(sequent, rule_match.side, rule_match.index)?;
    let usage = state.quantifier_usage.get(&key).cloned().unwrap_or_default();

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

fn schedule_quantifier_fresh_fallback(
    sequent: &Sequent,
    state: &BranchState,
    rule_match: &RuleMatch,
) -> Option<ScheduledRule> {
    let key = quantified_occurrence_key(sequent, rule_match.side, rule_match.index)?;
    let usage = state.quantifier_usage.get(&key).cloned().unwrap_or_default();
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

fn quantified_occurrence_key(sequent: &Sequent, side: Side, index: usize) -> Option<String> {
    let formulas = match side {
        Side::Left => &sequent.left,
        Side::Right => &sequent.right,
    };

    let formula = formulas.get(index)?;
    if !matches!(formula, crate::ast::Formula::ForAll(_, _) | crate::ast::Formula::Exists(_, _)) {
        return None;
    }

    let formula_text = formula.to_string();
    let ordinal = formulas[..=index]
        .iter()
        .filter(|candidate| candidate.to_string() == formula_text)
        .count();

    Some(format!("{side:?}:{ordinal}:{formula_text}"))
}

fn record_quantifier_term(state: &mut BranchState, key: &str, term: &Term, fresh_fallback: bool) {
    let usage = state.quantifier_usage.entry(key.to_owned()).or_default();
    usage.used_terms.insert(term.to_string());
    if fresh_fallback {
        usage.fresh_fallback_used = true;
    }
}
