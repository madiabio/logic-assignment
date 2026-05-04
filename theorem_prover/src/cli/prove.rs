//! Execution flow for the `prove` CLI subcommand.

use crate::cli::args::{OutputFormat, ProveCommand};
use crate::cli::cancel::{CancellationState, EXIT_CANCELLED, EXIT_FAILURE, prove_batch_exit_code};
use crate::cli::config::{biconditional_policy_from_cli, prover_options_from_cli};
use crate::cli::output::{
    human_proof_result, human_unknown_reason, print_prove_human_row, print_prove_preamble,
    print_summary_header, print_summary_row,
};
use crate::cli::parse_failure::{
    clear_parse_failure_marker, should_skip_parse_failed_file, write_parse_failure_marker,
};
use crate::cli::subset::{ProblemRun, subset_stats_fields};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use theorem_prover::{
    ProblemPipelineError, ProofStatus, RunProblemOptions,
};

/// Result of running the prover on one file.
#[derive(Clone)]
pub(crate) enum ProveFileResult {
    /// The file was processed successfully and produced a proof status.
    Status(ProofStatus),
    /// The file could not be processed because parsing or sequent building failed.
    ProcessingFailure,
}

/// Running counts and metadata for `prove` batch execution.
#[derive(Default)]
pub(crate) struct ProveBatchSummary {
    processed: usize,
    skipped: usize,
    provable: usize,
    not_provable: usize,
    timeout: usize,
    unknown: usize,
    cancelled: usize,
    not_implemented: usize,
    error: usize,
    failed_to_process: usize,
    interrupted_problem: Option<String>,
    failed_files: Vec<PathBuf>,
}

impl ProveBatchSummary {
    /// Records the outcome of one processed problem.
    fn record_result(&mut self, problem_run: &ProblemRun, result: &ProveFileResult) {
        self.processed += 1;
        match result {
            ProveFileResult::Status(ProofStatus::Provable) => self.provable += 1,
            ProveFileResult::Status(ProofStatus::NotProvable) => self.not_provable += 1,
            ProveFileResult::Status(ProofStatus::Timeout) => self.timeout += 1,
            ProveFileResult::Status(ProofStatus::Unknown) => self.unknown += 1,
            ProveFileResult::Status(ProofStatus::Cancelled) => {
                self.cancelled += 1;
                self.interrupted_problem = Some(problem_run.problem_id());
            }
            ProveFileResult::Status(ProofStatus::NotImplemented) => self.not_implemented += 1,
            ProveFileResult::Status(ProofStatus::Error) => self.error += 1,
            ProveFileResult::ProcessingFailure => {
                self.failed_to_process += 1;
                self.failed_files.push(problem_run.path.clone());
            }
        }
    }
}

/// Runs the prover over every `.p` file in a directory and prints per-status totals.
pub(crate) fn prove_directory(
    dir: &Path,
    options: &ProveCommand,
    cancellation: &CancellationState,
    settings: &str,
) {
    let entries = fs::read_dir(dir).expect("Failed to read directory");
    let mut problem_runs = Vec::new();
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("p") {
            continue;
        }

        problem_runs.push(ProblemRun {
            path,
            subset_stats: None,
        });
    }

    print_prove_preamble(options.format, None, settings);
    prove_paths(&problem_runs, options, cancellation);
}

/// Processes many problems through the prover and emits aggregate results.
pub(crate) fn prove_paths(
    problem_runs: &[ProblemRun],
    options: &ProveCommand,
    cancellation: &CancellationState,
) {
    let mut summary = ProveBatchSummary::default();
    let total = problem_runs.len();
    for (index, problem_run) in problem_runs.iter().enumerate() {
        if cancellation.is_requested() {
            break;
        }

        if should_skip_parse_failed_file(&problem_run.path, options) {
            summary.skipped += 1;
            continue;
        }

        let result = prove_file(problem_run, options, cancellation, index + 1, total);
        summary.record_result(problem_run, &result);

        if matches!(result, ProveFileResult::Status(ProofStatus::Cancelled)) {
            break;
        }
    }

    match options.format {
        OutputFormat::Human => {
            print_summary_header("summary");
            print_summary_row(&[
                ("processed", summary.processed.to_string()),
                ("skipped", summary.skipped.to_string()),
                ("provable", summary.provable.to_string()),
                ("not_provable", summary.not_provable.to_string()),
                ("timeout", summary.timeout.to_string()),
                ("unknown", summary.unknown.to_string()),
                ("cancelled", summary.cancelled.to_string()),
                ("not_impl", summary.not_implemented.to_string()),
                ("error", summary.error.to_string()),
                ("failed_to_process", summary.failed_to_process.to_string()),
            ]);
            if let Some(problem_id) = &summary.interrupted_problem {
                eprintln!("Cancelled while proving {problem_id}");
            } else if cancellation.is_requested() {
                eprintln!("Cancelled before starting the next problem");
            }
        }
        OutputFormat::Tsv => {
            println!(
                "summary\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                summary.processed,
                summary.skipped,
                summary.provable,
                summary.not_provable,
                summary.timeout,
                summary.unknown,
                summary.cancelled,
                summary.not_implemented,
                summary.error,
                summary.failed_to_process,
                summary.interrupted_problem.as_deref().unwrap_or_default()
            );
        }
    }

    if options.format == OutputFormat::Human && !summary.failed_files.is_empty() {
        eprintln!("Failed files:");
        for path in &summary.failed_files {
            eprintln!("  {}", path.display());
        }
    }

    if let Some(code) =
        prove_batch_exit_code(summary.cancelled, summary.failed_to_process, cancellation)
    {
        std::process::exit(code);
    }
}

