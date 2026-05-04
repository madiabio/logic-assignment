use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use theorem_prover::{
    ProblemPipelineError, ProofStatus, build_problem_sequent_from_path, run_problem_from_path,
};

fn make_temp_dir(test_name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("theorem_prover_{test_name}_{unique}"));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dirs should be created");
    }
    fs::write(path, contents).expect("file should be written");
}

#[test]
fn build_problem_sequent_from_path_loads_plain_includes_into_premises() {
    let root = make_temp_dir("include_loader_builds_sequent");
    let problem_path = root.join("Problems").join("GEO").join("GEO171+2.p");
    write_file(
        &problem_path,
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );
    write_file(
        &root.join("Axioms").join("GEO008+0.ax"),
        r#"
fof(ax_1,axiom,p).
"#,
    );

    let sequent = build_problem_sequent_from_path(&problem_path)
        .expect("loader should merge included axioms into the initial sequent");

    assert_eq!(sequent.left.len(), 1);
    assert_eq!(sequent.right.len(), 1);
}

#[test]
fn run_problem_from_path_proves_problem_using_included_axioms() {
    let root = make_temp_dir("include_loader_proves_problem");
    let problem_path = root.join("Problems").join("GEO").join("GEO171+2.p");
    write_file(
        &problem_path,
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );
    write_file(
        &root.join("Axioms").join("GEO008+0.ax"),
        r#"
fof(ax_1,axiom,p).
"#,
    );

    let result =
        run_problem_from_path(&problem_path).expect("loader-backed pipeline should succeed");

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn build_problem_sequent_from_path_deduplicates_repeated_includes() {
    let root = make_temp_dir("include_loader_deduplicates");
    let problem_path = root.join("Problems").join("GEO").join("GEO171+2.p");
    write_file(
        &problem_path,
        r#"
include('Axioms/GEO008+0.ax').
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );
    write_file(
        &root.join("Axioms").join("GEO008+0.ax"),
        r#"
fof(ax_1,axiom,p).
"#,
    );

    let sequent = build_problem_sequent_from_path(&problem_path)
        .expect("repeated includes should be loaded once");

    assert_eq!(sequent.left.len(), 1);
}

#[test]
fn build_problem_sequent_from_path_reports_missing_include_file() {
    let root = make_temp_dir("include_loader_missing");
    let problem_path = root.join("Problems").join("GEO").join("GEO171+2.p");
    write_file(
        &problem_path,
        r#"
include('Axioms/MISSING.ax').
fof(con,conjecture,p).
"#,
    );

    let err = build_problem_sequent_from_path(&problem_path)
        .expect_err("missing includes should fail explicitly");

    match err {
        ProblemPipelineError::Include(message) => {
            assert!(message.contains("Axioms/MISSING.ax"), "got: {message}");
        }
        other => panic!("expected include load failure, got {other:?}"),
    }
}

#[test]
fn build_problem_sequent_from_path_reports_include_cycles() {
    let root = make_temp_dir("include_loader_cycle");
    let problem_path = root.join("Problems").join("GEO").join("GEO171+2.p");
    let axiom_a = root.join("Axioms").join("A.ax");
    let axiom_b = root.join("Axioms").join("B.ax");
    write_file(
        &problem_path,
        r#"
include('Axioms/A.ax').
fof(con,conjecture,p).
"#,
    );
    write_file(
        &axiom_a,
        r#"
include('Axioms/B.ax').
fof(ax_a,axiom,p).
"#,
    );
    write_file(
        &axiom_b,
        r#"
include('Axioms/A.ax').
fof(ax_b,axiom,p).
"#,
    );

    let err = build_problem_sequent_from_path(&problem_path)
        .expect_err("include cycles should fail explicitly");

    match err {
        ProblemPipelineError::Include(message) => {
            assert!(message.contains("cycle"), "got: {message}");
        }
        other => panic!("expected include cycle failure, got {other:?}"),
    }
}

#[test]
fn build_problem_sequent_from_path_rejects_conjectures_inside_included_files() {
    let root = make_temp_dir("include_loader_rejects_included_conjecture");
    let problem_path = root.join("Problems").join("GEO").join("GEO171+2.p");
    write_file(
        &problem_path,
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );
    write_file(
        &root.join("Axioms").join("GEO008+0.ax"),
        r#"
fof(bad,conjecture,p).
"#,
    );

    let err = build_problem_sequent_from_path(&problem_path)
        .expect_err("included conjectures should fail explicitly");

    match err {
        ProblemPipelineError::Include(message) => {
            assert!(message.contains("conjecture"), "got: {message}");
        }
        other => panic!("expected included conjecture failure, got {other:?}"),
    }
}
