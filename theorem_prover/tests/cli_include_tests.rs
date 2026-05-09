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
    fs::create_dir_all(dir).expect("problem dir should be created");
    let path = dir.join(name);
    fs::write(&path, contents).expect("problem file should be written");
    path
}

#[test]
fn prove_subcommand_proves_problem_using_included_axioms() {
    let dir = make_temp_dir("prove_include_supported");
    let root = dir.join("TPTP-v9.2.1");
    let problem_path = write_problem_file(
        &root.join("Problems").join("GEO"),
        "GEO171+2.p",
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );
    write_problem_file(
        &root.join("Axioms"),
        "GEO008+0.ax",
        r#"
fof(ax_1,axiom,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args([
            "prove",
            "--problem-class", "provable",
            problem_path.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected included axioms to be loaded and proved\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
    assert!(
        !stdout.contains("not_provable"),
        "include problem must not be refuted as conjecture-only\nstdout:\n{stdout}"
    );
}

#[test]
fn prove_subcommand_continues_batch_after_include_backed_problem() {
    let dir = make_temp_dir("prove_include_supported_batch");
    let root = dir.join("TPTP-v9.2.1");
    write_problem_file(
        &root.join("Problems").join("GEO"),
        "GEO171+2.p",
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );
    write_problem_file(
        &root.join("Axioms"),
        "GEO008+0.ax",
        r#"
fof(ax_1,axiom,p).
"#,
    );
    write_problem_file(
        &root.join("Problems").join("SYN"),
        "SYN001+1.p",
        r#"
fof(ax_1,axiom,p).
fof(con,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable", dir.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected include-backed batch run to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(!stdout.contains("unsupported_include"), "stdout was:\n{stdout}");
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
    assert!(stdout.contains("summary"), "stdout was:\n{stdout}");
}

#[test]
fn rules_subcommand_inspects_problem_using_included_axioms() {
    let dir = make_temp_dir("rules_include_supported");
    let root = dir.join("TPTP-v9.2.1");
    let problem_path = write_problem_file(
        &root.join("Problems").join("GEO"),
        "GEO171+2.p",
        r#"
include('Axioms/GEO008+0.ax').
fof(con,conjecture,p).
"#,
    );
    write_problem_file(
        &root.join("Axioms"),
        "GEO008+0.ax",
        r#"
fof(ax_1,axiom,p).
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
        "expected included axioms to be loaded for rule inspection\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("yes"), "stdout was:\n{stdout}");
    assert!(!stdout.contains("unsupported_include"), "stdout was:\n{stdout}");
}
