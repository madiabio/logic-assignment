use pest::iterators::Pair;

use crate::ast::{Atom, Formula, NumberLit, Symbol, Term, Var};

use super::Rule;

pub fn build_formula(pair: Pair<'_, Rule>) -> Formula {
    match pair.as_rule() {
        Rule::fof_formula => build_fof_formula(pair),
        Rule::fof_unit_formula
        | Rule::fof_unitary_formula
        | Rule::fof_atomic_formula
        | Rule::fof_plain_atomic_formula
        | Rule::fof_defined_atomic_formula
        | Rule::fof_system_atomic_formula => {
            let inner = pair
                .into_inner()
                .next()
                .expect("formula wrapper should contain inner formula");
            build_formula(inner)
        }
        Rule::fof_unary_formula => build_unary_formula(pair),
        Rule::fof_quantified_formula => build_quantified_formula(pair),
        Rule::fof_plain_term => predicate_from_term_rule(pair),
        Rule::fof_defined_plain_term => predicate_from_defined_plain_term(pair),
        Rule::fof_system_term => predicate_from_system_term(pair),
        _ => panic!("unsupported formula rule: {:?}", pair.as_rule()),
    }
}

pub fn build_term(pair: Pair<'_, Rule>) -> Term {
    match pair.as_rule() {
        Rule::fof_term | Rule::fof_function_term | Rule::fof_defined_atomic_term => {
            let inner = pair
                .into_inner()
                .next()
                .expect("term wrapper should contain inner term");
            build_term(inner)
        }
        Rule::variable => Term::Var(Var {
            name: pair.as_str().to_owned(),
        }),
        Rule::fof_plain_term => term_from_plain_term(pair),
        Rule::fof_defined_term => {
            let inner = pair
                .into_inner()
                .next()
                .expect("defined term should contain inner value");
            build_term(inner)
        }
        Rule::fof_defined_plain_term => term_from_defined_plain_term(pair),
        Rule::fof_system_term => term_from_system_term(pair),
        Rule::number => {
            let inner = pair
                .into_inner()
                .next()
                .expect("number should contain literal kind");
            build_term(inner)
        }
        Rule::integer => Term::Number(NumberLit::Integer(pair.as_str().to_owned())),
        Rule::rational => Term::Number(NumberLit::Rational(pair.as_str().to_owned())),
        Rule::real => Term::Number(NumberLit::Real(pair.as_str().to_owned())),
        Rule::distinct_object => Term::DistinctObject(pair.as_str().to_owned()),
        _ => panic!("unsupported term rule: {:?}", pair.as_rule()),
    }
}

fn build_fof_formula(pair: Pair<'_, Rule>) -> Formula {
    let mut inner = pair.into_inner();
    let mut left = build_formula(
        inner
            .next()
            .expect("fof_formula should start with a unit formula"),
    );

    while let Some(operator) = inner.next() {
        let right = build_formula(
            inner
                .next()
                .expect("operator in fof_formula should have rhs"),
        );
        left = combine_binary(left, operator.as_str(), right);
    }

    left
}

fn build_unary_formula(pair: Pair<'_, Rule>) -> Formula {
    let mut inner = pair.into_inner();
    let first = inner.next().expect("unary formula should have content");

    match first.as_rule() {
        Rule::unary_connective => {
            let rhs = inner
                .next()
                .expect("unary connective should be followed by formula");
            Formula::Not(Box::new(build_formula(rhs)))
        }
        _ => panic!("unexpected unary formula shape: {:?}", first.as_rule()),
    }
}

fn build_quantified_formula(pair: Pair<'_, Rule>) -> Formula {
    let mut inner = pair.into_inner();
    let quantifier = inner
        .next()
        .expect("quantified formula should include quantifier")
        .as_str();

    let variables = inner
        .next()
        .expect("quantified formula should include variable list")
        .into_inner()
        .map(|variable| Var {
            name: variable.as_str().to_owned(),
        })
        .collect::<Vec<_>>();

    let body = build_formula(
        inner
            .next()
            .expect("quantified formula should include quantified body"),
    );

    match quantifier {
        "!" => Formula::ForAll(variables, Box::new(body)),
        "?" => Formula::Exists(variables, Box::new(body)),
        _ => panic!("unexpected quantifier: {quantifier}"),
    }
}

fn predicate_from_term_rule(pair: Pair<'_, Rule>) -> Formula {
    let (name, args) = plain_term_parts(pair);
    Formula::Atom(Atom::Predicate { name, args })
}

fn predicate_from_defined_plain_term(pair: Pair<'_, Rule>) -> Formula {
    let (name, args) = defined_plain_term_parts(pair);

    if args.is_empty() {
        if let Symbol::Defined(value) = &name {
            if value == "$true" {
                return Formula::True;
            }
            if value == "$false" {
                return Formula::False;
            }
        }
    }

    Formula::Atom(Atom::Predicate { name, args })
}

fn predicate_from_system_term(pair: Pair<'_, Rule>) -> Formula {
    let (name, args) = system_term_parts(pair);
    Formula::Atom(Atom::Predicate { name, args })
}

fn term_from_plain_term(pair: Pair<'_, Rule>) -> Term {
    let (name, args) = plain_term_parts(pair);
    if args.is_empty() {
        Term::Const(name)
    } else {
        Term::Fun { name, args }
    }
}

