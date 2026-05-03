use crate::cli::args::{OutputFormat, ParseFailureOptions, ProveCommand, RulesCommand};
use crate::cli::config::{ensure_config, prover_options_from_cli};
use crate::cli::output::{
    human_proof_status, print_prove_human_row, print_prove_preamble, print_rules_human_row,
    print_rules_preamble, print_summary_header, print_summary_row,
};
use crate::cli::subset::{ProblemRun, resolve_subset_targets, subset_stats_fields};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Mutex,
};
use std::time::Instant;
use theorem_prover::proof::rules::{RuleMatch, find_applicable_rules};
use theorem_prover::{
    ProblemPipelineError, ProofStatus, build_problem_sequent,
    run_problem_verbose_with_options_and_cancel,
};

/// Outcome of running rule inspection on one file.
#[derive(Clone, Copy)]
struct RulesInspectionResult {
    success: bool,
    had_rule_match: bool,
}

/// Result of running the prover on one file.
#[derive(Clone)]
enum ProveFileResult {
    /// The file was processed successfully and produced a proof status.
    Status(ProofStatus),
    /// The file could not be processed because parsing or sequent building failed.
    ProcessingFailure,
}

/// Shared cancellation state driven by the process `Ctrl+C` handler.
#[derive(Clone)]
struct CancellationState {
    requested: Arc<AtomicBool>,
    next_problem: Arc<Mutex<Option<String>>>,
}

impl CancellationState {
    /// Installs a `Ctrl+C` handler and exposes its atomic cancellation flag.
    fn install() -> Self {
        let requested = Arc::new(AtomicBool::new(false));
        let interrupt_count = Arc::new(AtomicUsize::new(0));
        let next_problem = Arc::new(Mutex::new(None));
        let handler_flag = Arc::clone(&requested);
        let handler_count = Arc::clone(&interrupt_count);
        let handler_next_problem = Arc::clone(&next_problem);
        ctrlc::set_handler(move || {
            let count = handler_count.fetch_add(1, Ordering::Relaxed) + 1;
            if count == 1 {
                handler_flag.store(true, Ordering::Relaxed);
                eprintln!("Cancellation requested. Press Ctrl+C again to force quit.");
            } else {
                if let Ok(guard) = handler_next_problem.lock() {
                    if let Some(next_problem) = &*guard {
                        eprintln!("Next problem was: {next_problem}");
                    }
                }
                eprintln!("Force quitting.");
                std::process::exit(130);
            }
        })
        .expect("failed to install Ctrl+C handler");
        Self {
            requested,
            next_problem,
        }
    }

    /// Returns whether cancellation has been requested.
    fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Relaxed)
    }

    /// Returns the raw atomic flag for proof-engine cancellation checks.
    fn flag(&self) -> &AtomicBool {
        &self.requested
    }

    /// Updates the next problem that would be started if execution continues.
    fn set_next_problem(&self, next_problem: Option<String>) {
        if let Ok(mut guard) = self.next_problem.lock() {
            *guard = next_problem;
        }
    }
}

/// Running counts and metadata for `prove` batch execution.
#[derive(Default)]
struct ProveBatchSummary {
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

/// Running counts and metadata for `rules` batch execution.
#[derive(Default)]
struct RulesBatchSummary {
    processed: usize,
    skipped: usize,
    succeeded: usize,
    failed: usize,
    rule_matches: usize,
    cancelled: bool,
    failed_files: Vec<PathBuf>,
}

/// Dispatches the `prove` command across direct targets or configured subset
/// runs.
pub(crate) fn run_prover_mode(options: &ProveCommand) {
    let cancellation = CancellationState::install();
    if let Some(target) = &options.target {
        let target = Path::new(target);
        if target.is_dir() {
            prove_directory(target, options, &cancellation);
        } else {
            print_prove_preamble(options.format, None);
            let result = prove_file(
                &ProblemRun {
                    path: target.to_path_buf(),
                    subset_stats: None,
                },
                options,
                &cancellation,
                1,
                1,
            );
            report_single_prove_file(result);
        }
        return;
    }

    let config = ensure_config();
    let targets = resolve_subset_targets(&config);
    print_prove_preamble(options.format, Some(targets.len()));
    prove_paths(&targets, options, &cancellation);
}

/// Dispatches the `rules` command across direct targets or configured subset
/// runs.
pub(crate) fn run_rules_mode(options: &RulesCommand) {
    let cancellation = CancellationState::install();
    if let Some(target) = &options.target {
        let target = Path::new(target);
        if target.is_dir() {
            inspect_rules_directory(target, options, &cancellation);
        } else {
            print_rules_preamble(options.format, None);
            let result = inspect_rules_file(
                &ProblemRun {
                    path: target.to_path_buf(),
                    subset_stats: None,
                },
                options,
                &cancellation,
                1,
                1,
            );
            report_single_file(result.success);
        }
        return;
    }

    let config = ensure_config();
    let targets = resolve_subset_targets(&config);
    print_rules_preamble(options.format, Some(targets.len()));
    inspect_rules_paths(&targets, options, &cancellation);
}

/// Runs the prover over every `.p` file in a directory and prints per-status totals.
fn prove_directory(dir: &Path, options: &ProveCommand, cancellation: &CancellationState) {
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

    print_prove_preamble(options.format, None);
    prove_paths(&problem_runs, options, cancellation);
}

/// Processes many problems through the prover and emits aggregate results.
fn prove_paths(problem_runs: &[ProblemRun], options: &ProveCommand, cancellation: &CancellationState) {
    let mut summary = ProveBatchSummary::default();
    let total = problem_runs.len();
    for (index, problem_run) in problem_runs.iter().enumerate() {
        cancellation.set_next_problem(Some(format!(
            "[{}/{}] {} ({})",
            index + 1,
            total,
            problem_run.problem_id(),
            problem_run.path.display()
        )));
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
    cancellation.set_next_problem(None);

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
                summary.interrupted_problem.unwrap_or_default()
            );
        }
    }

    if options.format == OutputFormat::Human && !summary.failed_files.is_empty() {
        eprintln!("Failed files:");
        for path in summary.failed_files {
            eprintln!("  {}", path.display());
        }
    }

    if summary.failed_to_process > 0 || summary.cancelled > 0 || cancellation.is_requested() {
        std::process::exit(1);
    }
}

