use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_dir(test_name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("theorem_prover_{test_name}_{unique}"));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

fn write_problem_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).expect("problem file should be written");
    path
}

#[test]
fn rules_subcommand_prints_matching_rules_for_a_single_file() {
    let dir = make_temp_dir("rules_single");
    let input = write_problem_file(
        &dir,
        "identity.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "expected success, got status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout,
        stderr
    );
    assert!(stdout.contains("p ⊢ p"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Id"), "stdout was:\n{stdout}");
}

#[test]
fn rules_subcommand_reports_when_a_file_has_no_applicable_rules() {
    let dir = make_temp_dir("rules_none");
    let input = write_problem_file(
        &dir,
        "no_match.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,q).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("p ⊢ q"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("no applicable rules"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn rules_subcommand_scans_only_p_files_in_a_directory_and_summarizes_results() {
    let dir = make_temp_dir("rules_dir");
    write_problem_file(
        &dir,
        "match.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );
    write_problem_file(
        &dir,
        "branch.p",
        r#"
fof(ax_1,axiom,(p | q)).
fof(conj_1,conjecture,r).
"#,
    );
    write_problem_file(
        &dir,
        "ignored.tptp",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", dir.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("match.p:"), "stdout was:\n{stdout}");
    assert!(stdout.contains("branch.p:"), "stdout was:\n{stdout}");
    assert!(!stdout.contains("ignored.tptp:"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("Processed 2 file(s)"),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("Succeeded: 2"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Failed: 0"), "stdout was:\n{stdout}");
}

#[test]
fn rules_subcommand_reports_sequent_build_failures_for_a_file() {
    let dir = make_temp_dir("rules_missing_conjecture");
    let input = write_problem_file(
        &dir,
        "missing_conjecture.p",
        r#"
fof(ax_1,axiom,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "stderr was:\n{stderr}");
    assert!(
        stderr.contains("sequent construction failed"),
        "stderr was:\n{stderr}"
    );
    assert!(
        stderr.contains("MissingConjecture"),
        "stderr was:\n{stderr}"
    );
}
