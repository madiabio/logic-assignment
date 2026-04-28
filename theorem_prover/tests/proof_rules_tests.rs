use theorem_prover::Sequent;
use theorem_prover::ast::{Atom, Formula, Symbol};
use theorem_prover::proof::rules::{Rule, RuleMatch, Side, find_applicable_rules};

fn atom(name: &str) -> Formula {
    Formula::Atom(Atom::Predicate {
        name: Symbol::User(name.to_owned()),
        args: vec![],
    })
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
        ],
        right: vec![
            atom("p"),
            Formula::True,
            Formula::And(vec![atom("h"), atom("i")]),
            Formula::Or(vec![atom("j"), atom("k")]),
            Formula::Implies(Box::new(atom("l")), Box::new(atom("m"))),
            Formula::Not(Box::new(atom("n"))),
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
