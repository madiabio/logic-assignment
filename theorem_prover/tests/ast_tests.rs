use theorem_prover::ast::{Atom, Formula, NumberLit, Symbol, Term, Var};

#[test]
fn term_variants_have_expected_shape() {
    let var = Term::Var(Var {
        name: "X".to_owned(),
    });
    assert!(matches!(var, Term::Var(Var { name }) if name == "X"));

    let constant = Term::Const(Symbol::User("a".to_owned()));
    assert!(matches!(constant, Term::Const(Symbol::User(name)) if name == "a"));

    let function = Term::Fun {
        name: Symbol::Defined("$f".to_owned()),
        args: vec![Term::Number(NumberLit::Integer("1".to_owned()))],
    };
    assert!(matches!(
        function,
        Term::Fun {
            name: Symbol::Defined(name),
            args
        } if name == "$f" && args.len() == 1
    ));

    let integer = Term::Number(NumberLit::Integer("-7".to_owned()));
    let rational = Term::Number(NumberLit::Rational("2/5".to_owned()));
    let real = Term::Number(NumberLit::Real("3.14".to_owned()));
    assert!(matches!(integer, Term::Number(NumberLit::Integer(value)) if value == "-7"));
    assert!(matches!(rational, Term::Number(NumberLit::Rational(value)) if value == "2/5"));
    assert!(matches!(real, Term::Number(NumberLit::Real(value)) if value == "3.14"));

    let distinct = Term::DistinctObject("\"obj\"".to_owned());
    assert!(matches!(distinct, Term::DistinctObject(value) if value == "\"obj\""));
}

#[test]
fn symbol_variants_are_classified() {
    let user = Symbol::User("plain_name".to_owned());
    let defined = Symbol::Defined("$defined".to_owned());
    let system = Symbol::System("$$system".to_owned());

    assert!(matches!(user, Symbol::User(value) if value == "plain_name"));
    assert!(matches!(defined, Symbol::Defined(value) if value == "$defined"));
    assert!(matches!(system, Symbol::System(value) if value == "$$system"));
}

#[test]
fn atom_variants_capture_predicate_and_equalities() {
    let predicate = Atom::Predicate {
        name: Symbol::User("p".to_owned()),
        args: vec![Term::Var(Var {
            name: "X".to_owned(),
        })],
    };
    assert!(matches!(
        predicate,
        Atom::Predicate {
            name: Symbol::User(name),
            args
        } if name == "p" && args.len() == 1
    ));

    let equality = Atom::Equality(
        Term::Const(Symbol::User("a".to_owned())),
        Term::Const(Symbol::User("b".to_owned())),
    );
    let inequality = Atom::Inequality(
        Term::Const(Symbol::User("a".to_owned())),
        Term::Const(Symbol::User("b".to_owned())),
    );
    assert!(matches!(equality, Atom::Equality(_, _)));
    assert!(matches!(inequality, Atom::Inequality(_, _)));
}

#[test]
fn formula_variants_cover_core_constructors() {
    let truth = Formula::True;
    let falsity = Formula::False;
    assert!(matches!(truth, Formula::True));
    assert!(matches!(falsity, Formula::False));

    let atom = Formula::Atom(Atom::Predicate {
        name: Symbol::User("p".to_owned()),
        args: vec![],
    });
    assert!(matches!(atom, Formula::Atom(Atom::Predicate { .. })));

    let not_formula = Formula::Not(Box::new(Formula::Atom(Atom::Predicate {
        name: Symbol::User("q".to_owned()),
        args: vec![],
    })));
    assert!(matches!(not_formula, Formula::Not(_)));

    let implies = Formula::Implies(
        Box::new(Formula::Atom(Atom::Predicate {
            name: Symbol::User("p".to_owned()),
            args: vec![],
        })),
        Box::new(Formula::Atom(Atom::Predicate {
            name: Symbol::User("q".to_owned()),
            args: vec![],
        })),
    );
    let iff = Formula::Iff(
        Box::new(Formula::Atom(Atom::Predicate {
            name: Symbol::User("p".to_owned()),
            args: vec![],
        })),
        Box::new(Formula::Atom(Atom::Predicate {
            name: Symbol::User("q".to_owned()),
            args: vec![],
        })),
    );
    assert!(matches!(implies, Formula::Implies(_, _)));
    assert!(matches!(iff, Formula::Iff(_, _)));

    let and_formula = Formula::And(vec![
        Formula::Atom(Atom::Predicate {
            name: Symbol::User("p".to_owned()),
            args: vec![],
        }),
        Formula::Atom(Atom::Predicate {
            name: Symbol::User("q".to_owned()),
            args: vec![],
        }),
        Formula::Atom(Atom::Predicate {
            name: Symbol::User("r".to_owned()),
            args: vec![],
        }),
    ]);
    let or_formula = Formula::Or(vec![
        Formula::Atom(Atom::Predicate {
            name: Symbol::User("p".to_owned()),
            args: vec![],
        }),
        Formula::Atom(Atom::Predicate {
            name: Symbol::User("q".to_owned()),
            args: vec![],
        }),
    ]);
    assert!(matches!(and_formula, Formula::And(items) if items.len() == 3));
    assert!(matches!(or_formula, Formula::Or(items) if items.len() == 2));

    let for_all = Formula::ForAll(
        vec![Var {
            name: "X".to_owned(),
        }],
        Box::new(Formula::Atom(Atom::Predicate {
            name: Symbol::User("p".to_owned()),
            args: vec![Term::Var(Var {
                name: "X".to_owned(),
            })],
        })),
    );
    let exists = Formula::Exists(
        vec![Var {
            name: "Y".to_owned(),
        }],
        Box::new(Formula::Atom(Atom::Predicate {
            name: Symbol::User("q".to_owned()),
            args: vec![Term::Var(Var {
                name: "Y".to_owned(),
            })],
        })),
    );
    assert!(matches!(for_all, Formula::ForAll(vars, _) if vars.len() == 1));
    assert!(matches!(exists, Formula::Exists(vars, _) if vars.len() == 1));
}
