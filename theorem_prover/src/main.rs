use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use theorem_prover::proof::rules::{RuleMatch, find_applicable_rules};
use theorem_prover::{ProblemPipelineError, build_problem_sequent, run_problem};

#[derive(Clone, Copy)]
struct RunOptions {
    retry_parse_failed: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_, target] => run_prover_mode(Path::new(target), RunOptions { retry_parse_failed: false }),
        [_, flag, target] if flag == "--rules" => {
            run_rules_mode(Path::new(target), RunOptions { retry_parse_failed: false })
        }
        [_, flag, target] if flag == "--retry-parse-failed" => {
            run_prover_mode(Path::new(target), RunOptions { retry_parse_failed: true })
        }
        [_, flag1, flag2, target]
            if flag1 == "--rules" && flag2 == "--retry-parse-failed" =>
        {
            run_rules_mode(Path::new(target), RunOptions { retry_parse_failed: true })
        }
        _ => {
            eprintln!("Usage: cargo run -- <file.tptp | directory>");
            eprintln!("   or: cargo run -- --retry-parse-failed <file.tptp | directory>");
            eprintln!("   or: cargo run -- --rules <file.p | directory>");
            eprintln!("   or: cargo run -- --rules --retry-parse-failed <file.p | directory>");
            std::process::exit(1);
        }
    }
}

fn run_prover_mode(target: &Path, options: RunOptions) {
    if target.is_dir() {
        prove_directory(target, options);
    } else {
        let result = prove_file(target);
        report_single_file(target, result);
    }
}

fn run_rules_mode(target: &Path, options: RunOptions) {
    if target.is_dir() {
        inspect_rules_directory(target, options);
    } else {
        let result = inspect_rules_file(target);
        report_single_file(target, result);
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
        if !prove_file(&path) {
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

fn prove_file(path: &Path) -> bool {
    let input = fs::read_to_string(path).expect("Failed to read input file");

    match run_problem(&input) {
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
        if !inspect_rules_file(&path) {
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

fn inspect_rules_file(path: &Path) -> bool {
    let input = fs::read_to_string(path).expect("Failed to read input file");

    match build_problem_sequent(&input) {
        Ok(sequent) => {
            clear_parse_failure_marker(path);
            println!("{}:", path.display());
            println!("  {sequent}");
            let matches = find_applicable_rules(&sequent);
            if matches.is_empty() {
                println!("  no applicable rules");
            } else {
                for rule_match in matches {
                    println!("  {}", format_rule_match(rule_match));
                }
            }
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
        && parse_failure_marker_path(path)
            .is_some_and(|marker_path| marker_path.exists())
}