/// Runs the prover for one file and returns either a proof status or a
/// processing failure.
pub(crate) fn prove_file(
    problem_run: &ProblemRun,
    options: &ProveCommand,
    cancellation: &CancellationState,
    current: usize,
    total: usize,
) -> ProveFileResult {
    let proof_options = prover_options_from_cli(options);
    let biconditional_policy = biconditional_policy_from_cli(options.run.max_biconditionals);
    let started_at = Instant::now();
    let problem_id = problem_run.problem_id();
    let (formulae, atoms) = subset_stats_fields(problem_run.subset_stats);

    match theorem_prover::run_problem_from_path_with_options(
        &problem_run.path,
        RunProblemOptions {
            show_sequent: options.display.show_sequent,
            proof: proof_options,
            biconditional_policy,
            cancel_requested: Some(cancellation.flag()),
        },
    ) {
        Ok(result) => {
            clear_parse_failure_marker(&problem_run.path);
            let elapsed_ms = started_at.elapsed().as_millis();
            let status = result.status.clone();
            let detail = result
                .unknown_reason
                .map(human_unknown_reason)
                .unwrap_or_default();
            let human_status = human_proof_result(&result);
            match options.format {
                OutputFormat::Human => print_prove_human_row(
                    current,
                    total,
                    &problem_id,
                    human_status.as_str(),
                    elapsed_ms,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => println!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\t{:?}\t{elapsed_ms}\t{detail}",
                    problem_run.path.display(),
                    status
                ),
            }
            ProveFileResult::Status(status)
        }
        Err(ProblemPipelineError::Parse(err)) => {
            write_parse_failure_marker(&problem_run.path, &err);
            match options.format {
                OutputFormat::Human => print_prove_human_row(
                    current,
                    total,
                    &problem_id,
                    "parse_failed",
                    0,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => eprintln!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\tparse_failed\t0",
                    problem_run.path.display()
                ),
            }
            eprintln!("{err}");
            ProveFileResult::ProcessingFailure
        }
        Err(ProblemPipelineError::Include(err)) => {
            clear_parse_failure_marker(&problem_run.path);
            match options.format {
                OutputFormat::Human => print_prove_human_row(
                    current,
                    total,
                    &problem_id,
                    "include_failed",
                    0,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => eprintln!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\tinclude_failed\t0",
                    problem_run.path.display()
                ),
            }
            eprintln!("{err}");
            ProveFileResult::ProcessingFailure
        }
        Err(ProblemPipelineError::SequentBuild(err)) => {
            match options.format {
                OutputFormat::Human => print_prove_human_row(
                    current,
                    total,
                    &problem_id,
                    "sequent_build_failed",
                    0,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => eprintln!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\tsequent_build_failed\t0",
                    problem_run.path.display()
                ),
            }
            eprintln!("sequent construction failed: {err:?}");
            ProveFileResult::ProcessingFailure
        }
    }
}

/// Prints single-file prover status and exits non-zero on processing failure.
pub(crate) fn report_single_prove_file(result: ProveFileResult) {
    match result {
        ProveFileResult::Status(ProofStatus::Cancelled) => {
            std::process::exit(EXIT_CANCELLED);
        }
        ProveFileResult::Status(_) => {}
        ProveFileResult::ProcessingFailure => {
            std::process::exit(EXIT_FAILURE);
        }
    }
}
