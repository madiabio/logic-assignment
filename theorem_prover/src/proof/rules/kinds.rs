//! Core rule, side, and match types used by proof search.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A sequent-calculus rule supported by the prover.
pub enum Rule {
    Id,
    TopR,
    BottomL,
    AndL,
    AndR,
    OrL,
    OrR,
    ImpliesL,
    ImpliesR,
    NotL,
    NotR,
    ForAllL,
    ForAllR,
    ExistsL,
    ExistsR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The side of the sequent on which a rule was matched.
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Identifies a specific rule occurrence within a sequent.
pub struct RuleMatch {
    pub rule: Rule,
    pub side: Side,
    pub index: usize,
}
