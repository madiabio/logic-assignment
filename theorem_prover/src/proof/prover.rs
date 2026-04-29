// Backwards Search Proof API
use crate::Sequent;
use crate::proof::apply::{RuleApplication, apply_rule};
use crate::proof::rules::find_applicable_rules;
use log::warn;

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
    let mut saw_not_implemented = false; // flag
    let status = if backwards_search(sequent, &mut saw_not_implemented) {
        ProofStatus::Provable
    } else if saw_not_implemented {
        ProofStatus::NotImplemented
    } else {
        ProofStatus::NotProvable
    };

    return ProofResult { status };
}

// Do backwards search as backtracking algorithm.
fn backwards_search(sequent: &Sequent, saw_not_implemented: &mut bool) -> bool {
    find_applicable_rules(sequent)
        .iter()
        .any(|rule_match| match apply_rule(sequent, rule_match) {
            RuleApplication::Closed => true,

            RuleApplication::NotImplemented => {
                *saw_not_implemented = true;
                warn!("Not implemented rule: {:?}", rule_match);
                false
            }

            // recursively prove the premises
            RuleApplication::Premises(premises) => premises
                .iter()
                .all(|premise| backwards_search(premise, saw_not_implemented)),
        })
}
