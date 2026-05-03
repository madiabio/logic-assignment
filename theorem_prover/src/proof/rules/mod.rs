pub mod apply;
pub mod kinds;
pub mod matcher;

pub use kinds::{Rule, RuleMatch, Side};
pub use matcher::find_applicable_rules;
