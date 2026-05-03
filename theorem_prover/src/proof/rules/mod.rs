//! Sequent-calculus rule definitions, matching, and application.
//!
//! File layout:
//! - `kinds.rs` defines the shared `Rule`, `Side`, and `RuleMatch` types.
//! - `matcher.rs` detects where rules apply in a sequent.
//! - `apply/structural.rs` handles the structural closing rules `Id`, `TopR`, and `BottomL`.
//! - `apply/connective.rs` handles the propositional rules `AndL`, `AndR`, `OrL`, `OrR`,
//!   `ImpliesL`, `ImpliesR`, `NotL`, and `NotR`.
//! - `apply/quantifier.rs` handles the quantified rules `ForAllL`, `ForAllR`, `ExistsL`,
//!   and `ExistsR`.

pub mod apply;
pub mod kinds;
pub mod matcher;

pub use kinds::{Rule, RuleMatch, Side};
pub use matcher::find_applicable_rules;
