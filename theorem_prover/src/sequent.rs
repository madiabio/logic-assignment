use crate::ast::Formula;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequent {
    pub left: Vec<Formula>, // Gamma
    pub right: Vec<Formula>, // Delta
}
