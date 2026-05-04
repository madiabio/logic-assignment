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
fn prove_subcommand_reports_unknown_for_unsupported_include() {
    let dir = make_temp_dir("prove_unsupported_include");
    let problem_path = write_problem_file(
        &dir,
        "GEO171+2.p",
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args([
            "prove",
            problem_path.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected unsupported include to be reported as unknown\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("unknown (unsupported_include)"),
        "stdout was:\n{stdout}"
    );
    assert!(
        !stdout.contains("not_provable"),
        "include problem must not be refuted as conjecture-only\nstdout:\n{stdout}"
    );
}

#[test]
fn prove_subcommand_continues_batch_after_unsupported_include() {
    let dir = make_temp_dir("prove_unsupported_include_batch");
    write_problem_file(
        &dir,
        "GEO171+2.p",
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );
    write_problem_file(
        &dir,
        "SYN001+1.p",
        r#"
fof(ax_1,axiom,p).
fof(con,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", dir.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected unsupported include to be an unknown batch result\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("unknown (unsupported_include)"),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
    assert!(stdout.contains("summary"), "stdout was:\n{stdout}");
}

#[test]
fn rules_subcommand_skips_unsupported_include_without_building_sequent() {
    let dir = make_temp_dir("rules_unsupported_include");
    let problem_path = write_problem_file(
        &dir,
        "GEO171+2.p",
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args([
            "rules",
            problem_path.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected unsupported include to be skipped by rule inspection\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("unsupported_include"),
        "stdout was:\n{stdout}"
    );
    assert!(
        !stdout.contains("no applicable rules"),
        "rules must not inspect a conjecture-only sequent\nstdout:\n{stdout}"
    );
}
