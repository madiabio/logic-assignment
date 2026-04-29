use theorem_prover::ast::{Atom, Formula, Symbol};
use theorem_prover::proof::apply::{RuleApplication, apply_rule};
use theorem_prover::proof::rules::{Rule, RuleMatch, Side};
use theorem_prover::{ProofResult, ProofStatus, Sequent, parse_problem, prove};

fn trace_rule_applications_enabled() -> bool {
    std::env::var("RULE_TRACE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false)
}

fn apply_rule_with_optional_trace(sequent: &Sequent, rule_match: RuleMatch) -> RuleApplication {
    let application = apply_rule(sequent, &rule_match);

    if trace_rule_applications_enabled() {
        println!("Applying {:?} on {:?}[{}]", rule_match.rule, rule_match.side, rule_match.index);
        println!("  Input: {sequent}");
        match &application {
            RuleApplication::Closed => println!("  Result: closed"),
            RuleApplication::NotImplemented => println!("  Result: not implemented"),
            RuleApplication::Premises(premises) => {
                for (index, premise) in premises.iter().enumerate() {
                    println!("  Premise {}: {}", index + 1, premise);
                }
            }
        }
    }

    application
}

fn predicate_formula(name: &str) -> Formula {
    Formula::Atom(Atom::Predicate {
        name: Symbol::User(name.to_owned()),
        args: Vec::new(),
    })
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
fn apply_rule_expands_binary_left_conjunction_into_two_formulas() {
    let sequent = Sequent {
        left: vec![Formula::And(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("r")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::AndL,
            side: Side::Left,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![predicate_formula("p"), predicate_formula("q")],
            right: vec![predicate_formula("r")],
        }])
    );
}

#[test]
fn apply_rule_peels_leftmost_formula_from_multiway_left_conjunction() {
    let sequent = Sequent {
        left: vec![Formula::And(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
        right: vec![predicate_formula("goal")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::AndL,
            side: Side::Left,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![
                predicate_formula("p"),
                Formula::And(vec![predicate_formula("q"), predicate_formula("r")]),
            ],
            right: vec![predicate_formula("goal")],
        }])
    );
}

#[test]
fn prove_returns_not_provable_after_applying_left_connective_rule() {
    let sequent = Sequent {
        left: vec![Formula::And(vec![
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
        left: vec![Formula::And(vec![
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
        left: vec![Formula::And(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("p")],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn apply_rule_expands_binary_right_disjunction_into_two_formulas() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::Or(vec![
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::OrR,
            side: Side::Right,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![predicate_formula("p")],
            right: vec![predicate_formula("q"), predicate_formula("r")],
        }])
    );
}

#[test]
fn apply_rule_peels_leftmost_formula_from_multiway_right_disjunction() {
    let sequent = Sequent {
        left: vec![predicate_formula("source")],
        right: vec![Formula::Or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::OrR,
            side: Side::Right,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![predicate_formula("source")],
            right: vec![
                predicate_formula("p"),
                Formula::Or(vec![predicate_formula("q"), predicate_formula("r")]),
            ],
        }])
    );
}

#[test]
fn prove_returns_not_provable_after_applying_right_connective_rule() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::Or(vec![
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
        right: vec![Formula::Or(vec![
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
        right: vec![Formula::Or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn apply_rule_moves_implication_antecedent_left_and_consequent_right() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::Implies(
            Box::new(predicate_formula("p")),
            Box::new(predicate_formula("r")),
        )],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::ImpliesR,
            side: Side::Right,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![predicate_formula("q"), predicate_formula("p")],
            right: vec![predicate_formula("r")],
        }])
    );
}

#[test]
fn prove_returns_not_provable_after_applying_implies_right_rule() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::Implies(
            Box::new(predicate_formula("p")),
            Box::new(predicate_formula("r")),
        )],
    };

    let result = prove(&sequent);

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_provable_when_impliesr_exposes_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::Implies(
            Box::new(predicate_formula("p")),
            Box::new(predicate_formula("q")),
        )],
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
