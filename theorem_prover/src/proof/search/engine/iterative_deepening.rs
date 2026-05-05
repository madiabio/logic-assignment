use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::Sequent;
use crate::proof::search::branch_state::BranchState;
use super::{ProofOptions, SearchOutcome, UnknownReason, backwards_search};

pub(super) fn run(
    sequent: &Sequent,
    deadline: Instant,
    options: &ProofOptions,
    cancel_requested: &AtomicBool,
) -> SearchOutcome {
    // steps_taken carries across iterations: max_steps is a total-work budget,
    // consistent with the naive engine's interpretation of the flag.
    let mut steps_taken = 0usize;

    for depth_limit in 1..=options.max_depth {
        if cancel_requested.load(Ordering::Relaxed) {
            return SearchOutcome::Cancelled;
        }
        if Instant::now() >= deadline {
            return SearchOutcome::Timeout;
        }
        let mut opts = *options;
        opts.max_depth = depth_limit;
        let state = BranchState::new();

        let outcome = backwards_search(
            sequent, deadline, &state, &opts, cancel_requested, 0, &mut steps_taken,
        );

        match outcome {
            SearchOutcome::Unknown(UnknownReason::MaxDepthExceeded) => continue,
            other => return other,
        }
    }

    SearchOutcome::Unknown(UnknownReason::MaxDepthExceeded)
}
