#![allow(dead_code)]

use crate::ast::{Formula, Term};

pub fn term_name(term: &Term) -> Option<&str> {
    match term {
        Term::Variable(name) | Term::Constant(name) => Some(name),
        Term::Function(name, _) => Some(name),
    }
}

pub fn formula_is_atomic(formula: &Formula) -> bool {
    matches!(formula, Formula::Atomic(_))
}
