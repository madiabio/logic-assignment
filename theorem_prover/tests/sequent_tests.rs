use theorem_prover::ast::Formula;
use theorem_prover::{FormulaRecord, ParsedProblem, Sequent, SequentBuildError};

fn predicate_formula(name: &str) -> Formula {
    Formula::atom(name)
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
        includes: Vec::new(),
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
        includes: Vec::new(),
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
        includes: Vec::new(),
    };

    let err = Sequent::from_parsed_problem(parsed).expect_err("expected missing conjecture");

    assert_eq!(err, SequentBuildError::MissingConjecture);
}

#[test]
fn displays_sequents_with_expected_turnstile_layout() {
    assert_eq!(
        format!(
            "{}",
            Sequent {
                left: vec![predicate_formula("p")],
                right: vec![predicate_formula("q")],
            }
        ),
        "p ⊢ q"
    );
    assert_eq!(
        format!(
            "{}",
            Sequent {
                left: vec![predicate_formula("p"), predicate_formula("q")],
                right: vec![predicate_formula("r"), predicate_formula("s")],
            }
        ),
        "p, q ⊢ r, s"
    );
    assert_eq!(
        format!(
            "{}",
            Sequent {
                left: Vec::new(),
                right: vec![predicate_formula("q")],
            }
        ),
        "⊢ q"
    );
    assert_eq!(
        format!(
            "{}",
            Sequent {
                left: vec![predicate_formula("p")],
                right: Vec::new(),
            }
        ),
        "p ⊢"
    );
    assert_eq!(
        format!(
            "{}",
            Sequent {
                left: Vec::new(),
                right: Vec::new(),
            }
        ),
        "⊢"
    );
}

#[test]
fn displays_nested_formulas_inside_sequents() {
    let left = Formula::And(vec![
        predicate_formula("p"),
        Formula::Implies(
            Box::new(predicate_formula("q")),
            Box::new(predicate_formula("r")),
        ),
    ]);
    let right = Formula::Implies(
        Box::new(Formula::And(vec![
            predicate_formula("s"),
            predicate_formula("t"),
        ])),
        Box::new(predicate_formula("u")),
    );

    assert_eq!(
        format!(
            "{}",
            Sequent {
                left: vec![left],
                right: vec![right],
            }
        ),
        "p ∧ (q ⇒ r) ⊢ s ∧ t ⇒ u"
    );
}
