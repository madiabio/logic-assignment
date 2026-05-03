//! Per-branch bookkeeping used to avoid repeating quantifier instantiations.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::Term;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Search state that is threaded through a single proof branch.
pub(crate) struct BranchState {
    pub(crate) quantifier_usage: BTreeMap<String, QuantifierUsage>,
}

impl BranchState {
    /// Creates empty branch state. Candidate terms are read from the current sequent.
    pub(crate) fn new() -> Self {
        Self {
            quantifier_usage: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Tracks which terms have been tried for one quantified occurrence.
pub(crate) struct QuantifierUsage {
    pub(crate) used_terms: BTreeSet<Term>,
    pub(crate) fresh_terms_used: usize,
}

/// Records that a quantified occurrence has been instantiated with the given term.
pub(crate) fn record_quantifier_term(
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
