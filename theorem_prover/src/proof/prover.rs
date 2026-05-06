//! Public proof-search API re-exported from the search engine.

/// Attempts to prove a sequent within the supplied proof options.
pub use crate::proof::search::engine::{
    ProofOptions, ProofResult, ProofStatus, SearchEngine, UnknownReason, prove, prove_with_cancel,
};
