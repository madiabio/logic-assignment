// Backwards Search Proof API
use crate::Sequent;
use crate::proof::apply::{RuleApplication, apply_rule};
use crate::proof::rules::find_applicable_rules;

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
    let status = if prove_inner(sequent) {
        ProofStatus::Provable
    } else {
        ProofStatus::NotProvable
    };

    return ProofResult { status };
}

// Do backwards search as backtracking algorithm.
fn prove_inner(sequent: &Sequent) -> bool {
    find_applicable_rules(sequent)
        .iter()
        .any(|rule_match| match apply_rule(sequent, rule_match) {
            RuleApplication::Closed => true,

            // recursively prove the premises
            RuleApplication::Premises(premises) => premises.iter().all(prove_inner),

            RuleApplication::NotImplemented => false,
        })
}
