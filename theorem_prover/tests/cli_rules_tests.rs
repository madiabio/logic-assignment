use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dirs should be created");
    }
    fs::write(path, contents).expect("file should be written");
}

fn biconditional_chain_problem() -> &'static str {
    r#"
fof(conj_1,conjecture,
    (p_1 <=> p_2 <=> p_3 <=> p_4 <=> p_5 <=> p_6 <=> p_7 <=>
     p_8 <=> p_9 <=> p_10 <=> p_11 <=> p_12 <=> p_13 <=> p_14)).
"#
}

fn run_with_stdin(current_dir: &Path, args: &[&str], stdin_contents: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(current_dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary should run");

    let stdin = child.stdin.as_mut().expect("stdin should be available");
    write!(stdin, "{stdin_contents}").expect("stdin should be writable");
    drop(child.stdin.take());

    child.wait_with_output().expect("output should be captured")
}

#[test]
fn prove_subcommand_without_target_uses_configured_subset() {
    let dir = make_temp_dir("prove_config_subset");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
% header
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    let config_path = dir.join("config.toml");
    write_file(
        &config_path,
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier = 1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("idx"), "stdout was:\n{stdout}");
    assert!(stdout.contains("problem"), "stdout was:\n{stdout}");
    assert!(stdout.contains("status"), "stdout was:\n{stdout}");
    assert!(stdout.contains("time_ms"), "stdout was:\n{stdout}");
    assert!(stdout.contains("frm"), "stdout was:\n{stdout}");
    assert!(stdout.contains("atoms"), "stdout was:\n{stdout}");
    assert!(stdout.contains("1/1"), "stdout was:\n{stdout}");
    assert!(stdout.contains("SYN001+1"), "stdout was:\n{stdout}");
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
    assert!(stdout.contains("summary"), "stdout was:\n{stdout}");
    assert!(stdout.contains("processed"), "stdout was:\n{stdout}");
    assert!(
        stdout
            .lines()
            .next()
            .is_some_and(|line| line.starts_with("% settings ")),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("timeout_ms=1000"), "stdout was:\n{stdout}");
    assert!(stdout.contains("max_depth=50"), "stdout was:\n{stdout}");
    assert!(stdout.contains("max_steps=50"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("max_fresh_terms_per_quantifier=1"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn prove_subcommand_without_target_supports_tsv_output() {
    let dir = make_temp_dir("prove_config_subset_tsv");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier = 1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable", "--format", "tsv"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(
        stdout.contains(
            "kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tstatus\telapsed_ms\tdetail"
        ),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("problem\t1\t1\tSYN001+1\t"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("\t1\t1\tProvable\t"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("summary\t1\t0\t1\t0\t0\t0\t0\t0"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn prove_subcommand_without_target_resolves_versioned_problem_ids() {
    let dir = make_temp_dir("prove_config_versioned_subset");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root
        .join("Problems")
        .join("LCL")
        .join("LCL662+1.001.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
LCL662+1.001        FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier = 1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("LCL662+1.001"), "stdout was:\n{stdout}");
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
}

#[test]
fn rules_subcommand_without_target_uses_configured_subset() {
    let dir = make_temp_dir("rules_config_subset");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
% header
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier = 1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["rules"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("idx"), "stdout was:\n{stdout}");
    assert!(stdout.contains("problem"), "stdout was:\n{stdout}");
    assert!(stdout.contains("ok"), "stdout was:\n{stdout}");
    assert!(stdout.contains("match"), "stdout was:\n{stdout}");
    assert!(stdout.contains("1/1"), "stdout was:\n{stdout}");
    assert!(stdout.contains("SYN001+1"), "stdout was:\n{stdout}");
    assert!(stdout.contains("yes"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Id"), "stdout was:\n{stdout}");
    assert!(stdout.contains("summary"), "stdout was:\n{stdout}");
}

#[test]
fn rules_subcommand_without_target_supports_tsv_output() {
    let dir = make_temp_dir("rules_config_subset_tsv");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier = 1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["rules", "--format", "tsv"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(
        stdout.contains(
            "kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tsuccess\thad_rule_match\tdetail"
        ),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("problem\t1\t1\tSYN001+1\t"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("\t1\t1\ttrue\ttrue"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("summary\t1\t0\t0\t1\t0\t1"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn prove_subcommand_without_target_uses_proof_limits_from_config() {
    let dir = make_temp_dir("prove_config_limits");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 0\nmax_fresh_terms_per_quantifier = 1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("unknown"), "stdout was:\n{stdout}");
    assert!(stdout.contains("max_steps"), "stdout was:\n{stdout}");
}

#[test]
fn prove_subcommand_without_target_allows_cli_flags_to_override_config_limits() {
    let dir = make_temp_dir("prove_config_override");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 0\nmax_fresh_terms_per_quantifier = 1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable", "--max-steps", "10"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
}

#[test]
fn prove_subcommand_without_target_prompts_and_writes_config_on_first_run() {
    let dir = make_temp_dir("prove_first_run_prompt");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    let output = run_with_stdin(
        &dir,
        &["prove", "--problem-class", "provable"],
        &format!(
            "{}\n{}\n1000\n50\n50\n1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        dir.join("config.toml").exists(),
        "expected config.toml to be written"
    );
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
}

#[test]
fn prove_subcommand_with_invalid_config_decline_does_not_overwrite_file() {
    let dir = make_temp_dir("prove_invalid_config_decline");
    let config_path = dir.join("config.toml");
    let original_config = r#"
tptp_root = "..\TPTP-v9.2.1"
default_subset_file = "..\subset_descriptions\easy_problems.txt"
timeout_ms = 100
max_depth = 50
max_steps = 50
max_fresh_terms_per_quantifier =
"#;
    write_file(&config_path, original_config);

    let output = run_with_stdin(&dir, &["prove", "--problem-class", "provable"], "n\n");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("invalid max_fresh_terms_per_quantifier in config.toml"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("Replace or repair config.toml now? [y/N]:"),
        "stdout was:\n{stdout}"
    );
    assert_eq!(
        fs::read_to_string(&config_path).expect("config should still be readable"),
        original_config
    );
}

#[test]
fn prove_subcommand_with_invalid_config_confirm_rewrites_file_and_continues() {
    let dir = make_temp_dir("prove_invalid_config_confirm");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let subset_path = dir.join("subset_descriptions").join("easy_problems.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    let config_path = dir.join("config.toml");
    write_file(
        &config_path,
        "tptp_root = \".\\TPTP-v9.2.1\"\ndefault_subset_file = \".\\subset_descriptions\\easy_problems.txt\"\ntimeout_ms = 100\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier =\n",
    );

    let output = run_with_stdin(
        &dir,
        &["prove", "--problem-class", "provable"],
        &format!(
            "y\n{}\n{}\n1000\n50\n50\n1\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("invalid max_fresh_terms_per_quantifier in config.toml"),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");

    let rewritten = fs::read_to_string(&config_path).expect("config should be readable");
    assert!(
        rewritten.contains("max_fresh_terms_per_quantifier = 1"),
        "config contents were:\n{rewritten}"
    );
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
        .args([
            "rules",
            "--show-sequent",
            input.to_str().expect("path should be utf-8"),
        ])
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
fn rules_subcommand_hides_sequent_by_default() {
    let dir = make_temp_dir("rules_hide_sequent");
    let input = write_problem_file(
        &dir,
        "identity_hidden.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(!stdout.contains("p ⊢ p"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Id"), "stdout was:\n{stdout}");
}

#[test]
fn rules_subcommand_prints_sequent_when_show_sequent_flag_is_present() {
    let dir = make_temp_dir("rules_show_sequent");
    let input = write_problem_file(
        &dir,
        "identity_visible.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args([
            "rules",
            "--show-sequent",
            input.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("p ⊢ p"), "stdout was:\n{stdout}");
    assert!(stdout.contains("Id"), "stdout was:\n{stdout}");
}

#[test]
fn prover_mode_hides_sequent_by_default() {
    let dir = make_temp_dir("prover_hide_sequent");
    let input = write_problem_file(
        &dir,
        "identity_default.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["prove", "--problem-class", "provable", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(!stdout.contains("p ⊢ p"), "stdout was:\n{stdout}");
    assert!(stdout.contains("idx"), "stdout was:\n{stdout}");
    assert!(stdout.contains("problem"), "stdout was:\n{stdout}");
    assert!(stdout.contains("status"), "stdout was:\n{stdout}");
    assert!(stdout.contains("identity_default"), "stdout was:\n{stdout}");
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
}

#[test]
fn prover_mode_prints_sequent_when_show_sequent_flag_is_present() {
    let dir = make_temp_dir("prover_show_sequent");
    let input = write_problem_file(
        &dir,
        "identity_visible.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args([
            "prove",
            "--problem-class", "provable",
            "--show-sequent",
            input.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("p ⊢ p"), "stdout was:\n{stdout}");
    assert!(stdout.contains("identity_visible"), "stdout was:\n{stdout}");
    assert!(stdout.contains("provable"), "stdout was:\n{stdout}");
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
        .args(["rules", input.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
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
        .args(["rules", dir.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("idx"), "stdout was:\n{stdout}");
    assert!(stdout.contains("problem"), "stdout was:\n{stdout}");
    assert!(stdout.contains("match"), "stdout was:\n{stdout}");
    assert!(stdout.contains("branch"), "stdout was:\n{stdout}");
    assert!(!stdout.contains("ignored.tptp"), "stdout was:\n{stdout}");
    assert!(stdout.contains("summary"), "stdout was:\n{stdout}");
    assert!(stdout.contains("succeeded"), "stdout was:\n{stdout}");
    assert!(stdout.contains("failed"), "stdout was:\n{stdout}");
    assert!(stdout.contains("rule_matches"), "stdout was:\n{stdout}");
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
        .args(["rules", input.to_str().expect("path should be utf-8")])
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
        .args(["rules", input.to_str().expect("path should be utf-8")])
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
        .args(["rules", input.to_str().expect("path should be utf-8")])
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
        .args(["rules", input.to_str().expect("path should be utf-8")])
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
        .args(["rules", input.to_str().expect("path should be utf-8")])
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
        .args(["rules", dir.to_str().expect("path should be utf-8")])
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
    fs::write(parse_failed_marker_path(&skipped), "stale marker")
        .expect("marker should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["rules", dir.to_str().expect("path should be utf-8")])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(!stdout.contains("skipped.p"), "stdout was:\n{stdout}");
    assert!(stdout.contains("processed"), "stdout was:\n{stdout}");
    assert!(stdout.contains("summary"), "stdout was:\n{stdout}");
    assert!(stdout.contains("skipped"), "stdout was:\n{stdout}");
    assert!(stdout.contains("succeeded"), "stdout was:\n{stdout}");
    assert!(stdout.contains("failed"), "stdout was:\n{stdout}");
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
            "rules",
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
    assert!(stdout.contains("retried"), "stdout was:\n{stdout}");
    assert!(stdout.contains("summary"), "stdout was:\n{stdout}");
    assert!(stdout.contains("skipped"), "stdout was:\n{stdout}");
    assert!(stdout.contains("succeeded"), "stdout was:\n{stdout}");
    assert!(stdout.contains("failed"), "stdout was:\n{stdout}");
    assert!(
        !marker.exists(),
        "expected retry flag to clear stale marker after success"
    );
}

#[test]
fn prove_subcommand_reports_unknown_when_step_limit_is_hit() {
    let dir = make_temp_dir("prove_unknown_step_limit");
    let input = write_problem_file(
        &dir,
        "bounded_unknown.p",
        r#"
fof(ax_1,axiom,(p & q)).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args([
            "prove",
            "--problem-class", "provable",
            "--max-steps",
            "0",
            input.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("bounded_unknown"), "stdout was:\n{stdout}");
    assert!(stdout.contains("unknown"), "stdout was:\n{stdout}");
    assert!(stdout.contains("max_steps"), "stdout was:\n{stdout}");
}

#[test]
fn prove_subcommand_uses_biconditional_cap_from_config_and_reports_reason() {
    let dir = make_temp_dir("prove_biconditional_cap_from_config");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(&problem_path, biconditional_chain_problem());

    let subset_path = dir.join("subset_descriptions").join("biconditionals.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier = 1\nmax_biconditionals = 12\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("unknown"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("biconditional_cap"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn prove_subcommand_tsv_includes_unknown_reason_column() {
    let dir = make_temp_dir("prove_tsv_unknown_reason");
    let input = write_problem_file(
        &dir,
        "bounded_unknown.p",
        r#"
fof(ax_1,axiom,(p & q)).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args([
            "prove",
            "--problem-class", "provable",
            "--format",
            "tsv",
            "--max-steps",
            "0",
            input.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "exit: {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status.code(),
    );
    assert!(
        stdout.contains(
            "kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tstatus\telapsed_ms\tdetail"
        ),
        "stdout was:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("\tUnknown\t"),
        "stdout was:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("\tmax_steps"),
        "stdout was:\n{stdout}\nstderr:\n{stderr}"
    );
}

#[test]
fn prove_subcommand_uses_quantifier_budget_from_config() {
    let dir = make_temp_dir("prove_quantifier_budget_from_config");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(conj_1,conjecture,? [X] : p(X)).
"#,
    );

    let subset_path = dir
        .join("subset_descriptions")
        .join("quantifier_budget.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier = 0\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("unknown"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("quantifier_budget"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn prove_subcommand_cli_overrides_quantifier_budget_from_config() {
    let dir = make_temp_dir("prove_quantifier_budget_override");
    let tptp_root = dir.join("TPTP-v9.2.1");
    let problem_path = tptp_root.join("Problems").join("SYN").join("SYN001+1.p");
    write_file(
        &problem_path,
        r#"
fof(conj_1,conjecture,? [X] : p(X)).
"#,
    );

    let subset_path = dir
        .join("subset_descriptions")
        .join("quantifier_budget_override.txt");
    write_file(
        &subset_path,
        r#"
SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1
"#,
    );

    write_file(
        &dir.join("config.toml"),
        &format!(
            "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = 1000\nmax_depth = 50\nmax_steps = 50\nmax_fresh_terms_per_quantifier = 0\n",
            tptp_root.display(),
            subset_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .current_dir(&dir)
        .args(["prove", "--problem-class", "provable", "--max-fresh-terms-per-quantifier", "1"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(stdout.contains("not_provable"), "stdout was:\n{stdout}");
}

#[test]
fn prove_subcommand_help_describes_quantifier_budget_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["prove", "--help"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("--max-fresh-terms-per-quantifier"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("Maximum fresh fallback terms allowed per quantified occurrence."),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn rules_subcommand_skips_biconditional_cap_without_parse_failed_marker() {
    let dir = make_temp_dir("rules_biconditional_cap_skip");
    let input = write_problem_file(&dir, "too_many_iffs.p", biconditional_chain_problem());
    let marker = parse_failed_marker_path(&input);

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args([
            "rules",
            "--max-biconditionals",
            "12",
            input.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("biconditional_cap"),
        "stdout was:\n{stdout}"
    );
    assert!(
        !marker.exists(),
        "did not expect marker at {}",
        marker.display()
    );
}

#[test]
fn rules_subcommand_help_describes_biconditional_cap_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args(["rules", "--help"])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("--max-biconditionals"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("Skip inputs whose non-comment `<=>` count exceeds this limit."),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn rules_subcommand_prints_effective_settings_comment_line() {
    let dir = make_temp_dir("rules_settings_comment");
    let input = write_problem_file(
        &dir,
        "identity.p",
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,p).
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_theorem_prover"))
        .args([
            "rules",
            "--show-sequent",
            "--max-biconditionals",
            "12",
            input.to_str().expect("path should be utf-8"),
        ])
        .output()
        .expect("binary should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(
        stdout
            .lines()
            .next()
            .is_some_and(|line| line.starts_with("% settings ")),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("format=human"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("show_sequent=true"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("retry_parse_failed=false"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("max_biconditionals=12"),
        "stdout was:\n{stdout}"
    );
}
