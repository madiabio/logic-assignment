use theorem_prover::Sequent;
use theorem_prover::ast::Formula;
use theorem_prover::proof::apply::{RuleApplication, apply_rule};
use theorem_prover::proof::rules::{Rule, RuleMatch, Side};

fn trace_rule_applications_enabled() -> bool {
    std::env::var("RULE_TRACE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false)
}

fn apply_rule_with_optional_trace(sequent: &Sequent, rule_match: RuleMatch) -> RuleApplication {
    let application = apply_rule(sequent, &rule_match);

    if trace_rule_applications_enabled() {
        println!(
            "Applying {:?} on {:?}[{}]",
            rule_match.rule, rule_match.side, rule_match.index
        );
        println!("  Input: {sequent}");
        match &application {
            RuleApplication::Closed => println!("  Result: closed"),
            RuleApplication::NotImplemented => println!("  Result: not implemented"),
            RuleApplication::Error => println!("  Result: error"),
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
    Formula::atom(name)
}

#[test]
fn apply_rule_expands_binary_left_conjunction_into_two_formulas() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![
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
        left: vec![Formula::and(vec![
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
                Formula::and(vec![predicate_formula("q"), predicate_formula("r")]),
            ],
            right: vec![predicate_formula("goal")],
        }])
    );
}

#[test]
fn apply_rule_branches_binary_left_disjunction_into_two_premises() {
    let sequent = Sequent {
        left: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("goal")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::OrL,
            side: Side::Left,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![
            Sequent {
                left: vec![predicate_formula("p")],
                right: vec![predicate_formula("goal")],
            },
            Sequent {
                left: vec![predicate_formula("q")],
                right: vec![predicate_formula("goal")],
            },
        ])
    );
}

#[test]
fn apply_rule_peels_leftmost_formula_from_multiway_left_disjunction() {
    let sequent = Sequent {
        left: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
        right: vec![predicate_formula("goal")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::OrL,
            side: Side::Left,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![
            Sequent {
                left: vec![predicate_formula("p")],
                right: vec![predicate_formula("goal")],
            },
            Sequent {
                left: vec![Formula::or(vec![
                    predicate_formula("q"),
                    predicate_formula("r"),
                ])],
                right: vec![predicate_formula("goal")],
            },
        ])
    );
}

#[test]
fn apply_rule_preserves_other_left_formulas_when_applying_orl() {
    let sequent = Sequent {
        left: vec![
            predicate_formula("before"),
            Formula::or(vec![predicate_formula("p"), predicate_formula("q")]),
            predicate_formula("after"),
        ],
        right: vec![predicate_formula("goal")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::OrL,
            side: Side::Left,
            index: 1,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![
            Sequent {
                left: vec![
                    predicate_formula("before"),
                    predicate_formula("p"),
                    predicate_formula("after"),
                ],
                right: vec![predicate_formula("goal")],
            },
            Sequent {
                left: vec![
                    predicate_formula("before"),
                    predicate_formula("q"),
                    predicate_formula("after"),
                ],
                right: vec![predicate_formula("goal")],
            },
        ])
    );
}

#[test]
fn apply_rule_branches_binary_right_conjunction_into_two_premises() {
    let sequent = Sequent {
        left: vec![predicate_formula("source")],
        right: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::AndR,
            side: Side::Right,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![
            Sequent {
                left: vec![predicate_formula("source")],
                right: vec![predicate_formula("p")],
            },
            Sequent {
                left: vec![predicate_formula("source")],
                right: vec![predicate_formula("q")],
            },
        ])
    );
}

#[test]
fn apply_rule_peels_leftmost_formula_from_multiway_right_conjunction() {
    let sequent = Sequent {
        left: vec![predicate_formula("source")],
        right: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::AndR,
            side: Side::Right,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![
            Sequent {
                left: vec![predicate_formula("source")],
                right: vec![predicate_formula("p")],
            },
            Sequent {
                left: vec![predicate_formula("source")],
                right: vec![Formula::and(vec![
                    predicate_formula("q"),
                    predicate_formula("r"),
                ])],
            },
        ])
    );
}

