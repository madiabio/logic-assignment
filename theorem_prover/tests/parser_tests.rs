use std::fs;

use theorem_prover::ast::{Atom, Formula, NumberLit, Symbol, Term};
use theorem_prover::{ParsedProblem, parse_problem, parse_tptp};

fn read_problem(path: &str) -> String {
    fs::read_to_string(path).expect("failed to read problem file")
}

fn premise_formula<'a>(parsed: &'a ParsedProblem, name: &str) -> &'a Formula {
    &parsed
        .premises
        .iter()
        .find(|record| record.name == name)
        .unwrap_or_else(|| panic!("missing premise '{name}'"))
        .formula
}

fn first_premise_formula(parsed: &ParsedProblem) -> &Formula {
    &parsed
        .premises
        .first()
        .expect("expected at least one premise")
        .formula
}

#[test]
fn parses_syn000_assignment_subset() {
    let input = read_problem("../tptp_problems/SYN000+1.p");
    parse_tptp(&input).expect("SYN000+1.p should parse");

    let parsed = parse_problem(&input).expect("SYN000+1.p should build AST records");

    match premise_formula(&parsed, "true_false") {
        Formula::Or(items) => assert_eq!(items, &vec![Formula::True, Formula::False]),
        other => panic!("expected true_false to normalize to Or(True, False), got {other:?}"),
    }

    match premise_formula(&parsed, "single_quoted") {
        Formula::Or(items) => {
            assert!(matches!(
                &items[0],
                Formula::Atom(Atom::Predicate {
                    name: Symbol::User(name),
                    args
                }) if name == "'A proposition'" && args.is_empty()
            ));

            assert!(matches!(
                &items[1],
                Formula::Atom(Atom::Predicate {
                    name: Symbol::User(name),
                    args
                }) if name == "'A predicate'" && args.len() == 1
            ));
        }
        other => panic!("expected quoted symbol disjunction, got {other:?}"),
    }

    match premise_formula(&parsed, "equality") {
        Formula::Exists(vars, body) => {
            assert_eq!(vars.len(), 1);
            match body.as_ref() {
                Formula::ForAll(inner_vars, inner_body) => {
                    assert_eq!(inner_vars.len(), 2);
                    match inner_body.as_ref() {
                        Formula::Or(items) => {
                            assert_eq!(items.len(), 3);
                            assert!(matches!(items[0], Formula::Atom(Atom::Equality(_, _))));
                            assert!(matches!(items[1], Formula::Atom(Atom::Inequality(_, _))));
                            assert!(matches!(items[2], Formula::Atom(Atom::Equality(_, _))));
                        }
                        other => panic!("expected equality body to normalize as Or, got {other:?}"),
                    }
                }
                other => panic!("expected nested universal quantifier, got {other:?}"),
            }
        }
        other => panic!("expected existential quantifier at top level, got {other:?}"),
    }

    match premise_formula(&parsed, "useful_connectives") {
        Formula::ForAll(_, body) => match body.as_ref() {
            Formula::Iff(left, right) => {
                assert!(matches!(left.as_ref(), Formula::Implies(_, _)));
                assert!(matches!(right.as_ref(), Formula::Exists(_, _)));
            }
            other => panic!("expected top-level iff in useful_connectives, got {other:?}"),
        },
        other => panic!("expected universal quantifier in useful_connectives, got {other:?}"),
    }
}

#[test]
fn parses_syn001_biconditional_problem() {
    let input = read_problem("../tptp_problems/SYN001+1.p");
    parse_tptp(&input).expect("SYN001+1.p should parse");

    let parsed = parse_problem(&input).expect("SYN001+1.p should build AST records");
    let conjecture = parsed
        .conjecture
        .as_ref()
        .expect("SYN001+1.p should have conjecture");

    match &conjecture.formula {
        Formula::Iff(left, right) => {
            assert!(matches!(left.as_ref(), Formula::Not(_)));
            assert!(matches!(right.as_ref(), Formula::Atom(_)));
        }
        other => panic!("expected conjecture to be Iff, got {other:?}"),
    }
}

