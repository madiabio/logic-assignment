use theorem_prover::ast::{Atom, Formula, NumberLit, Symbol, Term};
use theorem_prover::{ParsedProblem, parse_problem, parse_tptp};

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
fn parses_supported_assignment_subset() {
    let input = r#"
fof(true_false,axiom,($true | $false)).
fof(single_quoted,axiom,('A proposition' | 'A predicate'(X))).
fof(useful_connectives,axiom,(! [X] : (p(X) => ? [Y] : (q(Y) & r(X,Y))))).
"#;
    parse_tptp(input).expect("supported assignment subset should parse");

    let parsed =
        parse_problem(input).expect("supported assignment subset should build AST records");

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

    match premise_formula(&parsed, "useful_connectives") {
        Formula::ForAll(_, body) => match body.as_ref() {
            Formula::Implies(left, right) => {
                assert!(matches!(left.as_ref(), Formula::Atom(_)));
                assert!(matches!(right.as_ref(), Formula::Exists(_, _)));
            }
            other => panic!("expected top-level implication in useful_connectives, got {other:?}"),
        },
        other => panic!("expected universal quantifier in useful_connectives, got {other:?}"),
    }
}

#[test]
fn flattens_and_chain_into_single_vector() {
    let parsed =
        parse_problem("fof(and_chain,axiom,(p & q & r & s)).").expect("formula should parse");

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
fn parses_mixed_binary_connectives_with_tptp_left_association() {
    let parsed = parse_problem("fof(mix,axiom,(p & q | r & s)).").expect("formula should parse");

    match first_premise_formula(&parsed) {
        Formula::And(items) => {
            assert_eq!(items.len(), 2);
            match &items[0] {
                Formula::Or(or_items) => {
                    assert_eq!(or_items.len(), 2);
                    assert!(matches!(or_items[0], Formula::And(_)));
                    assert!(matches!(or_items[1], Formula::Atom(_)));
                }
                other => panic!("expected left child to be Or(vec), got {other:?}"),
            }
            assert!(matches!(items[1], Formula::Atom(_)));
        }
        other => panic!("expected TPTP left-associated shape ((p & q) | r) & s, got {other:?}"),
    }
}

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

    let parsed = parse_problem("fof(defsys,axiom,$$sys($d)).").expect("formula should parse");
    match first_premise_formula(&parsed) {
        Formula::Atom(Atom::Predicate { name, args }) => {
            assert!(matches!(name, Symbol::System(value) if value == "$$sys"));
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Term::Const(Symbol::Defined(ref value)) if value == "$d"));
        }
        other => panic!("expected system predicate over defined constant, got {other:?}"),
    }
}

#[test]
fn preserves_number_and_distinct_object_terms() {
    let parsed = parse_problem("fof(nums,axiom,$$sys(3,2/5,-1.2,\"obj\",f(7))).")
        .expect("numeric/system term formula should parse");

    match first_premise_formula(&parsed) {
        Formula::Atom(Atom::Predicate { name, args }) => {
            assert!(matches!(name, Symbol::System(value) if value == "$$sys"));
            assert_eq!(args.len(), 5);
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
            assert!(matches!(args[4], Term::Fun { .. }));
        }
        other => panic!("expected system predicate over mixed term arguments, got {other:?}"),
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
    let input = "fof(bad_num,axiom,f(1E)).";
    assert!(parse_tptp(input).is_err());
    assert!(parse_problem(input).is_err());
}

#[test]
fn rejects_removed_connectives_and_equality_syntax() {
    for input in [
        "fof(bad_iff,axiom,(p <=> q)).",
        "fof(bad_xor,axiom,(p <~> q)).",
        "fof(bad_lte,axiom,(p <= q)).",
        "fof(bad_eq,axiom,(a = b)).",
        "fof(bad_neq,axiom,(a != b)).",
    ] {
        assert!(
            parse_tptp(input).is_err(),
            "input should be rejected: {input}"
        );
        assert!(
            parse_problem(input).is_err(),
            "problem should be rejected: {input}"
        );
    }
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
