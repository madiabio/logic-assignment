pub mod ast;
pub mod parser;
pub mod persistence;
pub mod pipeline;
pub mod proof;

pub use parser::{FormulaRecord, IncludeDirective, ParsedProblem, parse_problem, parse_tptp};
pub use pipeline::*;
pub use proof::prover::*;
pub use proof::sequent::*;

// Re-exports used only in integration tests. Not part of the stable public API.
#[doc(hidden)]
pub use proof::search::scheduler::{ScheduleResult, ScheduledRule, quantified_occurrence_key, schedule_next_rules_lk_priority};
#[doc(hidden)]
pub use proof::search::branch_state::{BranchState, record_quantifier_term};
#[doc(hidden)]
pub use proof::rules::{Rule, Side, RuleMatch};