#[test]
fn parses_later_quantified_problem() {
    let input = read_problem("../tptp_problems/SYN036+1.p");
    parse_tptp(&input).expect("SYN036+1.p should parse");

    let parsed = parse_problem(&input).expect("SYN036+1.p should build AST records");
    let conjecture = parsed
        .conjecture
        .as_ref()
        .expect("SYN036+1.p should have conjecture");

    match &conjecture.formula {
        Formula::Iff(left, right) => {
            assert!(matches!(left.as_ref(), Formula::Iff(_, _)));
            assert!(matches!(right.as_ref(), Formula::Iff(_, _)));
        }
        other => panic!("expected SYN036 conjecture to be nested iff tree, got {other:?}"),
    }
}

#[test]
fn lowers_reverse_implication_for_lte() {
    let parsed = parse_problem("fof(lte,axiom,(p <= q)).").expect("formula should parse");

    match first_premise_formula(&parsed) {
        Formula::Implies(left, right) => {
            assert!(matches!(
                left.as_ref(),
                Formula::Atom(Atom::Predicate {
                    name: Symbol::User(name),
                    ..
                }) if name == "q"
            ));
            assert!(matches!(
                right.as_ref(),
                Formula::Atom(Atom::Predicate {
                    name: Symbol::User(name),
                    ..
                }) if name == "p"
            ));
        }
        other => panic!("expected <= to lower to reversed implication, got {other:?}"),
    }
}

#[test]
fn lowers_exclusive_biconditional_to_negated_iff() {
    let parsed = parse_problem("fof(xor_like,axiom,(p <~> q)).").expect("formula should parse");

    match first_premise_formula(&parsed) {
        Formula::Not(body) => assert!(matches!(body.as_ref(), Formula::Iff(_, _))),
        other => panic!("expected <~> to lower to Not(Iff(..)), got {other:?}"),
    }
}

#[test]
fn flattens_and_chain_into_single_vector() {
    let parsed = parse_problem("fof(and_chain,axiom,(p & q & r & s)).").expect("formula should parse");

    match first_premise_formula(&parsed) {
        Formula::And(items) => {
            assert_eq!(items.len(), 4);
            assert!(items.iter().all(|item| matches!(item, Formula::Atom(_))));
        }
        other => panic!("expected flattened And(vec), got {other:?}"),
    }
}

#[test]
fn flattens_or_chain_into_single_vector() {
    let parsed = parse_problem("fof(or_chain,axiom,(p | q | r)).").expect("formula should parse");

    match first_premise_formula(&parsed) {
        Formula::Or(items) => {
            assert_eq!(items.len(), 3);
            assert!(items.iter().all(|item| matches!(item, Formula::Atom(_))));
        }
        other => panic!("expected flattened Or(vec), got {other:?}"),
    }
}

#[test]
fn folds_mixed_assoc_chains_left_to_right() {
    let parsed = parse_problem("fof(mix,axiom,(p & q | r & s)).").expect("formula should parse");

    match first_premise_formula(&parsed) {
        Formula::And(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], Formula::Or(_)));
            assert!(matches!(items[1], Formula::Atom(_)));
        }
        other => panic!("expected deterministic left-fold shape to be And, got {other:?}"),
    }
}

#[test]
fn maps_true_false_constants_to_formula_variants() {
    let parsed = parse_problem("fof(tf,axiom,($true | $false)).").expect("formula should parse");

    match first_premise_formula(&parsed) {
        Formula::Or(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], Formula::True));
            assert!(matches!(items[1], Formula::False));
        }
        other => panic!("expected disjunction of True/False literals, got {other:?}"),
    }
}

#[test]
fn classifies_user_defined_and_system_symbols() {
    let parsed = parse_problem("fof(sym,axiom,('quoted' & `Back)).").expect("formula should parse");

    match first_premise_formula(&parsed) {
        Formula::And(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(
                &items[0],
                Formula::Atom(Atom::Predicate {
                    name: Symbol::User(name),
                    args
                }) if name == "'quoted'" && args.is_empty()
            ));
            assert!(matches!(
                &items[1],
                Formula::Atom(Atom::Predicate {
                    name: Symbol::User(name),
                    args
                }) if name == "`Back" && args.is_empty()
            ));
        }
        other => panic!("expected conjunction over user symbols, got {other:?}"),
    }

    let parsed = parse_problem("fof(defsys,axiom,($d = $$s)).").expect("formula should parse");
    match first_premise_formula(&parsed) {
        Formula::Atom(Atom::Equality(lhs, rhs)) => {
            assert!(matches!(lhs, Term::Const(Symbol::Defined(value)) if value == "$d"));
            assert!(matches!(rhs, Term::Const(Symbol::System(value)) if value == "$$s"));
        }
        other => panic!("expected equality between defined/system constants, got {other:?}"),
    }
}

