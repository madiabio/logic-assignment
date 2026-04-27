#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formula {
    True,
    False,
    Atom(Atom),
    Not(Box<Formula>),
    And(Vec<Formula>),
    Or(Vec<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    Iff(Box<Formula>, Box<Formula>),
    ForAll(Vec<Var>, Box<Formula>),
    Exists(Vec<Var>, Box<Formula>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    Predicate { name: Symbol, args: Vec<Term> },
    Equality(Term, Term),
    Inequality(Term, Term),
}

use crate::ast::term::{Symbol, Term, Var};
