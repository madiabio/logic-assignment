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

fn parse_failed_marker_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.parse_failed", path.display()))
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

#[test]
fn rules_subcommand_creates_parse_failed_marker_for_parse_errors() {
    let dir = make_temp_dir("rules_parse_marker");
    let input = write_problem_file(
        &dir,
        "bad_parse.p",
        r#"
fof(ax_1,axiom,p)
"#,
    );
    let marker = parse_failed_marker_path(&input);

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "stderr was:\n{stderr}");
    assert!(marker.exists(), "expected marker at {}", marker.display());

    let marker_contents = fs::read_to_string(&marker).expect("marker should be readable");
    assert!(
        marker_contents.contains(&input.display().to_string()),
        "marker contents were:\n{marker_contents}"
    );
    assert!(
        marker_contents.contains("parse failed"),
        "marker contents were:\n{marker_contents}"
    );
    assert!(
        marker_contents.contains("expected "),
        "marker contents were:\n{marker_contents}"
    );
}

#[test]
fn rules_subcommand_removes_stale_parse_failed_marker_after_successful_rerun() {
    let dir = make_temp_dir("rules_parse_marker_cleanup");
    let input = write_problem_file(
        &dir,
        "flaky_parse.p",
        r#"
fof(ax_1,axiom,p)
"#,
    );
    let marker = parse_failed_marker_path(&input);

    let first_run = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");
    assert!(
        !first_run.status.success(),
        "stderr was:\n{}",
        String::from_utf8_lossy(&first_run.stderr)
    );
    assert!(marker.exists(), "expected marker at {}", marker.display());

    fs::write(
        &input,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    )
    .expect("problem file should be rewritten");

    let second_run = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&second_run.stdout);
    let stderr = String::from_utf8_lossy(&second_run.stderr);
    assert!(
        second_run.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !marker.exists(),
        "expected stale marker to be removed at {}",
        marker.display()
    );
}

#[test]
fn rules_subcommand_does_not_create_parse_failed_marker_for_sequent_build_failures() {
    let dir = make_temp_dir("rules_no_marker_for_sequent_failure");
    let input = write_problem_file(
        &dir,
        "missing_conjecture_marker_check.p",
        r#"
fof(ax_1,axiom,p).
"#,
    );
    let marker = parse_failed_marker_path(&input);

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "stderr was:\n{stderr}");
    assert!(
        !marker.exists(),
        "did not expect marker for sequent-build failure at {}",
        marker.display()
    );
}

#[test]
fn rules_subcommand_directory_creates_markers_only_for_parse_failing_p_files() {
    let dir = make_temp_dir("rules_directory_markers");
    let good = write_problem_file(
        &dir,
        "good.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );
    let bad_parse = write_problem_file(
        &dir,
        "bad_parse.p",
        r#"
fof(ax_1,axiom,p)
"#,
    );
    let sequent_failure = write_problem_file(
        &dir,
        "missing_conjecture.p",
        r#"
fof(ax_1,axiom,p).
"#,
    );
    let ignored = write_problem_file(
        &dir,
        "ignored.tptp",
        r#"
fof(ax_1,axiom,p)
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", dir.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "expected failure summary\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        parse_failed_marker_path(&bad_parse).exists(),
        "expected marker for parse failure"
    );
    assert!(
        !parse_failed_marker_path(&good).exists(),
        "did not expect marker for successful file"
    );
    assert!(
        !parse_failed_marker_path(&sequent_failure).exists(),
        "did not expect marker for sequent-build failure"
    );
    assert!(
        !parse_failed_marker_path(&ignored).exists(),
        "did not expect marker for ignored non-.p file"
    );
}

#[test]
fn rules_subcommand_directory_skips_files_with_parse_failed_markers_by_default() {
    let dir = make_temp_dir("rules_directory_skip_marker");
    let skipped = write_problem_file(
        &dir,
        "skipped.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );
    let processed = write_problem_file(
        &dir,
        "processed.p",
        r#"
fof(ax_1,axiom,q).
fof(conj_1,conjecture,q).
"#,
    );
    fs::write(parse_failed_marker_path(&skipped), "stale marker").expect("marker should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["--rules", dir.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(!stdout.contains("skipped.p:"), "stdout was:\n{stdout}");
    assert!(stdout.contains("processed.p:"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("Processed 1 file(s)"),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("Skipped: 1"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Succeeded: 1"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Failed: 0"), "stdout was:\n{stdout}");
    assert!(
        parse_failed_marker_path(&skipped).exists(),
        "expected skipped marker to remain"
    );
    assert!(
        !parse_failed_marker_path(&processed).exists(),
        "did not expect a marker for processed file"
    );
}

#[test]
fn rules_subcommand_directory_retry_flag_reprocesses_parse_failed_files() {
    let dir = make_temp_dir("rules_directory_retry_marker");
    let retried = write_problem_file(
        &dir,
        "retried.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );
    let marker = parse_failed_marker_path(&retried);
    fs::write(&marker, "stale marker").expect("marker should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args([
            "--rules",
            "--retry-parse-failed",
            dir.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("retried.p:"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("Processed 1 file(s)"),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("Skipped: 0"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Succeeded: 1"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Failed: 0"), "stdout was:\n{stdout}");
    assert!(
        !marker.exists(),
        "expected retry flag to clear stale marker after success"
    );
}
