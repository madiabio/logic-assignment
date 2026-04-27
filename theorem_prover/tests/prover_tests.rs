use theorem_prover::ast::{Atom, Formula, Symbol};
use theorem_prover::{ProofResult, ProofStatus, Sequent, parse_problem, prove};

fn predicate_formula(name: &str) -> Formula {
    Formula::Atom(Atom::Predicate {
        name: Symbol::User(name.to_owned()),
        args: Vec::new(),
    })
}

#[test]
fn prove_returns_not_implemented_for_basic_initial_sequent() {
    let sequent = Sequent {
        left: vec![predicate_formula("p"), predicate_formula("q")],
        right: vec![predicate_formula("r")],
    };

    let result = prove(&sequent);

    assert_eq!(
        result,
        ProofResult {
            status: ProofStatus::NotImplemented,
        }
    );
}

#[test]
fn prove_returns_not_implemented_for_empty_left_sequent() {
    let sequent = Sequent {
        left: Vec::new(),
        right: vec![predicate_formula("goal")],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotImplemented);
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

    assert_eq!(first.status, ProofStatus::NotImplemented);
    assert_eq!(second.status, ProofStatus::NotImplemented);
    assert_eq!(sequent, before);
}

#[test]
fn prove_accepts_sequent_built_from_parsed_problem() {
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

    assert_eq!(result.status, ProofStatus::NotImplemented);
}
