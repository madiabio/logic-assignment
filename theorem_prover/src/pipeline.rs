use std::sync::atomic::AtomicBool;

use crate::{
    ParsedProblem, ProofOptions, ProofResult, Sequent, SequentBuildError, UnknownReason,
    parse_problem, prove_with_cancel,
};

/// Pre-search input policy shared by the CLI and pipeline helpers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BiconditionalPolicy {
    /// Maximum number of non-comment `"<=>"` tokens allowed in one input before
    /// the pipeline returns an inconclusive result without parsing or search.
    ///
    /// `None` disables the gate entirely.
    pub max_biconditionals: Option<usize>,
}

impl BiconditionalPolicy {
    /// Returns whether the raw input exceeds the configured biconditional cap.
    pub fn is_exceeded_by(&self, input: &str) -> bool {
        let Some(limit) = self.max_biconditionals else {
            return false;
        };

        count_non_comment_biconditionals(input) > limit
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProblemPipelineError {
    Parse(String),
    SequentBuild(SequentBuildError),
}

/// Builds the initial sequent after applying the configured input policy.
pub fn build_problem_sequent_with_policy(
    input: &str,
    policy: BiconditionalPolicy,
) -> Result<Sequent, ProblemPipelineError> {
    if policy.is_exceeded_by(input) {
        return Err(ProblemPipelineError::Parse(
            "biconditional cap exceeded before parsing".to_string(),
        ));
    }

    build_problem_sequent(input)
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
    run_problem_with_options_and_policy(input, options, BiconditionalPolicy::default())
}

/// Runs a problem with explicit prover options and biconditional input policy.
pub fn run_problem_with_options_and_policy(
    input: &str,
    options: ProofOptions,
    policy: BiconditionalPolicy,
) -> Result<ProofResult, ProblemPipelineError> {
    run_problem_verbose_with_options_and_policy(input, false, options, policy)
}

/// Runs a problem with explicit prover options and a cancellation flag.
pub fn run_problem_with_options_and_cancel(
    input: &str,
    options: ProofOptions,
    cancel_requested: &AtomicBool,
) -> Result<ProofResult, ProblemPipelineError> {
    run_problem_verbose_with_options_and_cancel(input, false, options, cancel_requested)
}

// Prints the sequent before running the problem with explicit prover options.
pub fn run_problem_verbose_with_options(
    input: &str,
    show_sequent: bool,
    options: ProofOptions,
) -> Result<ProofResult, ProblemPipelineError> {
    static NEVER_CANCELLED: AtomicBool = AtomicBool::new(false);
    run_problem_verbose_with_options_and_policy_and_cancel(
        input,
        show_sequent,
        options,
        BiconditionalPolicy::default(),
        &NEVER_CANCELLED,
    )
}

/// Prints the sequent before running the problem with explicit prover options
/// and biconditional input policy.
pub fn run_problem_verbose_with_options_and_policy(
    input: &str,
    show_sequent: bool,
    options: ProofOptions,
    policy: BiconditionalPolicy,
) -> Result<ProofResult, ProblemPipelineError> {
    static NEVER_CANCELLED: AtomicBool = AtomicBool::new(false);
    run_problem_verbose_with_options_and_policy_and_cancel(
        input,
        show_sequent,
        options,
        policy,
        &NEVER_CANCELLED,
    )
}

/// Prints the sequent before running the problem with explicit prover options
/// and an external cancellation flag.
pub fn run_problem_verbose_with_options_and_cancel(
    input: &str,
    show_sequent: bool,
    options: ProofOptions,
    cancel_requested: &AtomicBool,
) -> Result<ProofResult, ProblemPipelineError> {
    run_problem_verbose_with_options_and_policy_and_cancel(
        input,
        show_sequent,
        options,
        BiconditionalPolicy::default(),
        cancel_requested,
    )
}

/// Prints the sequent before running the problem with explicit prover options,
/// biconditional input policy, and an external cancellation flag.
pub fn run_problem_verbose_with_options_and_policy_and_cancel(
    input: &str,
    show_sequent: bool,
    options: ProofOptions,
    policy: BiconditionalPolicy,
    cancel_requested: &AtomicBool,
) -> Result<ProofResult, ProblemPipelineError> {
    if policy.is_exceeded_by(input) {
        return Ok(ProofResult {
            status: crate::ProofStatus::Unknown,
            unknown_reason: Some(UnknownReason::BiconditionalCapExceeded),
        });
    }

    let sequent = build_problem_sequent(input)?;
    if show_sequent {
        println!("{sequent}");
    }
    Ok(prove_with_cancel(&sequent, options, cancel_requested))
}

fn count_non_comment_biconditionals(input: &str) -> usize {
    let mut count = 0usize;
    for line in input.lines() {
        let line = line.trim_start();
        if line.starts_with('%') {
            continue;
        }
        count += line.matches("<=>").count();
    }
    count
}