#[test]
fn preserves_number_and_distinct_object_terms() {
    let parsed = parse_problem("fof(nums,axiom,($$sys(3,2/5,-1.2,\"obj\") = f(7))).")
        .expect("numeric/system term formula should parse");

    match first_premise_formula(&parsed) {
        Formula::Atom(Atom::Equality(Term::Fun { name, args }, rhs)) => {
            assert!(matches!(name, Symbol::System(value) if value == "$$sys"));
            assert_eq!(args.len(), 4);
            assert!(matches!(
                args[0],
                Term::Number(NumberLit::Integer(ref value)) if value == "3"
            ));
            assert!(matches!(
                args[1],
                Term::Number(NumberLit::Rational(ref value)) if value == "2/5"
            ));
            assert!(matches!(
                args[2],
                Term::Number(NumberLit::Real(ref value)) if value == "-1.2"
            ));
            assert!(matches!(args[3], Term::DistinctObject(ref obj) if obj == "\"obj\""));
            assert!(matches!(rhs, Term::Fun { .. }));
        }
        other => panic!("expected equality over system functor term, got {other:?}"),
    }
}

#[test]
fn routes_roles_into_premises_and_conjecture_and_ignores_others() {
    let input = r#"
fof(ax_1,axiom,p).
fof(hyp_1,hypothesis,q).
fof(conj_1,conjecture,r).
fof(lem_1,lemma,s).
"#;
    let parsed = parse_problem(input).expect("problem should parse");

    assert_eq!(parsed.premises.len(), 2);
    assert_eq!(parsed.premises[0].name, "ax_1");
    assert_eq!(parsed.premises[1].name, "hyp_1");

    let conjecture = parsed
        .conjecture
        .as_ref()
        .expect("conjecture should be captured");
    assert_eq!(conjecture.name, "conj_1");

    assert!(parsed.premises.iter().all(|record| record.name != "lem_1"));
}

#[test]
fn separates_premises_from_conjecture_in_syn000() {
    let input = read_problem("../tptp_problems/SYN000+1.p");
    let parsed = parse_problem(&input).expect("SYN000+1.p should parse into problem structure");

    assert_eq!(parsed.premises.len(), 8);
    assert_eq!(
        parsed
            .premises
            .iter()
            .map(|record| record.role.as_str())
            .collect::<Vec<_>>(),
        vec![
            "axiom",
            "axiom",
            "axiom",
            "axiom",
            "axiom",
            "axiom",
            "axiom",
            "hypothesis"
        ]
    );

    let conjecture = parsed
        .conjecture
        .as_ref()
        .expect("conjecture should be captured separately");
    assert_eq!(conjecture.name, "role_conjecture");
    assert!(matches!(conjecture.formula, Formula::Exists(_, _)));
}

#[test]
fn rejects_malformed_formula_syntax() {
    let input = "fof(bad,axiom,(p(a)).";
    assert!(parse_tptp(input).is_err());
    assert!(parse_problem(input).is_err());
}

#[test]
fn rejects_malformed_quantifier_variable_list() {
    let input = "fof(bad_q,axiom,(! [X,] : p(X))).";
    assert!(parse_tptp(input).is_err());
    assert!(parse_problem(input).is_err());
}

#[test]
fn rejects_malformed_numeric_literal_shape() {
    let input = "fof(bad_num,axiom,(f(1E) = a)).";
    assert!(parse_tptp(input).is_err());
    assert!(parse_problem(input).is_err());
}

#[test]
fn rejects_malformed_include_or_annotated_statement_syntax() {
    let bad_include = "include(foo).";
    assert!(parse_tptp(bad_include).is_err());
    assert!(parse_problem(bad_include).is_err());

    let bad_annotated = "fof(bad axiom,p).";
    assert!(parse_tptp(bad_annotated).is_err());
    assert!(parse_problem(bad_annotated).is_err());
}
