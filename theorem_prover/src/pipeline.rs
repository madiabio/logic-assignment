use crate::{ParsedProblem, ProofResult, Sequent, SequentBuildError, parse_problem, prove};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProblemPipelineError {
    Parse(String),
    SequentBuild(SequentBuildError),
}

pub fn build_problem_sequent(input: &str) -> Result<Sequent, ProblemPipelineError> {
    let parsed: ParsedProblem =
        parse_problem(input).map_err(|err| ProblemPipelineError::Parse(err.to_string()))?;
    Sequent::from_parsed_problem(parsed).map_err(ProblemPipelineError::SequentBuild)
}

pub fn run_problem(input: &str) -> Result<ProofResult, ProblemPipelineError> {
    let sequent = build_problem_sequent(input)?;
    println!("{sequent}");
    Ok(prove(&sequent))
}