#[test]
fn apply_rule_preserves_other_right_formulas_when_applying_andr() {
    let sequent = Sequent {
        left: vec![predicate_formula("left")],
        right: vec![
            predicate_formula("before"),
            Formula::and(vec![predicate_formula("p"), predicate_formula("q")]),
            predicate_formula("after"),
        ],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::AndR,
            side: Side::Right,
            index: 1,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![
            Sequent {
                left: vec![predicate_formula("left")],
                right: vec![
                    predicate_formula("before"),
                    predicate_formula("p"),
                    predicate_formula("after"),
                ],
            },
            Sequent {
                left: vec![predicate_formula("left")],
                right: vec![
                    predicate_formula("before"),
                    predicate_formula("q"),
                    predicate_formula("after"),
                ],
            },
        ])
    );
}

#[test]
fn apply_rule_expands_binary_right_disjunction_into_two_formulas() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::or(vec![
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
        right: vec![Formula::or(vec![
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
                Formula::or(vec![predicate_formula("q"), predicate_formula("r")]),
            ],
        }])
    );
}

#[test]
fn apply_rule_moves_implication_antecedent_left_and_consequent_right() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::implies(predicate_formula("p"), predicate_formula("r"))],
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
fn apply_rule_branches_left_implication_into_two_premises() {
    let sequent = Sequent {
        left: vec![Formula::implies(predicate_formula("p"), predicate_formula("q"))],
        right: vec![predicate_formula("goal")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::ImpliesL,
            side: Side::Left,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![
            Sequent {
                left: vec![],
                right: vec![predicate_formula("goal"), predicate_formula("p")],
            },
            Sequent {
                left: vec![predicate_formula("q")],
                right: vec![predicate_formula("goal")],
            },
        ])
    );
}

#[test]
fn apply_rule_preserves_other_left_formulas_when_applying_impliesl() {
    let sequent = Sequent {
        left: vec![
            predicate_formula("before"),
            Formula::implies(predicate_formula("p"), predicate_formula("q")),
            predicate_formula("after"),
        ],
        right: vec![predicate_formula("goal")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::ImpliesL,
            side: Side::Left,
            index: 1,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![
            Sequent {
                left: vec![predicate_formula("before"), predicate_formula("after")],
                right: vec![predicate_formula("goal"), predicate_formula("p")],
            },
            Sequent {
                left: vec![
                    predicate_formula("before"),
                    predicate_formula("after"),
                    predicate_formula("q"),
                ],
                right: vec![predicate_formula("goal")],
            },
        ])
    );
}

#[test]
fn apply_rule_moves_negated_formula_from_right_to_left() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::not(predicate_formula("p"))],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::NotR,
            side: Side::Right,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![predicate_formula("q"), predicate_formula("p")],
            right: vec![],
        }])
    );
}

#[test]
fn apply_rule_preserves_other_right_formulas_when_applying_notr() {
    let sequent = Sequent {
        left: vec![predicate_formula("left")],
        right: vec![
            predicate_formula("before"),
            Formula::not(predicate_formula("p")),
            predicate_formula("after"),
        ],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::NotR,
            side: Side::Right,
            index: 1,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![predicate_formula("left"), predicate_formula("p")],
            right: vec![predicate_formula("before"), predicate_formula("after")],
        }])
    );
}

#[test]
fn apply_rule_moves_negated_formula_from_left_to_right() {
    let sequent = Sequent {
        left: vec![Formula::not(predicate_formula("p"))],
        right: vec![predicate_formula("q")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::NotL,
            side: Side::Left,
            index: 0,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![],
            right: vec![predicate_formula("q"), predicate_formula("p")],
        }])
    );
}

#[test]
fn apply_rule_preserves_other_left_formulas_when_applying_notl() {
    let sequent = Sequent {
        left: vec![
            predicate_formula("before"),
            Formula::not(predicate_formula("p")),
            predicate_formula("after"),
        ],
        right: vec![predicate_formula("goal")],
    };

    let application = apply_rule_with_optional_trace(
        &sequent,
        RuleMatch {
            rule: Rule::NotL,
            side: Side::Left,
            index: 1,
        },
    );

    assert_eq!(
        application,
        RuleApplication::Premises(vec![Sequent {
            left: vec![predicate_formula("before"), predicate_formula("after")],
            right: vec![predicate_formula("goal"), predicate_formula("p")],
        }])
    );
}
