use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use theorem_prover::proof::rules::{RuleMatch, find_applicable_rules};
use theorem_prover::{ProblemPipelineError, build_problem_sequent, run_problem_verbose};

#[derive(Clone, Copy)]
struct RunOptions {
    retry_parse_failed: bool,
    show_sequent: bool,
}

#[derive(Clone, Copy)]
enum Mode {
    Prover,
    Rules,
}

struct CliOptions {
    mode: Mode,
    target: String,
    run_options: RunOptions,
}

#[derive(Clone, Copy)]
struct RulesInspectionResult {
    success: bool,
    had_rule_match: bool,
}

fn main() {
    let Some(options) = parse_cli_args(env::args().skip(1)) else {
        print_usage_and_exit();
    };

    match options.mode {
        Mode::Prover => run_prover_mode(Path::new(&options.target), options.run_options),
        Mode::Rules => run_rules_mode(Path::new(&options.target), options.run_options),
    }
}

fn parse_cli_args(args: impl IntoIterator<Item = String>) -> Option<CliOptions> {
    let mut mode = Mode::Prover;
    let mut retry_parse_failed = false;
    let mut show_sequent = false;
    let mut target: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "--rules" => mode = Mode::Rules,
            "--retry-parse-failed" => retry_parse_failed = true,
            "--show-sequent" => show_sequent = true,
            _ if arg.starts_with("--") => return None,
            _ => {
                if target.replace(arg).is_some() {
                    return None;
                }
            }
        }
    }

    target.map(|target| CliOptions {
        mode,
        target,
        run_options: RunOptions {
            retry_parse_failed,
            show_sequent,
        },
    })
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: cargo run -- [--show-sequent] <file.tptp | directory>");
    eprintln!("   or: cargo run -- --retry-parse-failed [--show-sequent] <file.tptp | directory>");
    eprintln!("   or: cargo run -- --rules [--show-sequent] <file.p | directory>");
    eprintln!(
        "   or: cargo run -- --rules --retry-parse-failed [--show-sequent] <file.p | directory>"
    );
    std::process::exit(1);
}

fn run_prover_mode(target: &Path, options: RunOptions) {
    if target.is_dir() {
        prove_directory(target, options);
    } else {
        let result = prove_file(target, options);
        report_single_file(target, result);
    }
}

fn run_rules_mode(target: &Path, options: RunOptions) {
    if target.is_dir() {
        inspect_rules_directory(target, options);
    } else {
        let result = inspect_rules_file(target, options);
        report_single_file(target, result.success);
    }
}

fn prove_directory(dir: &Path, options: RunOptions) {
    let mut failures = 0usize;
    let mut processed = 0usize;
    let mut skipped = 0usize;
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
        if !prove_file(&path, options) {
            failures += 1;
            failed_files.push(path);
        }
    }

    println!("Processed {processed} file(s)");
    println!("Skipped: {skipped}");
    println!("Succeeded: {}", processed - failures);
    println!("Failed: {failures}");

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

fn prove_file(path: &Path, options: RunOptions) -> bool {
    let input = fs::read_to_string(path).expect("Failed to read input file");

    match run_problem_verbose(&input, options.show_sequent) {
        Ok(result) => {
            clear_parse_failure_marker(path);
            println!("{}: prover returned {:?}", path.display(), result.status);
            true
        }
        Err(ProblemPipelineError::Parse(err)) => {
            write_parse_failure_marker(path, &err);
            eprintln!("{}: parse failed", path.display());
            eprintln!("{err}");
            false
        }
        Err(ProblemPipelineError::SequentBuild(err)) => {
            eprintln!("{}: sequent construction failed", path.display());
            eprintln!("{err:?}");
            false
        }
    }
}

fn inspect_rules_directory(dir: &Path, options: RunOptions) {
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

fn inspect_rules_file(path: &Path, options: RunOptions) -> RulesInspectionResult {
    let input = fs::read_to_string(path).expect("Failed to read input file");

    match build_problem_sequent(&input) {
        Ok(sequent) => {
            clear_parse_failure_marker(path);
            println!("{}:", path.display());
            if options.show_sequent {
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

fn format_rule_match(rule_match: RuleMatch) -> String {
    format!(
        "{:?} on {:?}[{}]",
        rule_match.rule, rule_match.side, rule_match.index
    )
}

fn report_single_file(path: &Path, success: bool) {
    if success {
        println!("{}: pipeline completed", path.display());
    } else {
        std::process::exit(1);
    }
}

fn parse_failure_marker_path(path: &Path) -> Option<PathBuf> {
    (path.extension().and_then(|ext| ext.to_str()) == Some("p"))
        .then(|| PathBuf::from(format!("{}.parse_failed", path.display())))
}

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

fn should_skip_parse_failed_file(path: &Path, options: RunOptions) -> bool {
    !options.retry_parse_failed
        && parse_failure_marker_path(path).is_some_and(|marker_path| marker_path.exists())
}
