//! Defines sequents and conversions from parsed problem statements.

use std::fmt;

use crate::ast::Formula;
use crate::parser::ParsedProblem;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A sequent `Gamma ⊢ Delta` with antecedents on the left and succedents on the right.
pub struct Sequent {
    pub left: Vec<Formula>,  // Gamma
    pub right: Vec<Formula>, // Delta
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors that can occur while building an initial sequent from parsed input.
pub enum SequentBuildError {
    /// The parsed problem did not contain a conjecture to place on the right.
    MissingConjecture,
}

impl Sequent {
    /// Builds the initial proof sequent from parsed premises and conjecture.
    pub fn from_parsed_problem(parsed: ParsedProblem) -> Result<Self, SequentBuildError> {
        let ParsedProblem {
            premises,
            conjecture,
            ..
        } = parsed;

        let left = premises.into_iter().map(|record| record.formula).collect();
        let conjecture = conjecture.ok_or(SequentBuildError::MissingConjecture)?;

        Ok(Self {
            left,
            right: vec![conjecture.formula],
        })
    }
}

impl fmt::Display for Sequent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, formula) in self.left.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{formula}")?;
        }

        if !self.left.is_empty() {
            f.write_str(" ")?;
        }

        f.write_str("⊢")?;

        if !self.right.is_empty() {
            f.write_str(" ")?;
            for (index, formula) in self.right.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{formula}")?;
            }
        }

        Ok(())
    }
}
