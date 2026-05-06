//! Iterative-deepening backward search strategy.
//!
//! Runs repeated depth-first searches with depth limits 1, 2, 3, … up to
//! [`ProofOptions::max_depth`].  Each iteration that exhausts the current
//! depth limit ([`UnknownReason::MaxDepthExceeded`]) increases the limit by one
//! and restarts from the root.  All other outcomes — proof found, sequent
//! unprovable, timeout, cancellation, step budget exhausted, or quantifier
//! budget exhausted — terminate the loop immediately.
//!
//! ## Step budget
//!
//! `steps_taken` accumulates across all iterations.  This keeps the
//! `--max-steps` flag consistent with the naive engine: it is a total-work
//! budget for the entire search call, not a per-depth-level budget.
//!
//! ## Branch state
//!
//! [`BranchState`] (quantifier instantiation history) is reset at the start of
//! each iteration so that each depth level is explored with a clean slate.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::Sequent;
use crate::proof::search::branch_state::BranchState;
use super::{ProofOptions, SearchOutcome, UnknownReason, backwards_search};

/// Runs iterative-deepening backward search from `sequent`.
///
/// The depth limit grows from `1` to `options.max_depth` inclusive.  On each
/// iteration a fresh [`BranchState`] is created and [`backwards_search`] is
/// called with the current depth limit.  The loop continues only when the
/// outcome is [`SearchOutcome::Unknown`]`(`[`UnknownReason::MaxDepthExceeded`]`)`;
/// any other outcome is returned immediately.
///
/// If the loop exhausts every depth level without finding a proof or a definitive
/// non-proof, [`SearchOutcome::Unknown`]`(`[`UnknownReason::MaxDepthExceeded`]`)`
/// is returned.
///
/// # Parameters
///
/// - `sequent`: The root sequent to prove.
/// - `deadline`: Wall-clock deadline after which search returns
///   [`SearchOutcome::Timeout`].
/// - `options`: Proof-search limits.  `options.max_depth` controls the maximum
///   depth limit tried; `options.max_steps` is a cumulative budget across all
///   iterations.
/// - `cancel_requested`: External cancellation flag checked before each
///   iteration.
///
/// # Returns
///
/// A [`SearchOutcome`] that [`super::prove_with_cancel`] converts to a
/// [`super::ProofResult`].
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