fn term_from_defined_plain_term(pair: Pair<'_, Rule>) -> Term {
    let (name, args) = defined_plain_term_parts(pair);
    if args.is_empty() {
        Term::Const(name)
    } else {
        Term::Fun { name, args }
    }
}

fn term_from_system_term(pair: Pair<'_, Rule>) -> Term {
    let (name, args) = system_term_parts(pair);
    if args.is_empty() {
        Term::Const(name)
    } else {
        Term::Fun { name, args }
    }
}

fn plain_term_parts(pair: Pair<'_, Rule>) -> (Symbol, Vec<Term>) {
    let mut inner = pair.into_inner();
    let first = inner.next().expect("plain term should contain identifier");

    match first.as_rule() {
        Rule::functor => {
            let name = symbol_from_functor(first);
            let args = inner.next().map(arguments_from_pair).unwrap_or_default();
            (name, args)
        }
        Rule::constant => {
            let name = symbol_from_constant(first);
            (name, Vec::new())
        }
        _ => panic!("unexpected plain term shape: {:?}", first.as_rule()),
    }
}

fn defined_plain_term_parts(pair: Pair<'_, Rule>) -> (Symbol, Vec<Term>) {
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .expect("defined plain term should contain identifier");

    match first.as_rule() {
        Rule::defined_functor => {
            let name = symbol_from_defined_functor(first);
            let args = inner.next().map(arguments_from_pair).unwrap_or_default();
            (name, args)
        }
        Rule::defined_constant => {
            let name = symbol_from_defined_constant(first);
            (name, Vec::new())
        }
        _ => panic!("unexpected defined plain term shape: {:?}", first.as_rule()),
    }
}

fn system_term_parts(pair: Pair<'_, Rule>) -> (Symbol, Vec<Term>) {
    let mut inner = pair.into_inner();
    let first = inner.next().expect("system term should contain identifier");

    match first.as_rule() {
        Rule::system_functor => {
            let name = symbol_from_system_functor(first);
            let args = inner.next().map(arguments_from_pair).unwrap_or_default();
            (name, args)
        }
        Rule::system_constant => {
            let name = symbol_from_system_constant(first);
            (name, Vec::new())
        }
        _ => panic!("unexpected system term shape: {:?}", first.as_rule()),
    }
}

fn arguments_from_pair(pair: Pair<'_, Rule>) -> Vec<Term> {
    pair.into_inner().map(build_term).collect()
}

fn symbol_from_functor(pair: Pair<'_, Rule>) -> Symbol {
    let atom = pair
        .into_inner()
        .next()
        .expect("functor should contain atomic_word");
    symbol_from_atomic_word(atom)
}

fn symbol_from_constant(pair: Pair<'_, Rule>) -> Symbol {
    let functor = pair
        .into_inner()
        .next()
        .expect("constant should contain functor");
    symbol_from_functor(functor)
}

fn symbol_from_defined_functor(pair: Pair<'_, Rule>) -> Symbol {
    let atom = pair
        .into_inner()
        .next()
        .expect("defined_functor should contain atomic_defined_word");
    symbol_from_atomic_defined_word(atom)
}

fn symbol_from_defined_constant(pair: Pair<'_, Rule>) -> Symbol {
    let defined_functor = pair
        .into_inner()
        .next()
        .expect("defined_constant should contain defined_functor");
    symbol_from_defined_functor(defined_functor)
}

fn symbol_from_system_functor(pair: Pair<'_, Rule>) -> Symbol {
    let atom = pair
        .into_inner()
        .next()
        .expect("system_functor should contain atomic_system_word");
    symbol_from_atomic_system_word(atom)
}

fn symbol_from_system_constant(pair: Pair<'_, Rule>) -> Symbol {
    let system_functor = pair
        .into_inner()
        .next()
        .expect("system_constant should contain system_functor");
    symbol_from_system_functor(system_functor)
}

fn symbol_from_atomic_word(pair: Pair<'_, Rule>) -> Symbol {
    let value = pair
        .into_inner()
        .next()
        .expect("atomic_word should contain concrete token")
        .as_str()
        .to_owned();
    Symbol::User(value)
}

fn symbol_from_atomic_defined_word(pair: Pair<'_, Rule>) -> Symbol {
    let value = pair
        .into_inner()
        .next()
        .expect("atomic_defined_word should contain concrete token")
        .as_str()
        .to_owned();
    Symbol::Defined(value)
}

fn symbol_from_atomic_system_word(pair: Pair<'_, Rule>) -> Symbol {
    let value = pair
        .into_inner()
        .next()
        .expect("atomic_system_word should contain concrete token")
        .as_str()
        .to_owned();
    Symbol::System(value)
}

fn combine_binary(left: Formula, operator: &str, right: Formula) -> Formula {
    match operator {
        "&" => merge_associative(left, right, true),
        "|" => merge_associative(left, right, false),
        "=>" => Formula::Implies(Box::new(left), Box::new(right)),
        _ => panic!("unsupported binary connective: {operator}"),
    }
}

fn merge_associative(left: Formula, right: Formula, is_and: bool) -> Formula {
    let mut formulas = Vec::new();

    match (is_and, left) {
        (true, Formula::And(items)) | (false, Formula::Or(items)) => formulas.extend(items),
        (_, item) => formulas.push(item),
    }

    match (is_and, right) {
        (true, Formula::And(items)) | (false, Formula::Or(items)) => formulas.extend(items),
        (_, item) => formulas.push(item),
    }

    if is_and {
        Formula::And(formulas)
    } else {
        Formula::Or(formulas)
    }
}
