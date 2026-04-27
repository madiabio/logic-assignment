use theorem_prover::{ProofStatus, ProblemPipelineError, run_problem};

#[test]
fn run_problem_feeds_built_sequent_to_the_prover() {
    let result = run_problem(
        r#"
fof(ax_1,axiom,p).
fof(hyp_1,hypothesis,q).
fof(conj_1,conjecture,r).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::NotImplemented);
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