/// Runs the prover for one file and returns either a proof status or a
/// processing failure.
fn prove_file(
    problem_run: &ProblemRun,
    options: &ProveCommand,
    cancellation: &CancellationState,
    current: usize,
    total: usize,
) -> ProveFileResult {
    let input = fs::read_to_string(&problem_run.path).expect("Failed to read input file");
    let proof_options = prover_options_from_cli(options);
    let started_at = Instant::now();
    let problem_id = problem_run.problem_id();
    let (formulae, atoms) = subset_stats_fields(problem_run.subset_stats);

    match run_problem_verbose_with_options_and_cancel(
        &input,
        options.display.show_sequent,
        proof_options,
        cancellation.flag(),
    ) {
        Ok(result) => {
            clear_parse_failure_marker(&problem_run.path);
            let elapsed_ms = started_at.elapsed().as_millis();
            let status = result.status;
            match options.format {
                OutputFormat::Human => print_prove_human_row(
                    current,
                    total,
                    &problem_id,
                    human_proof_status(&status),
                    elapsed_ms,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => println!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\t{:?}\t{elapsed_ms}",
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

/// Runs rule inspection over every `.p` file in a directory and prints aggregate counts.
fn inspect_rules_directory(dir: &Path, options: &RulesCommand, cancellation: &CancellationState) {
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

    print_rules_preamble(options.format, None);
    inspect_rules_paths(&problem_runs, options, cancellation);
}

/// Processes many problems through the rule matcher and emits aggregate results.
fn inspect_rules_paths(
    problem_runs: &[ProblemRun],
    options: &RulesCommand,
    cancellation: &CancellationState,
) {
    let mut summary = RulesBatchSummary::default();
    let total = problem_runs.len();
    for (index, problem_run) in problem_runs.iter().enumerate() {
        cancellation.set_next_problem(Some(format!(
            "[{}/{}] {} ({})",
            index + 1,
            total,
            problem_run.problem_id(),
            problem_run.path.display()
        )));
        if cancellation.is_requested() {
            summary.cancelled = true;
            break;
        }

        if should_skip_parse_failed_file(&problem_run.path, options) {
            summary.skipped += 1;
            continue;
        }

        summary.processed += 1;
        let inspection = inspect_rules_file(problem_run, options, cancellation, index + 1, total);
        if inspection.had_rule_match {
            summary.rule_matches += 1;
        }
        if inspection.success {
            summary.succeeded += 1;
        } else {
            summary.failed += 1;
            summary.failed_files.push(problem_run.path.clone());
        }
    }
    cancellation.set_next_problem(None);

    match options.format {
        OutputFormat::Human => {
            print_summary_header("summary");
            print_summary_row(&[
                ("processed", summary.processed.to_string()),
                ("skipped", summary.skipped.to_string()),
                ("succeeded", summary.succeeded.to_string()),
                ("failed", summary.failed.to_string()),
                ("rule_matches", summary.rule_matches.to_string()),
                ("cancelled", yes_no(summary.cancelled).to_string()),
            ]);
            if summary.cancelled {
                eprintln!("Cancelled before starting the next problem");
            }
        }
        OutputFormat::Tsv => {
            println!(
                "summary\t{}\t{}\t{}\t{}\t{}\t{}",
                summary.processed,
                summary.skipped,
                summary.succeeded,
                summary.failed,
                summary.rule_matches,
                summary.cancelled
            );
        }
    }

    if options.format == OutputFormat::Human && !summary.failed_files.is_empty() {
        eprintln!("Failed files:");
        for path in summary.failed_files {
            eprintln!("  {}", path.display());
        }
    }

    if summary.failed > 0 || summary.cancelled {
        std::process::exit(1);
    }
}

/// Runs rule inspection on one file and reports whether parsing/building succeeded.
fn inspect_rules_file(
    problem_run: &ProblemRun,
    options: &RulesCommand,
    _cancellation: &CancellationState,
    current: usize,
    total: usize,
) -> RulesInspectionResult {
    let input = fs::read_to_string(&problem_run.path).expect("Failed to read input file");
    let problem_id = problem_run.problem_id();
    let (formulae, atoms) = subset_stats_fields(problem_run.subset_stats);

    match build_problem_sequent(&input) {
        Ok(sequent) => {
            clear_parse_failure_marker(&problem_run.path);
            let matches = find_applicable_rules(&sequent);
            match options.format {
                OutputFormat::Human => print_rules_human_row(
                    current,
                    total,
                    &problem_id,
                    true,
                    !matches.is_empty(),
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => println!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\t{}\t{}",
                    problem_run.path.display(),
                    true,
                    !matches.is_empty()
                ),
            }
            if options.format == OutputFormat::Human && options.display.show_sequent {
                println!("  {sequent}");
            }
            if options.format == OutputFormat::Human {
                if matches.is_empty() {
                    println!("  no applicable rules");
                } else {
                    for rule_match in &matches {
                        println!("  {}", format_rule_match(*rule_match));
                    }
                }
            }
            RulesInspectionResult {
                success: true,
                had_rule_match: !matches.is_empty(),
            }
        }
        Err(ProblemPipelineError::Parse(err)) => {
            write_parse_failure_marker(&problem_run.path, &err);
            match options.format {
                OutputFormat::Human => print_rules_human_row(
                    current,
                    total,
                    &problem_id,
                    false,
                    false,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => eprintln!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\tfalse\tfalse",
                    problem_run.path.display()
                ),
            }
            eprintln!("{err}");
            RulesInspectionResult {
                success: false,
                had_rule_match: false,
            }
        }
        Err(ProblemPipelineError::SequentBuild(err)) => {
            match options.format {
                OutputFormat::Human => print_rules_human_row(
                    current,
                    total,
                    &problem_id,
                    false,
                    false,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => eprintln!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\tfalse\tfalse",
                    problem_run.path.display()
                ),
            }
            eprintln!("sequent construction failed: {err:?}");
            RulesInspectionResult {
                success: false,
                had_rule_match: false,
            }
        }
    }
}

/// Formats a matched rule occurrence for CLI output.
fn format_rule_match(rule_match: RuleMatch) -> String {
    format!(
        "{:?} on {:?}[{}]",
        rule_match.rule, rule_match.side, rule_match.index
    )
}

/// Prints single-file rule-inspection status and exits non-zero on failure.
fn report_single_file(success: bool) {
    if !success {
        std::process::exit(1);
    }
}

/// Prints single-file prover status and exits non-zero on processing failure.
fn report_single_prove_file(result: ProveFileResult) {
    match result {
        ProveFileResult::Status(ProofStatus::Cancelled) => {
            std::process::exit(1);
        }
        ProveFileResult::Status(_) => {}
        ProveFileResult::ProcessingFailure => {
            std::process::exit(1);
        }
    }
}

/// Returns the path of the `.parse_failed` marker associated with a `.p` file.
fn parse_failure_marker_path(path: &Path) -> Option<PathBuf> {
    (path.extension().and_then(|ext| ext.to_str()) == Some("p"))
        .then(|| PathBuf::from(format!("{}.parse_failed", path.display())))
}

/// Writes a `.parse_failed` marker alongside an input file after parse failure.
fn write_parse_failure_marker(path: &Path, err: &str) {
    let Some(marker_path) = parse_failure_marker_path(path) else {
        return;
    };

    let contents = format!("{}\nparse failed\n{err}\n", path.display());
    if let Err(write_err) = fs::write(&marker_path, contents) {
        eprintln!(
            "{}: failed to write parse-failure marker {}",
            path.display(),
            marker_path.display()
        );
        eprintln!("{write_err}");
    }
}

/// Removes any stale `.parse_failed` marker for a file after successful processing.
fn clear_parse_failure_marker(path: &Path) {
    let Some(marker_path) = parse_failure_marker_path(path) else {
        return;
    };

    match fs::remove_file(&marker_path) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => {
            eprintln!(
                "{}: failed to remove parse-failure marker {}",
                path.display(),
                marker_path.display()
            );
            eprintln!("{err}");
        }
    }
}

/// Returns whether a file should be skipped because it already has a
/// parse-failure marker and retry was not requested.
fn should_skip_parse_failed_file(path: &Path, options: &impl ParseFailureOptions) -> bool {
    !options.retry_parse_failed()
        && parse_failure_marker_path(path).is_some_and(|marker_path| marker_path.exists())
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
