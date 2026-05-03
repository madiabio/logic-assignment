//! Public rule-application entry points used by proof search.

/// Applies a matched rule to a sequent and returns the resulting proof step.
pub use crate::proof::rules::apply::{RuleApplication, apply_rule};

/// Applies `∃R` with an explicit term chosen by the search scheduler.
pub(crate) use crate::proof::rules::apply::{apply_exists_r_with_term, apply_forall_l_with_term};
