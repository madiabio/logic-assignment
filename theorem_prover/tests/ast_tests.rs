use theorem_prover::ast::{Atom, Formula, NumberLit, Symbol, Term, Var};

fn var(name: &str) -> Var {
    Var {
        name: name.to_owned(),
    }
}

fn variable(name: &str) -> Term {
    Term::Var(var(name))
}

fn constant(name: &str) -> Term {
    Term::Const(Symbol::User(name.to_owned()))
}

fn predicate(name: &str) -> Atom {
    Atom::Predicate {
        name: Symbol::User(name.to_owned()),
        args: Vec::new(),
    }
}

fn predicate_with_args(name: &str, args: Vec<Term>) -> Atom {
    Atom::Predicate {
        name: Symbol::User(name.to_owned()),
        args,
    }
}

fn atom_formula(name: &str) -> Formula {
    Formula::atom(name)
}

#[test]
fn formula_constructors_build_expected_ast_shapes() {
    assert_eq!(Formula::atom("p"), atom_formula("p"));
    assert_eq!(
        Formula::predicate("p", vec![variable("X"), constant("a")]),
        Formula::Atom(predicate_with_args("p", vec![variable("X"), constant("a")]))
    );
    assert_eq!(Formula::not(atom_formula("p")), Formula::Not(Box::new(atom_formula("p"))));
    assert_eq!(
        Formula::and(vec![atom_formula("p"), atom_formula("q")]),
        Formula::And(vec![atom_formula("p"), atom_formula("q")])
    );
    assert_eq!(
        Formula::or(vec![atom_formula("p"), atom_formula("q")]),
        Formula::Or(vec![atom_formula("p"), atom_formula("q")])
    );
    assert_eq!(
        Formula::implies(atom_formula("p"), atom_formula("q")),
        Formula::Implies(Box::new(atom_formula("p")), Box::new(atom_formula("q")))
    );
}

#[test]
fn symbols_display_as_stored() {
    assert_eq!(format!("{}", Symbol::User("p".to_owned())), "p");
    assert_eq!(
        format!("{}", Symbol::Defined("$trueish".to_owned())),
        "$trueish"
    );
    assert_eq!(format!("{}", Symbol::System("$$sys".to_owned())), "$$sys");
}

#[test]
fn terms_display_all_variants() {
    assert_eq!(format!("{}", variable("X")), "X");
    assert_eq!(format!("{}", constant("a")), "a");
    assert_eq!(
        format!("{}", Term::Number(NumberLit::Integer("-7".to_owned()))),
        "-7"
    );
    assert_eq!(
        format!("{}", Term::Number(NumberLit::Rational("2/5".to_owned()))),
        "2/5"
    );
    assert_eq!(
        format!("{}", Term::Number(NumberLit::Real("3.14".to_owned()))),
        "3.14"
    );
    assert_eq!(
        format!("{}", Term::DistinctObject("\"obj\"".to_owned())),
        "\"obj\""
    );
    assert_eq!(
        format!(
            "{}",
            Term::Fun {
                name: Symbol::Defined("$f".to_owned()),
                args: vec![variable("X"), constant("a")],
            }
        ),
        "$f(X, a)"
    );
    assert_eq!(
        format!(
            "{}",
            Term::Fun {
                name: Symbol::User("c".to_owned()),
                args: Vec::new(),
            }
        ),
        "c()"
    );
}

#[test]
fn atoms_display_predicates() {
    assert_eq!(format!("{}", predicate("p")), "p");
    assert_eq!(
        format!(
            "{}",
            predicate_with_args("p", vec![variable("X"), constant("a")])
        ),
        "p(X, a)"
    );
}

