use clap::{Args, Parser, Subcommand, ValueEnum};
use env_logger::Target;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Human,
    Tsv,
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
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,
    /// Input `.p` file or directory of `.p` files to prove.
    target: Option<String>,
}

#[derive(Clone, Args)]
struct RulesCommand {
    #[command(flatten)]
    run: SharedRunOptions,
    #[command(flatten)]
    display: SharedDisplayOptions,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,
    /// Input `.p` file or directory of `.p` files to inspect.
    target: Option<String>,
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
        Command::Prove(options) => run_prover_mode(&options),
        Command::Rules(options) => run_rules_mode(&options),
    }
}

/// Dispatches the `prove` subcommand to either file or directory handling.
fn run_prover_mode(options: &ProveCommand) {
    if let Some(target) = &options.target {
        let target = Path::new(target);
        if target.is_dir() {
            prove_directory(target, options);
        } else {
            match options.format {
                OutputFormat::Human => print_prove_human_header(),
                OutputFormat::Tsv => {
                    println!(
                        "kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tstatus\telapsed_ms"
                    )
                }
            }
            let result = prove_file(
                &ProblemRun {
                    path: target.to_path_buf(),
                    subset_stats: None,
                },
                options,
                1,
                1,
            );
            report_single_prove_file(result);
        }
        return;
    }

    let config = ensure_config();
    let targets = resolve_subset_targets(&config);
    if options.format == OutputFormat::Human {
        println!("Loaded {} problem(s) from subset", targets.len());
    } else {
        println!("kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tstatus\telapsed_ms");
    }
    prove_paths(&targets, options);
}

/// Dispatches the `rules` subcommand to either file or directory handling.
fn run_rules_mode(options: &RulesCommand) {
    if let Some(target) = &options.target {
        let target = Path::new(target);
        if target.is_dir() {
            inspect_rules_directory(target, options);
        } else {
            match options.format {
                OutputFormat::Human => print_rules_human_header(),
                OutputFormat::Tsv => println!(
                    "kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tsuccess\thad_rule_match"
                ),
            }
            let result = inspect_rules_file(
                &ProblemRun {
                    path: target.to_path_buf(),
                    subset_stats: None,
                },
                options,
                1,
                1,
            );
            report_single_file(result.success);
        }
        return;
    }

    let config = ensure_config();
    let targets = resolve_subset_targets(&config);
    if options.format == OutputFormat::Human {
        println!("Loaded {} problem(s) from subset", targets.len());
    } else {
        println!("kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tsuccess\thad_rule_match");
    }
    inspect_rules_paths(&targets, options);
}

/// Runs the prover over every `.p` file in a directory and prints per-status totals.
fn prove_directory(dir: &Path, options: &ProveCommand) {
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

    prove_paths(&problem_runs, options);
}

