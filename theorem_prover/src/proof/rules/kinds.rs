#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleMatch {
    pub rule: Rule,
    pub side: Side,
    pub index: usize,
}
