use crate::Sequent;
use crate::ast::Formula;
use crate::ast::{Atom, Formula, Symbol, Term};
use std::collections::BTreeSet;

/// Instantiates the outermost variable of a quantified formula.
///
/// Given a list of bound variables `vars` and a formula body `body`,
/// this function:
/// 1. Takes the first bound variable (e.g. `x` in ∀x,y A or ∃x,y A)
/// 2. Substitutes it with a fresh constant term (the "eigenconstant")
/// 3. Rebuilds the formula:
///    - If no variables remain → returns the instantiated body
///    - Otherwise → re-wraps the remaining quantifiers around the new body
///
/// Example:
///   ∀x,y. P(x, y)  →  ∀y. P(a, y)
///
/// This is used for rules like ∀R and ∃L in LK′, where we instantiate
/// one variable at a time.
///
/// Parameters:
/// - `vars`: list of bound variables from the quantifier
/// - `body`: the inner formula being quantified
/// - `replacement_name`: name of the fresh constant used for substitution
/// - `wrap_remaining`: constructor (ForAll or Exists) to rebuild remaining quantifiers
///
/// Returns:
/// - `Some(instantiated_formula)` if at least one variable exists
/// - `None` if `vars` is empty (invalid quantified formula)
///
/// Notes:
/// - Substitution should be capture-avoiding (handled by `substitute_var`)
/// - The replacement term is a constant (eigenconstant), ensuring correctness
///   for rules requiring freshness conditions
fn instantiate_quantified_formula(
    vars: &[crate::ast::Var],
    body: &Formula,
    replacement_name: String,
    wrap_remaining: fn(Vec<crate::ast::Var>, Box<Formula>) -> Formula,
) -> Option<Formula> {
    let (first_var, remaining_vars) = vars.split_first()?;
    let replacement = Term::Const(Symbol::User(replacement_name));
    let instantiated_body = body.substitute_var(&first_var.name, &replacement);

    Some(if remaining_vars.is_empty() {
        instantiated_body
    } else {
        wrap_remaining(remaining_vars.to_vec(), Box::new(instantiated_body))
    })
}

/// Generates a fresh eigenconstant name that does not appear anywhere
/// in the given sequent.
///
/// This is used in rules like ∀R and ∃L, where we must introduce a
/// new constant symbol that is guaranteed to be “fresh” (i.e. it does
/// not occur in the conclusion sequent).
///
/// Strategy:
/// 1. Traverse all formulas on both sides of the sequent
///    and collect every symbol name that is already in use.
/// 2. Generate candidate names in the sequence:
///       a, b, ..., z, a1, b1, ..., z1, a2, ...
/// 3. Return the first candidate not present in the `used` set.
///
/// This ensures:
/// - The freshness condition required by eigenvariable/eigenconstant rules
/// - Deterministic naming (useful for debugging and reproducibility)
///
/// Note:
/// - Freshness is checked syntactically (by name), not semantically.
/// - `collect_formula_symbols` must gather all relevant identifiers
///   (variables, constants, function symbols, predicate symbols) for correctness.
fn fresh_eigenconstant_name(sequent: &Sequent) -> String {
    let mut used = BTreeSet::new();
    for formula in &sequent.left {
        collect_formula_symbols(formula, &mut used);
    }
    for formula in &sequent.right {
        collect_formula_symbols(formula, &mut used);
    }

    // Generate a fresh variable name
    for suffix in 0.. {
        for letter in b'a'..=b'z' {
            let mut candidate = String::from(char::from(letter));
            if suffix > 0 {
                candidate.push_str(&suffix.to_string());
            }
            if !used.contains(&candidate) {
                return candidate;
            }
        }
    }

    unreachable!("fresh eigenconstant generation should always find a name")
}

/// Recursively traverses a formula and collects all symbol names into `used`.
///
/// This function walks the abstract syntax tree of a `Formula` and records
/// every identifier encountered (variables, constants, function symbols,
/// predicate symbols) into the provided `BTreeSet`.
///
/// Behavior by case:
/// - `True` / `False`: no symbols to collect
/// - `Atom`: delegates to `collect_atom_symbols` to gather predicate/function usage
/// - `Not`: recurses into the inner formula
/// - `And` / `Or`: recurses into each subformula
/// - `Implies`: recurses into both antecedent and consequent
/// - `ForAll` / `Exists`:
///     - inserts all bound variable names into `used`
///     - then recurses into the quantified body
///
/// This is used for tasks like generating fresh symbols (e.g. eigenconstants),
/// where we must ensure newly introduced names do not clash with any existing
/// identifiers in the formula.
///
/// Notes:
/// - Collection is purely syntactic (based on names), not semantic.
/// - The set `used` is mutated in place and accumulates results across calls.
/// - This is pretty inefficeint and can probably be improved.
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

/// Collects all symbol names appearing in an atomic formula and inserts them into `used`.
///
/// This function traverses a predicate atom, recording:
/// - the predicate symbol itself (e.g. `P` in `P(a, f(b))`)
/// - all symbols occurring inside its argument terms (constants, function symbols, etc.)
///
/// It delegates traversal of terms to `collect_term_symbols`, ensuring that nested
/// function terms are fully explored.
///
/// Parameters:
/// - `atom`: the atomic formula to inspect
/// - `used`: a set of symbol names that is mutated in-place to include all encountered symbols
///
/// Notes:
/// - Variables are handled at the term level (`collect_term_symbols`)
/// - This function assumes atoms are predicate applications (no equality handling shown)
///
/// Example:
///   Input:  P(a, f(b))
///   Output: used = { "P", "a", "f", "b" }
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

/// Collects all symbol names occurring within a term and inserts them into `used`.
///
/// This function recursively traverses a term and records:
/// - variable names (e.g. `x`)
/// - constant symbols (e.g. `a`)
/// - function symbols (e.g. `f` in `f(x, a)`)
///
/// For compound terms (functions), it:
/// 1. Records the function symbol itself
/// 2. Recursively processes all argument terms
///
/// Parameters:
/// - `term`: the term to inspect
/// - `used`: a set of symbol names that is mutated in-place to include all encountered symbols
///
/// Notes:
/// - Numeric literals and distinct objects are ignored, as they do not contribute
///   user-defined or logical symbol names.
/// - This function ensures full traversal of nested term structure.
///
/// Example:
///   Input:  f(x, g(a))
///   Output: used = { "f", "x", "g", "a" }
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

/// Inserts the name of a symbol into the `used` set.
///
/// This function handles all variants of `Symbol` uniformly, extracting
/// the underlying string value and recording it. It is used as a helper
/// during traversal of formulas and terms to collect all symbol names
/// present in a sequent.
///
/// Parameters:
/// - `symbol`: the symbol whose name should be recorded
/// - `used`: a set of symbol names that is mutated in-place
///
/// Notes:
/// - This function does not distinguish between user-defined, defined,
///   or system symbols; all are treated the same for collection purposes.
/// - Deduplication is handled automatically by the `BTreeSet`.
///
/// Example:
///   Input:  Symbol::User("f")
///   Output: used = { "f" }
fn collect_symbol(symbol: &Symbol, used: &mut BTreeSet<String>) {
    match symbol {
        Symbol::User(value) | Symbol::Defined(value) | Symbol::System(value) => {
            used.insert(value.clone());
        }
    }
}
