// Backwards Search Proof API
use std::time::{Duration, Instant};

use log::warn;

use crate::Sequent;
use crate::proof::apply::{RuleApplication, apply_rule};
use crate::proof::rules::find_applicable_rules;

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
        status: backwards_search(sequent, deadline).into_status(),
    }
}

// Backward search tries every rule that applies to the current sequent. A rule
// either closes the branch immediately or reduces the goal into premise
// sequents, which are then proved recursively.
fn backwards_search(sequent: &Sequent, deadline: Instant) -> SearchOutcome {
    if Instant::now() >= deadline {
        warn!("Proof search timed out.");
        return SearchOutcome::Timeout;
    }

    let mut best = SearchOutcome::NotProvable;

    for rule_match in find_applicable_rules(sequent) {
        let outcome = match apply_rule(sequent, &rule_match) {
            RuleApplication::Closed => SearchOutcome::Provable,
            RuleApplication::NotImplemented => {
                warn!("Not implemented rule: {:?}", rule_match);
                SearchOutcome::NotImplemented
            }
            // Recursive step: applying a rule can produce one or more premise
            // sequents, and each premise is proved by calling backwards_search
            // again via prove_premises below.
            RuleApplication::Premises(premises) => prove_premises(&premises, deadline),
            RuleApplication::Error => {
                warn!("Error.");
                SearchOutcome::Error
            }
        };

        match outcome {
            SearchOutcome::Provable | SearchOutcome::Timeout => return outcome,
            other => best = best.merge(other),
        }
    }

    best
}

// A branching rule succeeds only if every premise succeeds. The first premise
// that fails determines the whole rule outcome, so we can stop exploring that
// branch as soon as one premise is not provable.
fn prove_premises(premises: &[Sequent], deadline: Instant) -> SearchOutcome {
    for premise in premises {
        // This is the actual recursive call in the backward-search loop:
        // each premise generated from the current sequent becomes a fresh
        // subproblem that is searched in exactly the same way.
        let outcome = backwards_search(premise, deadline);

        if outcome != SearchOutcome::Provable {
            return outcome;
        }
    }

    SearchOutcome::Provable
}
