use crate::{ProofResult, Sequent, SequentBuildError, parse_problem, prove};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProblemPipelineError {
    Parse(String),
    SequentBuild(SequentBuildError),
}

pub fn run_problem(input: &str) -> Result<ProofResult, ProblemPipelineError> {
    let parsed =
        parse_problem(input).map_err(|err| ProblemPipelineError::Parse(err.to_string()))?;
    let sequent =
        Sequent::from_parsed_problem(parsed).map_err(ProblemPipelineError::SequentBuild)?;
    println!("{sequent}");
    Ok(prove(&sequent))
}
