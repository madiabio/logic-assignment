//! Term and symbol collection helpers used by quantifier scheduling.

use std::collections::BTreeSet;

use crate::Sequent;
use crate::ast::{Atom, Formula, Symbol, Term};

/// Collects distinct non-variable terms that are visible in the sequent.
pub(crate) fn visible_terms_in_sequent(sequent: &Sequent) -> Vec<Term> {
    let mut seen = BTreeSet::new();
    let mut terms = Vec::new();

    for formula in &sequent.left {
        collect_visible_formula_terms(formula, &mut seen, &mut terms);
    }
    for formula in &sequent.right {
        collect_visible_formula_terms(formula, &mut seen, &mut terms);
    }

    terms
}

/// Collects every symbol name that appears anywhere in the sequent.
pub(crate) fn collect_sequent_symbols(sequent: &Sequent, used: &mut BTreeSet<String>) {
    for formula in &sequent.left {
        collect_formula_symbols(formula, used);
    }
    for formula in &sequent.right {
        collect_formula_symbols(formula, used);
    }
}

/// Walks a formula and records all symbol names that it contains.
fn collect_formula_symbols(formula: &Formula, used: &mut BTreeSet<String>) {
    match formula {
        Formula::True | Formula::False => {}
        Formula::Atom(atom) => collect_atom_symbols(atom, used),
        Formula::Not(inner) => collect_formula_symbols(inner, used),
        Formula::And(items) | Formula::Or(items) => {
            for item in items {
                collect_formula_symbols(item, used);
            }
        }
        Formula::Implies(left, right) => {
            collect_formula_symbols(left, used);
            collect_formula_symbols(right, used);
        }
        Formula::ForAll(vars, body) | Formula::Exists(vars, body) => {
            for var in vars {
                used.insert(var.name.clone());
            }
            collect_formula_symbols(body, used);
        }
    }
}

/// Records the symbols mentioned by an atomic formula.
fn collect_atom_symbols(atom: &Atom, used: &mut BTreeSet<String>) {
    match atom {
        Atom::Predicate { name, args } => {
            collect_symbol(name, used);
            for arg in args {
                collect_term_symbols(arg, used);
            }
        }
    }
}

/// Records every symbol reachable from a term.
fn collect_term_symbols(term: &Term, used: &mut BTreeSet<String>) {
    match term {
        Term::Var(var) => {
            used.insert(var.name.clone());
        }
        Term::Const(symbol) => collect_symbol(symbol, used),
        Term::Fun { name, args } => {
            collect_symbol(name, used);
            for arg in args {
                collect_term_symbols(arg, used);
            }
        }
        Term::Number(_) | Term::DistinctObject(_) => {}
    }
}

/// Records a symbol's printable name.
fn collect_symbol(symbol: &Symbol, used: &mut BTreeSet<String>) {
    match symbol {
        Symbol::User(value) | Symbol::Defined(value) | Symbol::System(value) => {
            used.insert(value.clone());
        }
    }
}

/// Walks a formula and appends visible non-variable terms in encounter order.
fn collect_visible_formula_terms(
    formula: &Formula,
    seen: &mut BTreeSet<String>,
    terms: &mut Vec<Term>,
) {
    match formula {
        Formula::True | Formula::False => {}
        Formula::Atom(atom) => collect_visible_atom_terms(atom, seen, terms),
        Formula::Not(inner) => collect_visible_formula_terms(inner, seen, terms),
        Formula::And(items) | Formula::Or(items) => {
            for item in items {
                collect_visible_formula_terms(item, seen, terms);
            }
        }
        Formula::Implies(left, right) => {
            collect_visible_formula_terms(left, seen, terms);
            collect_visible_formula_terms(right, seen, terms);
        }
        Formula::ForAll(_, body) | Formula::Exists(_, body) => {
            collect_visible_formula_terms(body, seen, terms);
        }
    }
}

/// Appends visible terms from an atomic predicate's arguments.
fn collect_visible_atom_terms(atom: &Atom, seen: &mut BTreeSet<String>, terms: &mut Vec<Term>) {
    match atom {
        Atom::Predicate { args, .. } => {
            for arg in args {
                collect_visible_term(arg, seen, terms);
            }
        }
    }
}

/// Adds a term once, recursing into function arguments before recording the term itself.
fn collect_visible_term(term: &Term, seen: &mut BTreeSet<String>, terms: &mut Vec<Term>) {
    if matches!(term, Term::Var(_)) {
        return;
    }

    if let Term::Fun { args, .. } = term {
        for arg in args {
            collect_visible_term(arg, seen, terms);
        }
    }

    let key = term.to_string();
    if seen.insert(key) {
        terms.push(term.clone());
    }
}
