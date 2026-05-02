use theorem_prover::ast::Formula;
use theorem_prover::{ProofResult, ProofStatus, Sequent, parse_problem, prove};

fn predicate_formula(name: &str) -> Formula {
    Formula::atom(name)
}

#[test]
fn prove_returns_not_provable_for_atomic_dead_end_sequent() {
    let sequent = Sequent {
        left: vec![predicate_formula("p"), predicate_formula("q")],
        right: vec![predicate_formula("r")],
    };

    let result = prove(&sequent);

    assert_eq!(
        result,
        ProofResult {
            status: ProofStatus::NotProvable,
        }
    );
}

#[test]
fn prove_returns_not_provable_for_empty_left_atomic_goal() {
    let sequent = Sequent {
        left: Vec::new(),
        right: vec![predicate_formula("goal")],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_not_provable_after_applying_left_connective_rule() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("r")],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_not_provable_when_andl_cannot_expose_identity() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
        right: vec![predicate_formula("goal")],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_provable_when_andl_exposes_identity() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("p")],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_provable_when_andr_exposes_identity_on_both_branches() {
    let sequent = Sequent {
        left: vec![predicate_formula("p"), predicate_formula("q")],
        right: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_not_provable_when_only_one_andr_branch_closes() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_reduces_multiway_andr_recursively() {
    let sequent = Sequent {
        left: vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ],
        right: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_not_provable_after_applying_right_connective_rule() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::or(vec![
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_not_provable_when_orr_cannot_expose_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("source")],
        right: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_provable_when_orr_exposes_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_not_provable_after_applying_implies_right_rule() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::implies(predicate_formula("p"), predicate_formula("r"))],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_provable_when_impliesr_exposes_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::implies(predicate_formula("p"), predicate_formula("q"))],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_provable_when_notr_exposes_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![
            Formula::not(predicate_formula("p")),
            predicate_formula("p"),
        ],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_provable_when_notl_exposes_identity() {
    let sequent = Sequent {
        left: vec![
            predicate_formula("p"),
            Formula::not(predicate_formula("q")),
            predicate_formula("q"),
        ],
        right: vec![],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_does_not_mutate_the_borrowed_sequent() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![predicate_formula("q")],
    };
    let before = sequent.clone();

    let first = prove(&sequent);
    let second = prove(&sequent);

    assert_eq!(first.status, ProofStatus::NotProvable);
    assert_eq!(second.status, ProofStatus::NotProvable);
    assert_eq!(sequent, before);
}

#[test]
fn prove_returns_not_provable_for_atomic_sequent_built_from_parsed_problem() {
    let parsed = parse_problem(
        r#"
fof(ax_1,axiom,p).
fof(hyp_1,hypothesis,q).
fof(conj_1,conjecture,r).
"#,
    )
    .expect("problem should parse");
    let sequent = Sequent::from_parsed_problem(parsed).expect("sequent should build");

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_not_provable_for_sequent_built_from_parsed_problem_with_connective() {
    let parsed = parse_problem(
        r#"
fof(ax_1,axiom,(p & q)).
fof(conj_1,conjecture,r).
"#,
    )
    .expect("problem should parse");
    let sequent = Sequent::from_parsed_problem(parsed).expect("sequent should build");

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn proves_identity_sequent() {
    let p = predicate_formula("p");

    let sequent = Sequent {
        left: vec![p.clone()],
        right: vec![p],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}
