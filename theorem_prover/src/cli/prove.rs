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
    ProblemPipelineError, ProofStatus, RunProblemOptions, UnknownReason,
};
use theorem_prover::persistence::{self, ResultRecord};

/// Result of running the prover on one file.
#[derive(Clone)]
pub(crate) enum ProveFileResult {
    /// The file was processed successfully and produced a proof status.
    ///
    /// The second field carries the `UnknownReason` when the status is
    /// `ProofStatus::Unknown`; it is `None` for all other statuses.
    Status(ProofStatus, Option<UnknownReason>),
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
            ProveFileResult::Status(ProofStatus::Provable, _) => self.provable += 1,
            ProveFileResult::Status(ProofStatus::NotProvable, _) => self.not_provable += 1,
            ProveFileResult::Status(ProofStatus::Timeout, _) => self.timeout += 1,
            ProveFileResult::Status(ProofStatus::Unknown, _) => self.unknown += 1,
            ProveFileResult::Status(ProofStatus::Cancelled, _) => {
                self.cancelled += 1;
                self.interrupted_problem = Some(problem_run.problem_id());
            }
            ProveFileResult::Status(ProofStatus::NotImplemented, _) => {
                self.not_implemented += 1;
            }
            ProveFileResult::Status(ProofStatus::Error, _) => self.error += 1,
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
    db_state: Option<(rusqlite::Connection, i64)>,
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
    prove_paths(&problem_runs, options, cancellation, db_state);
}

/// Processes many problems through the prover, emits aggregate results, and
/// optionally persists each result to a SQLite database.
///
/// When `db_state` is `Some((conn, run_id))`, every successful problem result
/// is committed to the database immediately after it completes. The final
/// summary is then sourced from a `query_run_summary` DB query. When
/// `db_state` is `None`, the existing in-memory `ProveBatchSummary` is used.
pub(crate) fn prove_paths(
    problem_runs: &[ProblemRun],
    options: &ProveCommand,
    cancellation: &CancellationState,
    db_state: Option<(rusqlite::Connection, i64)>,
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

        let (result, elapsed_ms) = prove_file(problem_run, options, cancellation, index + 1, total);
        summary.record_result(problem_run, &result);

        if let Some((conn, run_id)) = db_state.as_ref() {
            if let Some(record) = result_record_for_problem(problem_run, &result, elapsed_ms) {
                if let Err(err) = persistence::insert_result(conn, *run_id, &record) {
                    eprintln!(
                        "warning: failed to persist result for {}: {err}",
                        problem_run.problem_id()
                    );
                }
            }
        }

        if matches!(result, ProveFileResult::Status(ProofStatus::Cancelled, _)) {
            break;
        }
    }

    // Print summary sourcing counts from DB when persistence is active.
    if let Some((conn, run_id)) = db_state.as_ref() {
        match persistence::query_run_summary(conn, *run_id) {
            Ok(db_summary) => {
                let get = |key: &str| db_summary.get(key).copied().unwrap_or(0).to_string();
                match options.format {
                    OutputFormat::Human => {
                        print_summary_header("summary");
                        print_summary_row(&[
                            ("processed", summary.processed.to_string()),
                            ("skipped", summary.skipped.to_string()),
                            ("provable", get("provable")),
                            ("not_provable", get("not_provable")),
                            ("timeout", get("timeout")),
                            ("unknown", get("unknown")),
                            ("cancelled", get("cancelled")),
                            ("not_impl", get("not_implemented")),
                            ("error", get("error")),
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
                            get("provable"),
                            get("not_provable"),
                            get("timeout"),
                            get("unknown"),
                            get("cancelled"),
                            get("not_implemented"),
                            get("error"),
                            summary.failed_to_process,
                            summary.interrupted_problem.as_deref().unwrap_or_default()
                        );
                    }
                }
            }
            Err(err) => {
                eprintln!(
                    "warning: failed to query run summary from DB: {err}"
                );
                print_in_memory_summary(options, &summary, cancellation);
            }
        }
    } else {
        print_in_memory_summary(options, &summary, cancellation);
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

/// Prints the in-memory `ProveBatchSummary` as the summary section.
fn print_in_memory_summary(
    options: &ProveCommand,
    summary: &ProveBatchSummary,
    cancellation: &CancellationState,
) {
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
}

/// Builds a SQLite result record for a completed problem, or `None` when the
/// file never reached proof execution.
pub(crate) fn result_record_for_problem(
    problem_run: &ProblemRun,
    result: &ProveFileResult,
    elapsed_ms: u128,
) -> Option<ResultRecord> {
    let (status, unknown_reason) = match result {
        ProveFileResult::Status(status, unknown_reason) => (status, unknown_reason),
        ProveFileResult::ProcessingFailure => return None,
    };

    let (formulae, atoms) = subset_stats_fields(problem_run.subset_stats);
    Some(ResultRecord {
        problem_id: problem_run.problem_id(),
        path: problem_run.path.display().to_string(),
        status: crate::cli::output::human_proof_status(status).to_string(),
        elapsed_ms,
        formulae: if problem_run.subset_stats.is_some() {
            Some(formulae as i64)
        } else {
            None
        },
        atoms: if problem_run.subset_stats.is_some() {
            Some(atoms as i64)
        } else {
            None
        },
        unknown_reason: unknown_reason.map(|reason| human_unknown_reason(reason).to_string()),
    })
}

/// Runs the prover for one file and returns either a proof status or a
/// processing failure, along with the internally-measured elapsed time in milliseconds.
pub(crate) fn prove_file(
    problem_run: &ProblemRun,
    options: &ProveCommand,
    cancellation: &CancellationState,
    current: usize,
    total: usize,
) -> (ProveFileResult, u128) {
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
            let unknown_reason = result.unknown_reason;
            let detail = unknown_reason
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
            (ProveFileResult::Status(status, unknown_reason), elapsed_ms)
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
            (ProveFileResult::ProcessingFailure, started_at.elapsed().as_millis())
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
            (ProveFileResult::ProcessingFailure, started_at.elapsed().as_millis())
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
            (ProveFileResult::ProcessingFailure, started_at.elapsed().as_millis())
        }
    }
}

/// Prints single-file prover status and exits non-zero on processing failure.
pub(crate) fn report_single_prove_file(result: ProveFileResult) {
    match result {
        ProveFileResult::Status(ProofStatus::Cancelled, _) => {
            std::process::exit(EXIT_CANCELLED);
        }
        ProveFileResult::Status(_, _) => {}
        ProveFileResult::ProcessingFailure => {
            std::process::exit(EXIT_FAILURE);
        }
    }
}

#[cfg(test)]
#[path = "prove_persist_tests.rs"]
mod prove_persist_tests;
