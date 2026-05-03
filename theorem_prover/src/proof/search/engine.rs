use std::time::{Duration, Instant};

use log::warn;

use crate::Sequent;
use crate::proof::apply::{
    RuleApplication, apply_exists_r_with_term, apply_forall_l_with_term, apply_rule,
};
use crate::proof::search::branch_state::{BranchState, record_quantifier_term};
use crate::proof::search::scheduler::{ScheduledRule, schedule_next_rules};

const DEFAULT_PROVE_TIMEOUT: Duration = Duration::from_secs(1);

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

pub fn prove(sequent: &Sequent, options: ProofOptions) -> ProofResult {
    let deadline = Instant::now() + options.timeout;

    ProofResult {
        status: backwards_search(sequent, deadline, &BranchState::default()).into_status(),
    }
}

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

fn prove_premises(premises: &[Sequent], deadline: Instant, state: &BranchState) -> SearchOutcome {
    for premise in premises {
        let outcome = backwards_search(premise, deadline, state);

        if outcome != SearchOutcome::Provable {
            return outcome;
        }
    }

    SearchOutcome::Provable
}
