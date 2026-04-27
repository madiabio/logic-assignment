#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Variable(String),
    Constant(String),
    Function(String, Vec<Term>),
}
