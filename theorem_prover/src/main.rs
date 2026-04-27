use std::env;
use std::fs;
use std::path::Path;
use theorem_prover::{Sequent, parse_problem};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <file.tptp | directory>");
        std::process::exit(1);
    }

    let target = Path::new(&args[1]);

    if target.is_dir() {
        parse_directory(target);
    } else {
        let result = build_initial_sequent(target);
        report_single_file(target, result);
    }
}

fn parse_directory(dir: &Path) {
    let mut failures = 0usize;
    let mut parsed = 0usize;
    let mut failed_files = Vec::new();

    let entries = fs::read_dir(dir).expect("Failed to read directory");
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("p") {
            continue;
        }

        parsed += 1;
        if !build_initial_sequent(&path) {
            failures += 1;
            failed_files.push(path);
        }
    }

    println!("Parsed {parsed} file(s)");
    println!("Succeeded: {}", parsed - failures);
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

fn build_initial_sequent(path: &Path) -> bool {
    let input = fs::read_to_string(path).expect("Failed to read input file");

    match parse_problem(&input) {
        Ok(parsed) => match Sequent::from_parsed_problem(parsed) {
            Ok(_) => true,
            Err(err) => {
                eprintln!("{}: sequent construction failed", path.display());
                eprintln!("{err:?}");
                false
            }
        },
        Err(e) => {
            eprintln!("{}: parse failed", path.display());
            eprintln!("{e}");
            false
        }
    }
}

fn report_single_file(path: &Path, success: bool) {
    if success {
        println!("{}: initial sequent built successfully", path.display());
    } else {
        std::process::exit(1);
    }
}
