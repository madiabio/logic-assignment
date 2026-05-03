use crate::{
    ParsedProblem, ProofOptions, ProofResult, Sequent, SequentBuildError, parse_problem, prove,
};

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
    run_problem_with_options(input, ProofOptions::default())
}

// Prints the sequent before running the problem
pub fn run_problem_verbose(
    input: &str,
    show_sequent: bool,
) -> Result<ProofResult, ProblemPipelineError> {
    run_problem_verbose_with_options(input, show_sequent, ProofOptions::default())
}

pub fn run_problem_with_options(
    input: &str,
    options: ProofOptions,
) -> Result<ProofResult, ProblemPipelineError> {
    run_problem_verbose_with_options(input, false, options)
}

// Prints the sequent before running the problem with explicit prover options.
pub fn run_problem_verbose_with_options(
    input: &str,
    show_sequent: bool,
    options: ProofOptions,
) -> Result<ProofResult, ProblemPipelineError> {
    let sequent = build_problem_sequent(input)?;
    if show_sequent {
        println!("{sequent}");
    }
    Ok(prove(&sequent, options))
}
