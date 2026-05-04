//! Depth-first backward proof search with timeout and bounded search handling.
//!
//! Default prover limits are defined by:
//! - [`crate::proof::defaults::DEFAULT_PROVE_TIMEOUT`]
//! - [`crate::proof::defaults::DEFAULT_MAX_DEPTH`]
//! - [`crate::proof::defaults::DEFAULT_MAX_STEPS`]
//! - [`crate::proof::defaults::DEFAULT_MAX_FRESH_TERMS_PER_QUANTIFIER`]
//!
//! CLI usage:
//! - `cargo run -- prove problem.p`
//! - `cargo run -- prove --timeout-ms 1000 problem.p`
//! - `cargo run -- prove --max-depth 64 problem.p`
//! - `cargo run -- prove --max-steps 10000 problem.p`
//! - `cargo run -- prove --timeout-ms 1000 --max-depth 64 --max-steps 10000 problem.p`
//!
//! Rule inspection uses the separate `rules` subcommand:
//! - `cargo run -- rules problem.p`
//! - `cargo run -- rules --show-sequent problem.p`

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use log::warn;

use crate::Sequent;
use crate::proof::apply::{
    RuleApplication, apply_exists_r_with_term, apply_forall_l_with_term, apply_rule,
};
use crate::proof::defaults::{
    DEFAULT_MAX_DEPTH, DEFAULT_MAX_FRESH_TERMS_PER_QUANTIFIER, DEFAULT_MAX_STEPS,
    DEFAULT_PROVE_TIMEOUT,
};
use crate::proof::search::branch_state::{BranchState, record_quantifier_term};
use crate::proof::search::scheduler::{ScheduleResult, ScheduledRule, schedule_next_rules};

/// Explains why a proof attempt ended with [`ProofStatus::Unknown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownReason {
    /// Input processing was skipped because the configured biconditional cap was exceeded.
    BiconditionalCapExceeded,
    /// Input contains a TPTP include directive, which is not loaded yet.
    UnsupportedInclude,
    /// Search reached the configured recursive branch depth limit.
    MaxDepthExceeded,
    /// Search reached the configured proof-step limit.
    MaxStepsExceeded,
    /// Search exhausted the fresh fallback terms available for one quantified occurrence.
    QuantifierBudgetExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Runtime options controlling proof search.
pub struct ProofOptions {
    /// Maximum wall-clock time allowed for a single proof attempt.
    ///
    /// The default comes from
    /// [`crate::proof::defaults::DEFAULT_PROVE_TIMEOUT`].
    pub timeout: Duration,
    /// Maximum recursive branch depth before search returns `Unknown`.
    ///
    /// The default comes from [`crate::proof::defaults::DEFAULT_MAX_DEPTH`].
    pub max_depth: usize,
    /// Maximum search steps before search returns `Unknown`.
    ///
    /// The default comes from [`crate::proof::defaults::DEFAULT_MAX_STEPS`].
    pub max_steps: usize,
    /// Maximum fresh fallback terms for one reusable quantified occurrence.
    ///
    /// Exhausting this budget leaves the branch open and returns `Unknown`.
    /// The default comes from
    /// [`crate::proof::defaults::DEFAULT_MAX_FRESH_TERMS_PER_QUANTIFIER`].
    pub max_fresh_terms_per_quantifier: usize,
}

impl Default for ProofOptions {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_PROVE_TIMEOUT,
            max_depth: DEFAULT_MAX_DEPTH,
            max_steps: DEFAULT_MAX_STEPS,
            max_fresh_terms_per_quantifier: DEFAULT_MAX_FRESH_TERMS_PER_QUANTIFIER,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Final status of a proof attempt.
pub enum ProofStatus {
    /// Search reached a recognised but unimplemented rule.
    NotImplemented,
    /// The sequent was proved.
    Provable,
    /// Search exhausted all scheduled choices without proving the sequent.
    NotProvable,
    /// Search exceeded the configured timeout.
    Timeout,
    /// Search hit a configured depth or step bound before reaching a proof result.
    Unknown,
    /// Search was interrupted by a cancellation request such as `Ctrl+C`.
    Cancelled,
    /// Search encountered an unexpected rule-application failure.
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result returned by the public prover API.
pub struct ProofResult {
    pub status: ProofStatus,
    /// More specific detail for [`ProofStatus::Unknown`].
    pub unknown_reason: Option<UnknownReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Internal search outcome used while combining branch results.
enum SearchOutcome {
    Provable,
    NotProvable,
    Timeout,
    Unknown(UnknownReason),
    Cancelled,
    NotImplemented,
    Error,
}

impl SearchOutcome {
    /// Converts an internal search outcome into the public proof result.
    fn into_result(self) -> ProofResult {
        match self {
            SearchOutcome::Provable => ProofResult {
                status: ProofStatus::Provable,
                unknown_reason: None,
            },
            SearchOutcome::Timeout => ProofResult {
                status: ProofStatus::Timeout,
                unknown_reason: None,
            },
            SearchOutcome::Unknown(reason) => ProofResult {
                status: ProofStatus::Unknown,
                unknown_reason: Some(reason),
            },
            SearchOutcome::Cancelled => ProofResult {
                status: ProofStatus::Cancelled,
                unknown_reason: None,
            },
            SearchOutcome::NotImplemented => ProofResult {
                status: ProofStatus::NotImplemented,
                unknown_reason: None,
            },
            SearchOutcome::Error => ProofResult {
                status: ProofStatus::Error,
                unknown_reason: None,
            },
            SearchOutcome::NotProvable => ProofResult {
                status: ProofStatus::NotProvable,
                unknown_reason: None,
            },
        }
    }

