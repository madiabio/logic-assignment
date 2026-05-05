use std::sync::atomic::AtomicBool;
use std::time::Instant;

use crate::Sequent;
use crate::proof::search::branch_state::BranchState;
use super::{ProofOptions, SearchOutcome, backwards_search};

pub(super) fn run(
    sequent: &Sequent,
    deadline: Instant,
    options: &ProofOptions,
    cancel_requested: &AtomicBool,
) -> SearchOutcome {
    let state = BranchState::new();
    let mut steps_taken = 0usize;
    backwards_search(sequent, deadline, &state, options, cancel_requested, 0, &mut steps_taken)
}
