use crate::ast::term::Term;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formula {
    Atomic(Term),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    Iff(Box<Formula>, Box<Formula>),
    ForAll(Vec<String>, Box<Formula>),
    Exists(Vec<String>, Box<Formula>),
}
