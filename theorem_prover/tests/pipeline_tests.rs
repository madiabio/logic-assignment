use theorem_prover::{
    BiconditionalPolicy, ProblemPipelineError, ProofOptions, ProofStatus, RunProblemOptions,
    UnknownReason, run_problem, run_problem_with_options,
};

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
fn run_problem_proves_syn968_shape_by_revisiting_exists_right() {
    let result = run_problem(
        r#"
fof(conj_1,conjecture,? [X] : ! [Y] : (p(X) => p(Y))).
"#,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn run_problem_does_not_refute_open_quantified_theorem_shape() {
    let result = run_problem(
        r#"
fof(conj_1,conjecture,~ ? [Y] : ! [X] : (a(X,Y) <=> ~ a(X,X))).
"#,
    )
    .expect("pipeline should succeed");

    assert_ne!(result.status, ProofStatus::NotProvable);
}

#[test]
fn run_problem_returns_unknown_when_biconditional_cap_is_exceeded() {
    let result = run_problem_with_options(
        r#"
p_1 <=> p_2 <=> p_3 <=> p_4 <=> p_5 <=> p_6 <=> p_7 <=>
p_8 <=> p_9 <=> p_10 <=> p_11 <=> p_12 <=> p_13 <=> p_14
"#,
        RunProblemOptions {
            proof: ProofOptions::default(),
            biconditional_policy: BiconditionalPolicy {
                max_biconditionals: Some(12),
            },
            ..RunProblemOptions::default()
        },
    )
    .expect("pipeline should return an inconclusive proof result");

    assert_eq!(result.status, ProofStatus::Unknown);
    assert_eq!(
        result.unknown_reason,
        Some(UnknownReason::BiconditionalCapExceeded)
    );
}

#[test]
fn run_problem_without_biconditional_cap_still_reports_parse_failures() {
    let err = run_problem_with_options(
        r#"
p_1 <=> p_2 <=> p_3 <=> p_4 <=> p_5 <=> p_6 <=> p_7 <=>
p_8 <=> p_9 <=> p_10 <=> p_11 <=> p_12 <=> p_13 <=> p_14
"#,
        RunProblemOptions::default(),
    )
    .expect_err("pipeline should parse input when no biconditional cap is configured");

    match err {
        ProblemPipelineError::Parse(_) => {}
        other => panic!("expected parse failure, got {other:?}"),
    }
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
