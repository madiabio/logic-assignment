// Defines sequents and their display properties.
use std::fmt;

use crate::ast::Formula;
use crate::parser::ParsedProblem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequent {
    pub left: Vec<Formula>,  // Gamma
    pub right: Vec<Formula>, // Delta
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequentBuildError {
    MissingConjecture,
}

impl Sequent {
    pub fn from_parsed_problem(parsed: ParsedProblem) -> Result<Self, SequentBuildError> {
        let ParsedProblem {
            premises,
            conjecture,
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
