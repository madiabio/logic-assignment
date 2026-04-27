use std::env;
use std::fs;
use std::path::Path;
use theorem_prover::{ProblemPipelineError, run_problem};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <file.tptp | directory>");
        std::process::exit(1);
    }

    let target = Path::new(&args[1]);

    if target.is_dir() {
        prove_directory(target);
    } else {
        let result = prove_file(target);
        report_single_file(target, result);
    }
}

fn prove_directory(dir: &Path) {
    let mut failures = 0usize;
    let mut processed = 0usize;
    let mut failed_files = Vec::new();

    let entries = fs::read_dir(dir).expect("Failed to read directory");
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("p") {
            continue;
        }

        processed += 1;
        if !prove_file(&path) {
            failures += 1;
            failed_files.push(path);
        }
    }

    println!("Processed {processed} file(s)");
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
            println!("{}: prover returned {:?}", path.display(), result.status);
            true
        }
        Err(ProblemPipelineError::Parse(err)) => {
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

fn report_single_file(path: &Path, success: bool) {
    if success {
        println!("{}: pipeline completed", path.display());
    } else {
        std::process::exit(1);
    }
}
