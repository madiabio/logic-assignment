use clap::{Args, Parser, Subcommand};
use env_logger::Target;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use theorem_prover::proof::rules::{RuleMatch, find_applicable_rules};
use theorem_prover::{
    ProblemPipelineError, ProofOptions, ProofStatus, build_problem_sequent,
    run_problem_verbose_with_options,
};

#[derive(Clone, Args)]
struct SharedRunOptions {
    /// Reprocess files that already have a `.parse_failed` marker.
    #[arg(long)]
    retry_parse_failed: bool,
}

#[derive(Clone, Args)]
struct SharedDisplayOptions {
    /// Print the constructed sequent before running the selected command.
    #[arg(long)]
    show_sequent: bool,
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Theorem prover CLI.\n\nUse `prove` to run proof search with configurable timeout, depth, and step limits.\nUse `rules` to inspect which sequent-calculus rules apply to a problem."
)]
struct CliOptions {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the prover on a file or directory of `.p` problems.
    Prove(ProveCommand),
    /// Show which rules apply to a file or directory of `.p` problems.
    Rules(RulesCommand),
}

#[derive(Clone, Args)]
struct ProveCommand {
    #[command(flatten)]
    run: SharedRunOptions,
    #[command(flatten)]
    display: SharedDisplayOptions,
    /// Wall-clock timeout in milliseconds.
    #[arg(long)]
    timeout_ms: Option<u64>,
    /// Maximum recursive proof-search depth before returning `Unknown`.
    #[arg(long)]
    max_depth: Option<usize>,
    /// Maximum proof-search steps before returning `Unknown`.
    #[arg(long)]
    max_steps: Option<usize>,
    /// Input `.p` file or directory of `.p` files to prove.
    target: String,
}

#[derive(Clone, Args)]
struct RulesCommand {
    #[command(flatten)]
    run: SharedRunOptions,
    #[command(flatten)]
    display: SharedDisplayOptions,
    /// Input `.p` file or directory of `.p` files to inspect.
    target: String,
}

#[derive(Clone, Copy)]
/// Outcome of running rule inspection on one file.
struct RulesInspectionResult {
    success: bool,
    had_rule_match: bool,
}

#[derive(Clone)]
/// Result of running the prover on one file.
enum ProveFileResult {
    /// The file was processed successfully and produced a proof status.
    Status(ProofStatus),
    /// The file could not be processed because parsing or sequent building failed.
    ProcessingFailure,
}

fn main() {
    // init a logger
    env_logger::Builder::new().target(Target::Stdout).init();

    match CliOptions::parse().command {
        Command::Prove(options) => run_prover_mode(Path::new(&options.target), &options),
        Command::Rules(options) => run_rules_mode(Path::new(&options.target), &options),
    }
}

/// Dispatches the `prove` subcommand to either file or directory handling.
fn run_prover_mode(target: &Path, options: &ProveCommand) {
    if target.is_dir() {
        prove_directory(target, options);
    } else {
        let result = prove_file(target, options);
        report_single_prove_file(target, result);
    }
}

/// Dispatches the `rules` subcommand to either file or directory handling.
fn run_rules_mode(target: &Path, options: &RulesCommand) {
    if target.is_dir() {
        inspect_rules_directory(target, options);
    } else {
        let result = inspect_rules_file(target, options);
        report_single_file(target, result.success);
    }
}

/// Runs the prover over every `.p` file in a directory and prints per-status totals.
fn prove_directory(dir: &Path, options: &ProveCommand) {
    let mut processed = 0usize;
    let mut skipped = 0usize;
    let mut failed_files = Vec::new();
    let mut provable = 0usize;
    let mut not_provable = 0usize;
    let mut timeout = 0usize;
    let mut unknown = 0usize;
    let mut not_implemented = 0usize;
    let mut error = 0usize;
    let mut processing_failures = 0usize;

    let entries = fs::read_dir(dir).expect("Failed to read directory");
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("p") {
            continue;
        }

        if should_skip_parse_failed_file(&path, options) {
            skipped += 1;
            continue;
        }

        processed += 1;
        match prove_file(&path, options) {
            ProveFileResult::Status(ProofStatus::Provable) => provable += 1,
            ProveFileResult::Status(ProofStatus::NotProvable) => not_provable += 1,
            ProveFileResult::Status(ProofStatus::Timeout) => timeout += 1,
            ProveFileResult::Status(ProofStatus::Unknown) => unknown += 1,
            ProveFileResult::Status(ProofStatus::NotImplemented) => not_implemented += 1,
            ProveFileResult::Status(ProofStatus::Error) => error += 1,
            ProveFileResult::ProcessingFailure => {
                processing_failures += 1;
                failed_files.push(path);
            }
        }
    }

    println!("Processed {processed} file(s)");
    println!("Skipped: {skipped}");
    println!("Provable: {provable}");
    println!("Not provable: {not_provable}");
    println!("Timeout: {timeout}");
    println!("Unknown: {unknown}");
    println!("Not implemented: {not_implemented}");
    println!("Error: {error}");
    println!("Failed to process: {processing_failures}");

    if !failed_files.is_empty() {
        eprintln!("Failed files:");
        for path in failed_files {
            eprintln!("  {}", path.display());
        }
    }

    if processing_failures > 0 {
        std::process::exit(1);
    }
}

