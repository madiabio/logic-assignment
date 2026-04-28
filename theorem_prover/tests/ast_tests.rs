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
    Formula::Atom(predicate(name))
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
    assert_eq!(
        format!("{}", Formula::Not(Box::new(atom_formula("p")))),
        "¬p"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::Not(Box::new(Formula::And(vec![
                atom_formula("p"),
                atom_formula("q")
            ])))
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
            Formula::And(vec![
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
            Formula::Or(vec![atom_formula("p"), atom_formula("q")])
        ),
        "p ∨ q"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::Implies(Box::new(atom_formula("p")), Box::new(atom_formula("q")))
        ),
        "p ⇒ q"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::Implies(
                Box::new(Formula::And(vec![atom_formula("p"), atom_formula("q")])),
                Box::new(atom_formula("r"))
            )
        ),
        "p ∧ q ⇒ r"
    );
    assert_eq!(
        format!(
            "{}",
            Formula::And(vec![
                atom_formula("p"),
                Formula::Implies(Box::new(atom_formula("q")), Box::new(atom_formula("r")))
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
