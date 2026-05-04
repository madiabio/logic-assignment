//! Fallback runtime defaults for proof search.
//!
//! These values are used by [`crate::ProofOptions::default`] when no caller
//! supplies explicit bounds. CLI runs may override them through `config.toml`
//! and command-line flags.

use std::time::Duration;

/// Default wall-clock timeout for one proof attempt.
pub const DEFAULT_PROVE_TIMEOUT: Duration = Duration::from_secs(50);

/// Default maximum recursive proof-search depth.
pub const DEFAULT_MAX_DEPTH: usize = 128;

/// Default maximum number of proof-search steps.
pub const DEFAULT_MAX_STEPS: usize = 50_000;

/// Default maximum number of fresh fallback terms per quantified occurrence.
pub const DEFAULT_MAX_FRESH_TERMS_PER_QUANTIFIER: usize = 1;
