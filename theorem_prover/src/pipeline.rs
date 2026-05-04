use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
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
    Include(String),
    SequentBuild(SequentBuildError),
}

/// Builds the initial sequent for one parsed problem input.
pub fn build_problem_sequent(input: &str) -> Result<Sequent, ProblemPipelineError> {
    let parsed: ParsedProblem =
        parse_problem(input).map_err(|err| ProblemPipelineError::Parse(err.to_string()))?;
    if !parsed.includes.is_empty() {
        return Err(ProblemPipelineError::Include(
            "include directives require path-aware loading".to_string(),
        ));
    }
    Sequent::from_parsed_problem(parsed).map_err(ProblemPipelineError::SequentBuild)
}

/// Builds the initial sequent for one problem file after recursively loading includes.
pub fn build_problem_sequent_from_path(path: &Path) -> Result<Sequent, ProblemPipelineError> {
    let loaded = load_problem_from_path(path)?;
    Sequent::from_parsed_problem(loaded.parsed).map_err(ProblemPipelineError::SequentBuild)
}

/// Runs a problem with default pipeline and proof-search options.
pub fn run_problem(input: &str) -> Result<ProofResult, ProblemPipelineError> {
    run_problem_with_options(input, RunProblemOptions::default())
}

/// Runs one on-disk problem after recursively loading includes.
pub fn run_problem_from_path(path: &Path) -> Result<ProofResult, ProblemPipelineError> {
    run_problem_from_path_with_options(path, RunProblemOptions::default())
}

fn run_problem_impl(
    sequent: &Sequent,
    options: RunProblemOptions<'_>,
) -> Result<ProofResult, ProblemPipelineError> {
    static NEVER_CANCELLED: AtomicBool = AtomicBool::new(false);
    let cancel_requested = options.cancel_requested.unwrap_or(&NEVER_CANCELLED);
    if options.show_sequent {
        println!("{sequent}");
    }
    Ok(prove_with_cancel(sequent, options.proof, cancel_requested))
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

    let sequent = build_problem_sequent(input)?;
    run_problem_impl(&sequent, options)
}

/// Runs one on-disk problem with explicit pipeline options after recursively loading includes.
pub fn run_problem_from_path_with_options(
    path: &Path,
    options: RunProblemOptions<'_>,
) -> Result<ProofResult, ProblemPipelineError> {
    let loaded = load_problem_from_path(path)?;
    if options.biconditional_policy.is_exceeded_by(&loaded.input) {
        return Ok(ProofResult {
            status: crate::ProofStatus::Unknown,
            unknown_reason: Some(UnknownReason::BiconditionalCapExceeded),
        });
    }

    let sequent = Sequent::from_parsed_problem(loaded.parsed)
        .map_err(ProblemPipelineError::SequentBuild)?;
    run_problem_impl(&sequent, options)
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

struct LoadedProblem {
    input: String,
    parsed: ParsedProblem,
}

fn load_problem_from_path(path: &Path) -> Result<LoadedProblem, ProblemPipelineError> {
    let canonical_path = canonicalize_problem_path(path)?;
    let include_root = infer_include_root(&canonical_path)?;
    let mut loaded = HashSet::new();
    let mut stack = Vec::new();
    let input = read_problem_text(&canonical_path)?;
    let parsed = load_problem_recursive(
        &canonical_path,
        &include_root,
        true,
        Some(&input),
        &mut loaded,
        &mut stack,
    )?;
    Ok(LoadedProblem { input, parsed })
}

fn load_problem_recursive(
    canonical_path: &Path,
    include_root: &Path,
    is_top_level: bool,
    top_level_input: Option<&str>,
    loaded: &mut HashSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
) -> Result<ParsedProblem, ProblemPipelineError> {
    if stack.iter().any(|path| path == canonical_path) {
        return Err(ProblemPipelineError::Include(format!(
            "include cycle detected at {}",
            canonical_path.display()
        )));
    }

    if !loaded.insert(canonical_path.to_path_buf()) {
        return Ok(ParsedProblem::default());
    }

    stack.push(canonical_path.to_path_buf());
    let input = if is_top_level {
        top_level_input
            .map(str::to_owned)
            .unwrap_or_else(|| read_problem_text(canonical_path).expect(
                "canonicalized problem path should still be readable",
            ))
    } else {
        read_problem_text(canonical_path)?
    };
    let mut parsed =
        parse_problem(&input).map_err(|err| ProblemPipelineError::Parse(err.to_string()))?;

    if !is_top_level && parsed.conjecture.is_some() {
        stack.pop();
        return Err(ProblemPipelineError::Include(format!(
            "included file {} contains a conjecture",
            canonical_path.display()
        )));
    }

    let mut merged_premises = parsed.premises;
    let conjecture = parsed.conjecture.take();
    for include in parsed.includes {
        let include_path = resolve_include_path(include_root, &include.path)?;
        let included = load_problem_recursive(
            &include_path,
            include_root,
            false,
            None,
            loaded,
            stack,
        )?;
        merged_premises.extend(included.premises);
    }

    stack.pop();
    Ok(ParsedProblem {
        premises: merged_premises,
        conjecture,
        includes: Vec::new(),
    })
}

fn read_problem_text(path: &Path) -> Result<String, ProblemPipelineError> {
    match fs::read_to_string(path) {
        Ok(input) => Ok(input),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(ProblemPipelineError::Include(
            format!("failed to read {}: {err}", path.display()),
        )),
        Err(err) => Err(ProblemPipelineError::Include(format!(
            "failed to read {}: {err}",
            path.display()
        ))),
    }
}

fn canonicalize_problem_path(path: &Path) -> Result<PathBuf, ProblemPipelineError> {
    path.canonicalize().map_err(|err| {
        ProblemPipelineError::Include(format!("failed to resolve {}: {err}", path.display()))
    })
}

fn resolve_include_path(
    include_root: &Path,
    include_path: &str,
) -> Result<PathBuf, ProblemPipelineError> {
    let joined = include_root.join(include_path);
    joined.canonicalize().map_err(|err| {
        ProblemPipelineError::Include(format!("failed to resolve include {include_path}: {err}"))
    })
}

fn infer_include_root(problem_path: &Path) -> Result<PathBuf, ProblemPipelineError> {
    for ancestor in problem_path.ancestors() {
        let Some(name) = ancestor.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if matches!(name, "Problems" | "Axioms") {
            return ancestor.parent().map(Path::to_path_buf).ok_or_else(|| {
                ProblemPipelineError::Include(format!(
                    "failed to determine include root for {}",
                    problem_path.display()
                ))
            });
        }
    }

    problem_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            ProblemPipelineError::Include(format!(
                "failed to determine include root for {}",
                problem_path.display()
            ))
        })
}
