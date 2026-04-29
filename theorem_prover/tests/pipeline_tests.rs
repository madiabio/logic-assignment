use theorem_prover::{ProblemPipelineError, ProofStatus, run_problem};

#[test]
fn run_problem_returns_not_provable_for_atomic_dead_end_problem() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,p).
fof(hyp_1,hypothesis,q).
fof(conj_1,conjecture,r).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn run_problem_returns_not_provable_for_problem_with_left_conjunction() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,(p & q)).
fof(conj_1,conjecture,r).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn run_problem_returns_not_provable_for_multiway_left_conjunction_without_identity() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,(p & q & r)).
fof(conj_1,conjecture,goal).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn run_problem_returns_provable_for_problem_where_andl_exposes_identity() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,(p & q)).
fof(conj_1,conjecture,p).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn run_problem_returns_not_provable_for_problem_with_right_disjunction() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,(q | r)).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn run_problem_returns_not_provable_for_multiway_right_disjunction_without_identity() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,source).
fof(conj_1,conjecture,(p | q | r)).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn run_problem_returns_provable_for_problem_where_orr_exposes_identity() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,p).
fof(conj_1,conjecture,(p | q)).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn run_problem_returns_not_provable_for_problem_with_right_implication() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,q).
fof(conj_1,conjecture,(p => r)).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn run_problem_returns_provable_for_problem_where_impliesr_exposes_identity() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,q).
fof(conj_1,conjecture,(p => q)).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn run_problem_reports_parse_failures() {
    let err = run_problem("fof(bad,axiom,(p(a)).").expect_err("pipeline should reject bad syntax");

    match err {
        ProblemPipelineError::Parse(_) => {}
        other => panic!("expected parse failure, got {other:?}"),
    }
}

#[test]
fn run_problem_reports_sequent_build_failures() {
    let err = run_problem("fof(ax_1,axiom,p).").expect_err("pipeline should require a conjecture");

    match err {
        ProblemPipelineError::SequentBuild(inner) => {
            assert_eq!(format!("{inner:?}"), "MissingConjecture");
        }
        other => panic!("expected sequent build failure, got {other:?}"),
    }
}
