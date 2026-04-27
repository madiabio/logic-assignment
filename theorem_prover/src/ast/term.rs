#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Var(Var),
    Const(Symbol),
    Fun { name: Symbol, args: Vec<Term> },
    Number(NumberLit),
    DistinctObject(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberLit {
    Integer(String),
    Rational(String),
    Real(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    User(String),
    Defined(String),
    System(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    pub name: String,
}
