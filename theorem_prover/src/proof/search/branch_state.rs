//! Per-branch bookkeeping used to avoid repeating quantifier instantiations.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ast::Term;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Search state that is threaded through a single proof branch.
pub(crate) struct BranchState {
    pub(crate) baseline_terms: Arc<Vec<Term>>,
    pub(crate) quantifier_usage: BTreeMap<String, QuantifierUsage>,
}

impl BranchState {
    /// Creates branch state with a frozen baseline term set shared by all descendants.
    pub(crate) fn new(baseline_terms: Vec<Term>) -> Self {
        Self {
            baseline_terms: Arc::new(baseline_terms),
            quantifier_usage: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Tracks which frozen baseline terms and fresh fallback have been tried for one occurrence.
pub(crate) struct QuantifierUsage {
    pub(crate) used_baseline_terms: BTreeSet<Term>,
    pub(crate) fresh_used: bool,
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
        usage.fresh_used = true;
    } else {
        usage.used_baseline_terms.insert(term.clone());
    }
}
