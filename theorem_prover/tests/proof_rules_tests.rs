use theorem_prover::Sequent;
use theorem_prover::ast::{Atom, Formula, Symbol, Var};
use theorem_prover::proof::rules::{Rule, RuleMatch, Side, find_applicable_rules};

fn atom(name: &str) -> Formula {
    Formula::Atom(Atom::Predicate {
        name: Symbol::User(name.to_owned()),
        args: vec![],
    })
}

fn var(name: &str) -> Var {
    Var {
        name: name.to_owned(),
    }
}

fn forall(name: &str, body: Formula) -> Formula {
    Formula::ForAll(vec![var(name)], Box::new(body))
}

fn exists(name: &str, body: Formula) -> Formula {
    Formula::Exists(vec![var(name)], Box::new(body))
}

#[test]
fn finds_identity_rule_for_matching_formula_on_both_sides() {
    let formula = atom("p");
    let sequent = Sequent {
        left: vec![formula.clone()],
        right: vec![formula],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![RuleMatch {
            rule: Rule::Id,
            side: Side::Left,
            index: 0,
        }]
    );
}

#[test]
fn finds_top_right_rule() {
    let sequent = Sequent {
        left: vec![atom("p")],
        right: vec![Formula::True],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![RuleMatch {
            rule: Rule::TopR,
            side: Side::Right,
            index: 0,
        }]
    );
}

#[test]
fn finds_bottom_left_rule() {
    let sequent = Sequent {
        left: vec![Formula::False],
        right: vec![atom("p")],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![RuleMatch {
            rule: Rule::BottomL,
            side: Side::Left,
            index: 0,
        }]
    );
}

#[test]
fn finds_forall_left_rule() {
    let sequent = Sequent {
        left: vec![forall("X", atom("p"))],
        right: vec![atom("q")],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![RuleMatch {
            rule: Rule::ForAllL,
            side: Side::Left,
            index: 0,
        }]
    );
}

#[test]
fn finds_forall_right_rule() {
    let sequent = Sequent {
        left: vec![atom("p")],
        right: vec![forall("X", atom("q"))],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![RuleMatch {
            rule: Rule::ForAllR,
            side: Side::Right,
            index: 0,
        }]
    );
}

#[test]
fn finds_exists_left_rule() {
    let sequent = Sequent {
        left: vec![exists("X", atom("p"))],
        right: vec![atom("q")],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![RuleMatch {
            rule: Rule::ExistsL,
            side: Side::Left,
            index: 0,
        }]
    );
}

#[test]
fn finds_exists_right_rule() {
    let sequent = Sequent {
        left: vec![atom("p")],
        right: vec![exists("X", atom("q"))],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![RuleMatch {
            rule: Rule::ExistsR,
            side: Side::Right,
            index: 0,
        }]
    );
}

#[test]
fn finds_multiple_quantifier_matches_with_distinct_indices() {
    let sequent = Sequent {
        left: vec![
            forall("X", atom("p")),
            atom("q"),
            forall("Y", atom("r")),
            forall("Z", atom("s")),
        ],
        right: vec![
            atom("t"),
            exists("A", atom("u")),
            exists("B", atom("v")),
        ],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![
            RuleMatch {
                rule: Rule::ForAllL,
                side: Side::Left,
                index: 0,
            },
            RuleMatch {
                rule: Rule::ForAllL,
                side: Side::Left,
                index: 2,
            },
            RuleMatch {
                rule: Rule::ForAllL,
                side: Side::Left,
                index: 3,
            },
            RuleMatch {
                rule: Rule::ExistsR,
                side: Side::Right,
                index: 1,
            },
            RuleMatch {
                rule: Rule::ExistsR,
                side: Side::Right,
                index: 2,
            },
        ]
    );
}

#[test]
fn returns_no_matches_for_simple_non_matching_sequent() {
    let sequent = Sequent {
        left: vec![atom("p")],
        right: vec![atom("q")],
    };

    assert!(find_applicable_rules(&sequent).is_empty());
}

#[test]
fn recognizes_connective_rules_on_both_sides_in_deterministic_order() {
    let sequent = Sequent {
        left: vec![
            atom("p"),
            Formula::And(vec![atom("a"), atom("b")]),
            Formula::Or(vec![atom("c"), atom("d")]),
            Formula::Implies(Box::new(atom("e")), Box::new(atom("f"))),
            Formula::Not(Box::new(atom("g"))),
            forall("X", atom("h")),
            exists("Y", atom("i")),
        ],
        right: vec![
            atom("p"),
            Formula::True,
            Formula::And(vec![atom("j"), atom("k")]),
            Formula::Or(vec![atom("l"), atom("m")]),
            Formula::Implies(Box::new(atom("n")), Box::new(atom("o"))),
            Formula::Not(Box::new(atom("q"))),
            forall("Z", atom("r")),
            exists("W", atom("s")),
        ],
    };

    assert_eq!(
        find_applicable_rules(&sequent),
        vec![
            RuleMatch {
                rule: Rule::Id,
                side: Side::Left,
                index: 0,
            },
            RuleMatch {
                rule: Rule::AndL,
                side: Side::Left,
                index: 1,
            },
            RuleMatch {
                rule: Rule::OrL,
                side: Side::Left,
                index: 2,
            },
            RuleMatch {
                rule: Rule::ImpliesL,
                side: Side::Left,
                index: 3,
            },
            RuleMatch {
                rule: Rule::NotL,
                side: Side::Left,
                index: 4,
            },
            RuleMatch {
                rule: Rule::ForAllL,
                side: Side::Left,
                index: 5,
            },
            RuleMatch {
                rule: Rule::ExistsL,
                side: Side::Left,
                index: 6,
            },
            RuleMatch {
                rule: Rule::TopR,
                side: Side::Right,
                index: 1,
            },
            RuleMatch {
                rule: Rule::AndR,
                side: Side::Right,
                index: 2,
            },
            RuleMatch {
                rule: Rule::OrR,
                side: Side::Right,
                index: 3,
            },
            RuleMatch {
                rule: Rule::ImpliesR,
                side: Side::Right,
                index: 4,
            },
            RuleMatch {
                rule: Rule::NotR,
                side: Side::Right,
                index: 5,
            },
            RuleMatch {
                rule: Rule::ForAllR,
                side: Side::Right,
                index: 6,
            },
            RuleMatch {
                rule: Rule::ExistsR,
                side: Side::Right,
                index: 7,
            },
        ]
    );
}

#[test]
fn matcher_does_not_mutate_the_borrowed_sequent() {
    let sequent = Sequent {
        left: vec![Formula::And(vec![atom("p"), atom("q")])],
        right: vec![Formula::Not(Box::new(atom("r")))],
    };
    let snapshot = sequent.clone();

    let _ = find_applicable_rules(&sequent);

    assert_eq!(sequent, snapshot);
}
