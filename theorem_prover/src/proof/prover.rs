// Backwards Search Proof API
use crate::Sequent;
use crate::proof::apply::{RuleApplication, apply_rule};
use crate::proof::rules::find_applicable_rules;
use log::{debug, warn};

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

// public API
pub fn prove(sequent: &Sequent) -> ProofResult {
    let status = if backwards_search(sequent) {
        ProofStatus::Provable
    } else {
        ProofStatus::NotProvable
    };

    return ProofResult { status };
}

// Do backwards search as backtracking algorithm.
fn backwards_search(sequent: &Sequent) -> bool {
    find_applicable_rules(sequent)
        .iter()
        .any(|rule_match| match apply_rule(sequent, rule_match) {
            RuleApplication::Closed => true,

            // recursively prove the premises
            RuleApplication::Premises(premises) => premises.iter().all(backwards_search),

            RuleApplication::NotImplemented => {
                warn!("Not implemented rule: {:?}", rule_match);
                false
            }
        })
}