    /// Keeps the more informative of two non-successful search outcomes.
    fn merge(self, other: Self) -> Self {
        if self.priority() >= other.priority() {
            self
        } else {
            other
        }
    }

    /// Returns the precedence used when combining competing outcomes.
    fn priority(self) -> u8 {
        match self {
            SearchOutcome::Provable => 5,
            SearchOutcome::Timeout => 4,
            SearchOutcome::Cancelled => 3,
            SearchOutcome::Unknown(_) => 2,
            SearchOutcome::NotImplemented => 1,
            SearchOutcome::Error => 1,
            SearchOutcome::NotProvable => 0,
        }
    }
}

/// Attempts to prove a sequent within the configured timeout and search bounds.
pub fn prove(sequent: &Sequent, options: ProofOptions) -> ProofResult {
    static NEVER_CANCELLED: AtomicBool = AtomicBool::new(false);
    prove_with_cancel(sequent, options, &NEVER_CANCELLED)
}

/// Attempts to prove a sequent while observing an external cancellation flag.
pub fn prove_with_cancel(
    sequent: &Sequent,
    options: ProofOptions,
    cancel_requested: &AtomicBool,
) -> ProofResult {
    let deadline = Instant::now() + options.timeout;
    let state = BranchState::new();
    let mut steps_taken = 0usize;

    backwards_search(
        sequent,
        deadline,
        &state,
        &options,
        cancel_requested,
        0,
        &mut steps_taken,
    )
    .into_result()
}

/// Performs backward search from a single sequent until it closes or fails.
fn backwards_search(
    sequent: &Sequent,
    deadline: Instant,
    state: &BranchState,
    options: &ProofOptions,
    cancel_requested: &AtomicBool,
    depth: usize,
    steps_taken: &mut usize,
) -> SearchOutcome {
    if cancel_requested.load(Ordering::Relaxed) {
        warn!("Proof search cancelled.");
        return SearchOutcome::Cancelled;
    }

    if Instant::now() >= deadline {
        warn!("Proof search timed out.");
        return SearchOutcome::Timeout;
    }

    if depth > options.max_depth {
        warn!("Proof search hit the max depth limit.");
        return SearchOutcome::Unknown(UnknownReason::MaxDepthExceeded);
    }
    if *steps_taken >= options.max_steps {
        warn!("Proof search hit the max step limit.");
        return SearchOutcome::Unknown(UnknownReason::MaxStepsExceeded);
    }
    *steps_taken += 1;

    let scheduled_rules =
        match schedule_next_rules(sequent, state, options.max_fresh_terms_per_quantifier) {
            ScheduleResult::Rules(rules) => rules,
            ScheduleResult::QuantifierExhausted => {
                warn!("Proof search exhausted the fresh quantifier fallback budget.");
                return SearchOutcome::Unknown(UnknownReason::QuantifierBudgetExceeded);
            }
            ScheduleResult::NoRules => return SearchOutcome::NotProvable,
        };

    let mut best = SearchOutcome::NotProvable;

    for scheduled_rule in scheduled_rules {
        if cancel_requested.load(Ordering::Relaxed) {
            warn!("Proof search cancelled.");
            return SearchOutcome::Cancelled;
        }

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
            RuleApplication::Premises(premises) => prove_premises(
                &premises,
                deadline,
                &next_state,
                options,
                cancel_requested,
                depth + 1,
                steps_taken,
            ),
            RuleApplication::Error => {
                warn!("Error.");
                SearchOutcome::Error
            }
        };

        match outcome {
            SearchOutcome::Provable | SearchOutcome::Timeout | SearchOutcome::Cancelled => {
                return outcome;
            }
            other => {
                best = best.merge(other);
            }
        }
    }

    best
}

/// Proves all premises generated by a rule application on the current branch.
fn prove_premises(
    premises: &[Sequent],
    deadline: Instant,
    state: &BranchState,
    options: &ProofOptions,
    cancel_requested: &AtomicBool,
    depth: usize,
    steps_taken: &mut usize,
) -> SearchOutcome {
    for premise in premises {
        if cancel_requested.load(Ordering::Relaxed) {
            warn!("Proof search cancelled.");
            return SearchOutcome::Cancelled;
        }

        let outcome = backwards_search(
            premise,
            deadline,
            state,
            options,
            cancel_requested,
            depth,
            steps_taken,
        );

        if outcome != SearchOutcome::Provable {
            return outcome;
        }
    }

    SearchOutcome::Provable
}