#[test]
fn formulas_display_atoms_constants_and_quantifiers() {
    assert_eq!(format!("{}", Formula::True), "⊤");
    assert_eq!(format!("{}", Formula::False), "⊥");
    assert_eq!(format!("{}", atom_formula("p")), "p");
    assert_eq!(format!("{}", Formula::not(atom_formula("p"))), "¬p");
    assert_eq!(
        format!(
            "{}",
            Formula::not(Formula::and(vec![
                atom_formula("p"),
                atom_formula("q")
            ]))
        ),
        "¬(p ∧ q)"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::ForAll(
                vec![var("X")],
                Box::new(Formula::Atom(predicate_with_args("p", vec![variable("X")])))
            )
        ),
        "∀X. p(X)"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::Exists(
                vec![var("X"), var("Y")],
                Box::new(Formula::Atom(predicate_with_args(
                    "q",
                    vec![variable("X"), variable("Y")]
                )))
            )
        ),
        "∃X, Y. q(X, Y)"
    );
}

#[test]
fn formulas_display_connectives_with_precedence() {
    assert_eq!(
        format!(
            "{}",
            Formula::and(vec![
                atom_formula("p"),
                atom_formula("q"),
                atom_formula("r")
            ])
        ),
        "p ∧ q ∧ r"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::or(vec![atom_formula("p"), atom_formula("q")])
        ),
        "p ∨ q"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::implies(atom_formula("p"), atom_formula("q"))
        ),
        "p ⇒ q"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::implies(Formula::and(vec![atom_formula("p"), atom_formula("q")]), atom_formula("r"))
        ),
        "p ∧ q ⇒ r"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::and(vec![
                atom_formula("p"),
                Formula::implies(atom_formula("q"), atom_formula("r"))
            ])
        ),
        "p ∧ (q ⇒ r)"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::ForAll(
                vec![var("X")],
                Box::new(Formula::Implies(
                    Box::new(Formula::Atom(predicate_with_args("p", vec![variable("X")]))),
                    Box::new(atom_formula("q"))
                ))
            )
        ),
        "∀X. (p(X) ⇒ q)"
    );
}

#[test]
fn formulas_parenthesize_ambiguous_binary_nesting() {
    assert_eq!(
        format!(
            "{}",
            Formula::Implies(
                Box::new(atom_formula("p")),
                Box::new(Formula::Implies(
                    Box::new(atom_formula("q")),
                    Box::new(atom_formula("r"))
                ))
            )
        ),
        "p ⇒ (q ⇒ r)"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::Implies(
                Box::new(Formula::Implies(
                    Box::new(atom_formula("p")),
                    Box::new(atom_formula("q"))
                )),
                Box::new(atom_formula("r"))
            )
        ),
        "p ⇒ q ⇒ r"
    );
}

#[test]
fn term_substitution_replaces_variables_inside_nested_function_arguments() {
    let term = Term::Fun {
        name: Symbol::User("f".to_owned()),
        args: vec![
            variable("X"),
            Term::Fun {
                name: Symbol::User("g".to_owned()),
                args: vec![variable("X"), constant("a")],
            },
        ],
    };

    assert_eq!(
        term.substitute_var("X", &constant("b")),
        Term::Fun {
            name: Symbol::User("f".to_owned()),
            args: vec![
                constant("b"),
                Term::Fun {
                    name: Symbol::User("g".to_owned()),
                    args: vec![constant("b"), constant("a")],
                },
            ],
        }
    );
}

#[test]
fn formula_substitution_stops_at_shadowing_quantifier() {
    let formula = Formula::ForAll(
        vec![var("X")],
        Box::new(Formula::and(vec![
            Formula::predicate("p", vec![variable("X")]),
            Formula::Exists(
                vec![var("X")],
                Box::new(Formula::predicate("q", vec![variable("X")])),
            ),
        ])),
    );

    assert_eq!(
        formula.substitute_var("X", &constant("a")),
        Formula::ForAll(
            vec![var("X")],
            Box::new(Formula::and(vec![
                Formula::predicate("p", vec![variable("X")]),
                Formula::Exists(
                    vec![var("X")],
                    Box::new(Formula::predicate("q", vec![variable("X")])),
                ),
            ]))
        )
    );
}
