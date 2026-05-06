//! Per-branch bookkeeping used to avoid repeating quantifier instantiations.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::Term;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Search state that is threaded through a single proof branch.
pub struct BranchState {
    pub(crate) quantifier_usage: BTreeMap<String, QuantifierUsage>,
}

impl BranchState {
    /// Creates empty branch state. Candidate terms are read from the current sequent.
    pub fn new() -> Self {
        Self {
            quantifier_usage: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Tracks which terms have been tried for one quantified occurrence.
pub(crate) struct QuantifierUsage {
    pub used_terms: BTreeSet<Term>,
    pub fresh_terms_used: usize,
}

/// Records the use of a term for a specific quantified formula occurrence on this branch.
///
/// This updates the branch-local quantifier usage state so that:
///
/// - The same term is not reused for the same quantified occurrence.
/// - Fresh-term usage is counted toward the per-quantifier budget.
///
/// This function should be called whenever a `∀L` or `∃R` rule is applied
/// with a chosen instantiation term.
///
/// # Parameters
///
/// - `state`: The current branch state tracking quantifier instantiation history.
/// - `key`: A stable identifier for the quantified occurrence (see
///   [`quantified_occurrence_key`]).
/// - `term`: The term used to instantiate the quantifier.
/// - `fresh_fallback`: Whether this term was introduced as a fresh fallback
///   (as opposed to reusing an existing term).
///
/// # Effects
///
/// - Inserts `term` into the set of used terms for this occurrence.
/// - Increments the fresh-term counter if `fresh_fallback` is `true`.
///
/// This ensures that future scheduling avoids repeating the same instantiations
/// and respects the configured fresh-term budget.
pub fn record_quantifier_term(
    state: &mut BranchState,
    key: &str,
    term: &Term,
    fresh_fallback: bool,
) {
    let usage = state.quantifier_usage.entry(key.to_owned()).or_default();
    if fresh_fallback {
        usage.fresh_terms_used += 1;
    }
    usage.used_terms.insert(term.clone());
}
