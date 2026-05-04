use std::sync::atomic::AtomicBool;

use crate::parser::Rule;
use crate::{
    ParsedProblem, ProofOptions, ProofResult, Sequent, SequentBuildError, UnknownReason,
    parse_problem, parse_tptp, prove_with_cancel,
};
use pest::iterators::Pair;

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

/// Options for one end-to-end problem run through the parsing and proving
/// pipeline.
#[derive(Debug, Clone, Copy)]
pub struct RunProblemOptions<'a> {
    /// Whether to print the constructed sequent before proof search.
    pub show_sequent: bool,
    /// Proof-search bounds and timeout settings.
    pub proof: ProofOptions,
    /// Pre-search input policy for large biconditional chains.
    pub biconditional_policy: BiconditionalPolicy,
    /// Optional external cancellation flag observed during proof search.
    pub cancel_requested: Option<&'a AtomicBool>,
}

impl Default for RunProblemOptions<'static> {
    fn default() -> Self {
        Self {
            show_sequent: false,
            proof: ProofOptions::default(),
            biconditional_policy: BiconditionalPolicy::default(),
            cancel_requested: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProblemPipelineError {
    Parse(String),
    UnsupportedInclude,
    SequentBuild(SequentBuildError),
}

/// Builds the initial sequent for one parsed problem input.
pub fn build_problem_sequent(input: &str) -> Result<Sequent, ProblemPipelineError> {
    if contains_include_directive(input)? {
        return Err(ProblemPipelineError::UnsupportedInclude);
    }

    let parsed: ParsedProblem =
        parse_problem(input).map_err(|err| ProblemPipelineError::Parse(err.to_string()))?;
    Sequent::from_parsed_problem(parsed).map_err(ProblemPipelineError::SequentBuild)
}

/// Runs a problem with default pipeline and proof-search options.
pub fn run_problem(input: &str) -> Result<ProofResult, ProblemPipelineError> {
    run_problem_with_options(input, RunProblemOptions::default())
}

/// Runs a problem with explicit pipeline options.
pub fn run_problem_with_options(
    input: &str,
    options: RunProblemOptions<'_>,
) -> Result<ProofResult, ProblemPipelineError> {
    if options.biconditional_policy.is_exceeded_by(input) {
        return Ok(ProofResult {
            status: crate::ProofStatus::Unknown,
            unknown_reason: Some(UnknownReason::BiconditionalCapExceeded),
        });
    }

    if contains_include_directive(input)? {
        return Ok(ProofResult {
            status: crate::ProofStatus::Unknown,
            unknown_reason: Some(UnknownReason::UnsupportedInclude),
        });
    }

    static NEVER_CANCELLED: AtomicBool = AtomicBool::new(false);
    let cancel_requested = options.cancel_requested.unwrap_or(&NEVER_CANCELLED);
    let sequent = build_problem_sequent(input)?;
    if options.show_sequent {
        println!("{sequent}");
    }
    Ok(prove_with_cancel(&sequent, options.proof, cancel_requested))
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

fn contains_include_directive(input: &str) -> Result<bool, ProblemPipelineError> {
    let pairs = parse_tptp(input).map_err(|err| ProblemPipelineError::Parse(err.to_string()))?;
    Ok(pairs.into_iter().any(pair_contains_include_directive))
}

fn pair_contains_include_directive(pair: Pair<'_, Rule>) -> bool {
    pair.as_rule() == Rule::include_directive
        || pair.into_inner().any(pair_contains_include_directive)
}
