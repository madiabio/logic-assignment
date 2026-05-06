//! Naive (single-pass depth-first) backward search strategy.
//!
//! Performs one depth-limited DFS from the root sequent.  Search stops when
//! the first proof is found, the sequent is shown unprovable within the
//! implemented fragment, or a configured resource limit is reached.

use std::sync::atomic::AtomicBool;
use std::time::Instant;

use crate::Sequent;
use crate::proof::search::branch_state::BranchState;
use super::{ProofOptions, SearchOutcome, backwards_search};

/// Runs a single depth-first backward search from `sequent`.
///
/// Creates a fresh [`BranchState`] and a zero step counter, then delegates to
/// [`backwards_search`].  The depth limit, step limit, timeout, and
/// quantifier budget are all taken from `options`.
///
/// # Parameters
///
/// - `sequent`: The root sequent to prove.
/// - `deadline`: Wall-clock deadline after which search returns
///   [`SearchOutcome::Timeout`].
/// - `options`: Proof-search limits (depth, steps, quantifier budget).
/// - `cancel_requested`: External cancellation flag.
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
    let state = BranchState::new();
    let mut steps_taken = 0usize;
    backwards_search(sequent, deadline, &state, options, cancel_requested, 0, &mut steps_taken)
}
