//! Per-branch bookkeeping used to avoid repeating quantifier instantiations.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::Term;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Search state that is threaded through a single proof branch.
pub(crate) struct BranchState {
    pub(crate) quantifier_usage: BTreeMap<String, QuantifierUsage>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Tracks which terms have been tried for a quantified occurrence.
pub(crate) struct QuantifierUsage {
    pub(crate) used_terms: BTreeSet<String>,
    pub(crate) fresh_fallback_used: bool,
}

/// Records that a quantified occurrence has been instantiated with the given term.
pub(crate) fn record_quantifier_term(
    state: &mut BranchState,
    key: &str,
    term: &Term,
    fresh_fallback: bool,
) {
    let usage = state.quantifier_usage.entry(key.to_owned()).or_default();
    usage.used_terms.insert(term.to_string());
    if fresh_fallback {
        usage.fresh_fallback_used = true;
    }
}
