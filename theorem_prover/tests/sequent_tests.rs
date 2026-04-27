use theorem_prover::ast::{Atom, Formula, Symbol};
use theorem_prover::{FormulaRecord, ParsedProblem, Sequent, SequentBuildError};

fn predicate_formula(name: &str) -> Formula {
    Formula::Atom(Atom::Predicate {
        name: Symbol::User(name.to_owned()),
        args: Vec::new(),
    })
}

fn formula_record(name: &str, role: &str, formula: Formula) -> FormulaRecord {
    FormulaRecord {
        name: name.to_owned(),
        role: role.to_owned(),
        formula,
    }
}

#[test]
fn builds_initial_sequent_from_premises_and_conjecture() {
    let premise_one = predicate_formula("p");
    let premise_two = predicate_formula("q");
    let conjecture = predicate_formula("r");
    let parsed = ParsedProblem {
        premises: vec![
            formula_record("ax_1", "axiom", premise_one.clone()),
            formula_record("hyp_1", "hypothesis", premise_two.clone()),
        ],
        conjecture: Some(formula_record("conj_1", "conjecture", conjecture.clone())),
    };

    let sequent = Sequent::from_parsed_problem(parsed).expect("expected sequent to build");

    assert_eq!(sequent.left, vec![premise_one, premise_two]);
    assert_eq!(sequent.right, vec![conjecture]);
}

#[test]
fn builds_initial_sequent_without_premises() {
    let conjecture = predicate_formula("goal");
    let parsed = ParsedProblem {
        premises: Vec::new(),
        conjecture: Some(formula_record("conj_1", "conjecture", conjecture.clone())),
    };

    let sequent = Sequent::from_parsed_problem(parsed).expect("expected sequent to build");

    assert!(sequent.left.is_empty());
    assert_eq!(sequent.right, vec![conjecture]);
}

#[test]
fn rejects_problem_without_conjecture() {
    let parsed = ParsedProblem {
        premises: vec![formula_record("ax_1", "axiom", predicate_formula("p"))],
        conjecture: None,
    };

    let err = Sequent::from_parsed_problem(parsed).expect_err("expected missing conjecture");

    assert_eq!(err, SequentBuildError::MissingConjecture);
}