fn prove_paths(problem_runs: &[ProblemRun], options: &ProveCommand) {
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

    let total = problem_runs.len();
    if options.format == OutputFormat::Human {
        print_prove_human_header();
    }
    for (index, problem_run) in problem_runs.iter().enumerate() {
        if should_skip_parse_failed_file(&problem_run.path, options) {
            skipped += 1;
            continue;
        }

        processed += 1;
        match prove_file(problem_run, options, index + 1, total) {
            ProveFileResult::Status(ProofStatus::Provable) => provable += 1,
            ProveFileResult::Status(ProofStatus::NotProvable) => not_provable += 1,
            ProveFileResult::Status(ProofStatus::Timeout) => timeout += 1,
            ProveFileResult::Status(ProofStatus::Unknown) => unknown += 1,
            ProveFileResult::Status(ProofStatus::NotImplemented) => not_implemented += 1,
            ProveFileResult::Status(ProofStatus::Error) => error += 1,
            ProveFileResult::ProcessingFailure => {
                processing_failures += 1;
                failed_files.push(problem_run.path.clone());
            }
        }
    }

    match options.format {
        OutputFormat::Human => {
            print_summary_header("summary");
            print_summary_row(&[
                ("processed", processed.to_string()),
                ("skipped", skipped.to_string()),
                ("provable", provable.to_string()),
                ("not_provable", not_provable.to_string()),
                ("timeout", timeout.to_string()),
                ("unknown", unknown.to_string()),
                ("not_impl", not_implemented.to_string()),
                ("error", error.to_string()),
                ("failed_to_process", processing_failures.to_string()),
            ]);
        }
        OutputFormat::Tsv => {
            println!(
                "summary\t{processed}\t{skipped}\t{provable}\t{not_provable}\t{timeout}\t{unknown}\t{not_implemented}\t{error}\t{processing_failures}"
            );
        }
    }

    if options.format == OutputFormat::Human && !failed_files.is_empty() {
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
fn prove_file(
    problem_run: &ProblemRun,
    options: &ProveCommand,
    current: usize,
    total: usize,
) -> ProveFileResult {
    let input = fs::read_to_string(&problem_run.path).expect("Failed to read input file");
    let proof_options = prover_options_from_cli(options);
    let started_at = Instant::now();
    let problem_id = problem_run.problem_id();
    let (formulae, atoms) = subset_stats_fields(problem_run.subset_stats);

    match run_problem_verbose_with_options(&input, options.display.show_sequent, proof_options) {
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
fn inspect_rules_directory(dir: &Path, options: &RulesCommand) {
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

    inspect_rules_paths(&problem_runs, options);
}

fn inspect_rules_paths(problem_runs: &[ProblemRun], options: &RulesCommand) {
    let mut failures = 0usize;
    let mut processed = 0usize;
    let mut skipped = 0usize;
    let mut matched_problems = 0usize;
    let mut failed_files = Vec::new();

    let total = problem_runs.len();
    if options.format == OutputFormat::Human {
        print_rules_human_header();
    }
    for (index, problem_run) in problem_runs.iter().enumerate() {
        if should_skip_parse_failed_file(&problem_run.path, options) {
            skipped += 1;
            continue;
        }

        processed += 1;
        let inspection = inspect_rules_file(problem_run, options, index + 1, total);
        if inspection.had_rule_match {
            matched_problems += 1;
        }
        if !inspection.success {
            failures += 1;
            failed_files.push(problem_run.path.clone());
        }
    }

    match options.format {
        OutputFormat::Human => {
            print_summary_header("summary");
            print_summary_row(&[
                ("processed", processed.to_string()),
                ("skipped", skipped.to_string()),
                ("succeeded", (processed - failures).to_string()),
                ("failed", failures.to_string()),
                ("rule_matches", matched_problems.to_string()),
            ]);
        }
        OutputFormat::Tsv => {
            println!(
                "summary\t{processed}\t{skipped}\t{}\t{failures}\t{matched_problems}",
                processed - failures
            );
        }
    }

    if options.format == OutputFormat::Human && !failed_files.is_empty() {
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
fn inspect_rules_file(
    problem_run: &ProblemRun,
    options: &RulesCommand,
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
    if let Some(config) = load_config_if_present() {
        if let Some(timeout_ms) = config.timeout_ms {
            proof_options.timeout = std::time::Duration::from_millis(timeout_ms);
        }
        if let Some(max_depth) = config.max_depth {
            proof_options.max_depth = max_depth;
        }
        if let Some(max_steps) = config.max_steps {
            proof_options.max_steps = max_steps;
        }
    }

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

fn load_config_if_present() -> Option<AppConfig> {
    load_config().ok()
}

fn ensure_config() -> AppConfig {
    load_config().unwrap_or_else(|_| prompt_for_config())
}

fn prompt_for_config() -> AppConfig {
    println!("No usable config.toml found. Enter values to create one.");

    let config = AppConfig {
        tptp_root: PathBuf::from(prompt("TPTP root path")),
        default_subset_file: PathBuf::from(prompt("Default subset file path")),
        timeout_ms: Some(
            prompt("Default timeout in milliseconds")
                .parse::<u64>()
                .expect("timeout_ms must be an integer"),
        ),
        max_depth: Some(
            prompt("Default max depth")
                .parse::<usize>()
                .expect("max_depth must be an integer"),
        ),
        max_steps: Some(
            prompt("Default max steps")
                .parse::<usize>()
                .expect("max_steps must be an integer"),
        ),
    };

    write_config(&config).expect("failed to write config.toml");
    config
}

fn prompt(label: &str) -> String {
    print!("{label}: ");
    io::stdout().flush().expect("stdout should flush");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("stdin should provide a value");
    input.trim().to_string()
}

fn write_config(config: &AppConfig) -> Result<(), String> {
    let default_options = ProofOptions::default();
    let contents = format!(
        "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = {}\nmax_depth = {}\nmax_steps = {}\n",
        config.tptp_root.display(),
        config.default_subset_file.display(),
        config
            .timeout_ms
            .unwrap_or(default_options.timeout.as_millis() as u64),
        config.max_depth.unwrap_or(default_options.max_depth),
        config.max_steps.unwrap_or(default_options.max_steps),
    );

    fs::write("config.toml", contents).map_err(|err| format!("failed to write config.toml: {err}"))
}

struct AppConfig {
    tptp_root: PathBuf,
    default_subset_file: PathBuf,
    timeout_ms: Option<u64>,
    max_depth: Option<usize>,
    max_steps: Option<usize>,
}

#[derive(Clone)]
struct ProblemRun {
    path: PathBuf,
    subset_stats: Option<SubsetStats>,
}

impl ProblemRun {
    fn problem_id(&self) -> String {
        self.path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_string()
    }

    fn human_formulae(&self) -> String {
        self.subset_stats
            .map(|stats| stats.formulae.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    fn human_atoms(&self) -> String {
        self.subset_stats
            .map(|stats| stats.atoms.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

#[derive(Clone, Copy)]
struct SubsetStats {
    formulae: usize,
    atoms: usize,
}

fn load_config() -> Result<AppConfig, String> {
    let config_path = Path::new("config.toml");
    let config_contents = fs::read_to_string(config_path)
        .map_err(|err| format!("failed to read {}: {err}", config_path.display()))?;

    let mut tptp_root = None;
    let mut default_subset_file = None;
    let mut timeout_ms = None;
    let mut max_depth = None;
    let mut max_steps = None;

    for raw_line in config_contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');

        match key {
            "tptp_root" => tptp_root = Some(PathBuf::from(value)),
            "default_subset_file" => default_subset_file = Some(PathBuf::from(value)),
            "timeout_ms" => {
                timeout_ms = Some(
                    value
                        .parse::<u64>()
                        .map_err(|err| format!("invalid timeout_ms in config.toml: {err}"))?,
                )
            }
            "max_depth" => {
                max_depth = Some(
                    value
                        .parse::<usize>()
                        .map_err(|err| format!("invalid max_depth in config.toml: {err}"))?,
                )
            }
            "max_steps" => {
                max_steps = Some(
                    value
                        .parse::<usize>()
                        .map_err(|err| format!("invalid max_steps in config.toml: {err}"))?,
                )
            }
            _ => {}
        }
    }

    Ok(AppConfig {
        tptp_root: tptp_root.ok_or_else(|| "config.toml is missing tptp_root".to_string())?,
        default_subset_file: default_subset_file
            .ok_or_else(|| "config.toml is missing default_subset_file".to_string())?,
        timeout_ms,
        max_depth,
        max_steps,
    })
}

fn resolve_subset_targets(config: &AppConfig) -> Vec<ProblemRun> {
    let subset_contents = fs::read_to_string(&config.default_subset_file).unwrap_or_else(|err| {
        panic!(
            "failed to read subset file {}: {err}",
            config.default_subset_file.display()
        )
    });

    subset_contents
        .lines()
        .filter_map(parse_subset_problem_line)
        .map(|(problem_id, subset_stats)| ProblemRun {
            path: resolve_tptp_problem_path(&config.tptp_root, &problem_id),
            subset_stats,
        })
        .collect()
}

fn parse_subset_problem_line(line: &str) -> Option<(String, Option<SubsetStats>)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('%') {
        return None;
    }

    let tokens: Vec<_> = trimmed.split_whitespace().collect();
    let problem_id = *tokens.first()?;
    if !problem_id.contains('+') {
        return None;
    }

    let subset_stats = match (tokens.get(5), tokens.get(8)) {
        (Some(formulae), Some(atoms)) => {
            match (formulae.parse::<usize>(), atoms.parse::<usize>()) {
                (Ok(formulae), Ok(atoms)) => Some(SubsetStats { formulae, atoms }),
                _ => None,
            }
        }
        _ => None,
    };

    Some((problem_id.to_string(), subset_stats))
}

fn resolve_tptp_problem_path(tptp_root: &Path, problem_id: &str) -> PathBuf {
    let domain = &problem_id[..3];
    let problems_dir = tptp_root.join("Problems").join(domain);
    let exact_path = problems_dir.join(format!("{problem_id}.p"));
    if exact_path.exists() {
        return exact_path;
    }

    let base_problem_id = problem_id.split('.').next().unwrap_or(problem_id);
    problems_dir.join(format!("{base_problem_id}.p"))
}

fn subset_stats_fields(stats: Option<SubsetStats>) -> (usize, usize) {
    stats
        .map(|stats| (stats.formulae, stats.atoms))
        .unwrap_or((0, 0))
}

fn print_prove_human_header() {
    println!(
        "{:<8}  {:<16}  {:<20}  {:>8}  {:>5}  {:>5}  path",
        "idx", "problem", "status", "time_ms", "frm", "atoms"
    );
}

fn print_rules_human_header() {
    println!(
        "{:<8}  {:<16}  {:<3}  {:<5}  {:>5}  {:>5}  path",
        "idx", "problem", "ok", "match", "frm", "atoms"
    );
}

fn print_prove_human_row(
    current: usize,
    total: usize,
    problem_id: &str,
    status: &str,
    elapsed_ms: u128,
    formulae: String,
    atoms: String,
    path: &Path,
) {
    println!(
        "{:<8}  {:<16}  {:<20}  {:>8}  {:>5}  {:>5}  {}",
        format!("{current}/{total}"),
        problem_id,
        status,
        elapsed_ms,
        formulae,
        atoms,
        path.display()
    );
}

fn print_rules_human_row(
    current: usize,
    total: usize,
    problem_id: &str,
    success: bool,
    had_rule_match: bool,
    formulae: String,
    atoms: String,
    path: &Path,
) {
    println!(
        "{:<8}  {:<16}  {:<3}  {:<5}  {:>5}  {:>5}  {}",
        format!("{current}/{total}"),
        problem_id,
        yes_no(success),
        yes_no(had_rule_match),
        formulae,
        atoms,
        path.display()
    );
}

fn print_summary_header(title: &str) {
    println!();
    println!("{title}");
}

fn print_summary_row(values: &[(&str, String)]) {
    let labels = values
        .iter()
        .map(|(label, _)| format!("{:<17}", label))
        .collect::<Vec<_>>()
        .join(" ");
    let row = values
        .iter()
        .map(|(_, value)| format!("{:<17}", value))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{labels}");
    println!("{row}");
}

fn human_proof_status(status: &ProofStatus) -> &'static str {
    match status {
        ProofStatus::NotImplemented => "not_implemented",
        ProofStatus::Provable => "provable",
        ProofStatus::NotProvable => "not_provable",
        ProofStatus::Timeout => "timeout",
        ProofStatus::Unknown => "unknown",
        ProofStatus::Error => "error",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