/// Runs the prover on one file and returns either a proof status or a processing failure.
fn prove_file(path: &Path, options: &ProveCommand) -> ProveFileResult {
    let input = fs::read_to_string(path).expect("Failed to read input file");
    let proof_options = prover_options_from_cli(options);

    match run_problem_verbose_with_options(&input, options.display.show_sequent, proof_options) {
        Ok(result) => {
            clear_parse_failure_marker(path);
            println!("{}: prover returned {:?}", path.display(), result.status);
            ProveFileResult::Status(result.status)
        }
        Err(ProblemPipelineError::Parse(err)) => {
            write_parse_failure_marker(path, &err);
            eprintln!("{}: parse failed", path.display());
            eprintln!("{err}");
            ProveFileResult::ProcessingFailure
        }
        Err(ProblemPipelineError::SequentBuild(err)) => {
            eprintln!("{}: sequent construction failed", path.display());
            eprintln!("{err:?}");
            ProveFileResult::ProcessingFailure
        }
    }
}

/// Runs rule inspection over every `.p` file in a directory and prints aggregate counts.
fn inspect_rules_directory(dir: &Path, options: &RulesCommand) {
    let mut failures = 0usize;
    let mut processed = 0usize;
    let mut skipped = 0usize;
    let mut matched_problems = 0usize;
    let mut failed_files = Vec::new();

    let entries = fs::read_dir(dir).expect("Failed to read directory");
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("p") {
            continue;
        }

        if should_skip_parse_failed_file(&path, options) {
            skipped += 1;
            continue;
        }

        processed += 1;
        let inspection = inspect_rules_file(&path, options);
        if inspection.had_rule_match {
            matched_problems += 1;
        }
        if !inspection.success {
            failures += 1;
            failed_files.push(path);
        }
    }

    println!("Processed {processed} file(s)");
    println!("Skipped: {skipped}");
    println!("Succeeded: {}", processed - failures);
    println!("Failed: {failures}");
    println!("Problems with rule matches: {matched_problems}");

    if !failed_files.is_empty() {
        eprintln!("Failed files:");
        for path in failed_files {
            eprintln!("  {}", path.display());
        }
    }

    if failures > 0 {
        std::process::exit(1);
    }
}

/// Runs rule inspection on one file and reports whether parsing/building succeeded.
fn inspect_rules_file(path: &Path, options: &RulesCommand) -> RulesInspectionResult {
    let input = fs::read_to_string(path).expect("Failed to read input file");

    match build_problem_sequent(&input) {
        Ok(sequent) => {
            clear_parse_failure_marker(path);
            println!("{}:", path.display());
            if options.display.show_sequent {
                println!("  {sequent}");
            }
            let matches = find_applicable_rules(&sequent);
            if matches.is_empty() {
                println!("  no applicable rules");
            } else {
                for rule_match in &matches {
                    println!("  {}", format_rule_match(*rule_match));
                }
            }
            RulesInspectionResult {
                success: true,
                had_rule_match: !matches.is_empty(),
            }
        }
        Err(ProblemPipelineError::Parse(err)) => {
            write_parse_failure_marker(path, &err);
            eprintln!("{}: parse failed", path.display());
            eprintln!("{err}");
            RulesInspectionResult {
                success: false,
                had_rule_match: false,
            }
        }
        Err(ProblemPipelineError::SequentBuild(err)) => {
            eprintln!("{}: sequent construction failed", path.display());
            eprintln!("{err:?}");
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
fn report_single_file(path: &Path, success: bool) {
    if success {
        println!("{}: pipeline completed", path.display());
    } else {
        std::process::exit(1);
    }
}

/// Prints single-file prover status and exits non-zero on processing failure.
fn report_single_prove_file(path: &Path, result: ProveFileResult) {
    match result {
        ProveFileResult::Status(_) => {
            println!("{}: pipeline completed", path.display());
        }
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

/// Returns whether a file should be skipped because it already has a parse-failure marker.
fn should_skip_parse_failed_file(path: &Path, options: &impl ParseFailureOptions) -> bool {
    !options.retry_parse_failed()
        && parse_failure_marker_path(path).is_some_and(|marker_path| marker_path.exists())
}

/// Shared access to the retry-parse-failed option across CLI subcommands.
trait ParseFailureOptions {
    fn retry_parse_failed(&self) -> bool;
}

impl ParseFailureOptions for ProveCommand {
    fn retry_parse_failed(&self) -> bool {
        self.run.retry_parse_failed
    }
}

impl ParseFailureOptions for RulesCommand {
    fn retry_parse_failed(&self) -> bool {
        self.run.retry_parse_failed
    }
}

/// Builds prover options by applying CLI overrides on top of the default search bounds.
fn prover_options_from_cli(options: &ProveCommand) -> ProofOptions {
    let mut proof_options = ProofOptions::default();
    if let Some(timeout_ms) = options.timeout_ms {
        proof_options.timeout = std::time::Duration::from_millis(timeout_ms);
    }
    if let Some(max_depth) = options.max_depth {
        proof_options.max_depth = max_depth;
    }
    if let Some(max_steps) = options.max_steps {
        proof_options.max_steps = max_steps;
    }
    proof_options
}
