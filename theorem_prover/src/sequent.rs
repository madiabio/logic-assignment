use crate::ast::Formula;
use crate::parser::ParsedProblem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequent {
    pub left: Vec<Formula>, // Gamma
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
