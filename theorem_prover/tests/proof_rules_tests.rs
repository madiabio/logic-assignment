use theorem_prover::Sequent;
use theorem_prover::ast::{Formula, Var};
use theorem_prover::proof::rules::{Rule, RuleMatch, Side, find_applicable_rules};

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
    let formula = Formula::atom("p");
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
fn finds_identity_rule_for_each_matching_left_formula() {
    let p = Formula::atom("p");
    let q = Formula::atom("q");
    let sequent = Sequent {
        left: vec![p.clone(), q.clone(), p.clone()],
        right: vec![q, p],
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
                rule: Rule::Id,
                side: Side::Left,
                index: 1,
            },
            RuleMatch {
                rule: Rule::Id,
                side: Side::Left,
                index: 2,
            },
        ]
    );
}

#[test]
fn finds_top_right_rule() {
    let sequent = Sequent {
        left: vec![Formula::atom("p")],
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
        right: vec![Formula::atom("p")],
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
        left: vec![forall("X", Formula::atom("p"))],
        right: vec![Formula::atom("q")],
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
        left: vec![Formula::atom("p")],
        right: vec![forall("X", Formula::atom("q"))],
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
        left: vec![exists("X", Formula::atom("p"))],
        right: vec![Formula::atom("q")],
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
        left: vec![Formula::atom("p")],
        right: vec![exists("X", Formula::atom("q"))],
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
            forall("X", Formula::atom("p")),
            Formula::atom("q"),
            forall("Y", Formula::atom("r")),
            forall("Z", Formula::atom("s")),
        ],
        right: vec![
            Formula::atom("t"),
            exists("A", Formula::atom("u")),
            exists("B", Formula::atom("v")),
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
        left: vec![Formula::atom("p")],
        right: vec![Formula::atom("q")],
    };

    assert!(find_applicable_rules(&sequent).is_empty());
}

#[test]
fn recognizes_connective_rules_on_both_sides_in_deterministic_order() {
    let sequent = Sequent {
        left: vec![
            Formula::atom("p"),
            Formula::And(vec![Formula::atom("a"), Formula::atom("b")]),
            Formula::Or(vec![Formula::atom("c"), Formula::atom("d")]),
            Formula::Implies(Box::new(Formula::atom("e")), Box::new(Formula::atom("f"))),
            Formula::Not(Box::new(Formula::atom("g"))),
            forall("X", Formula::atom("h")),
            exists("Y", Formula::atom("i")),
        ],
        right: vec![
            Formula::atom("p"),
            Formula::True,
            Formula::And(vec![Formula::atom("j"), Formula::atom("k")]),
            Formula::Or(vec![Formula::atom("l"), Formula::atom("m")]),
            Formula::Implies(Box::new(Formula::atom("n")), Box::new(Formula::atom("o"))),
            Formula::Not(Box::new(Formula::atom("q"))),
            forall("Z", Formula::atom("r")),
            exists("W", Formula::atom("s")),
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
        left: vec![Formula::And(vec![Formula::atom("p"), Formula::atom("q")])],
        right: vec![Formula::Not(Box::new(Formula::atom("r")))],
    };
    let snapshot = sequent.clone();

    let _ = find_applicable_rules(&sequent);

    assert_eq!(sequent, snapshot);
}
