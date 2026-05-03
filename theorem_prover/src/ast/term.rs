use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Var(Var),
    Const(Symbol),
    Fun { name: Symbol, args: Vec<Term> },
    Number(NumberLit),
    DistinctObject(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumberLit {
    Integer(String),
    Rational(String),
    Real(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    User(String),
    Defined(String),
    System(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Var {
    pub name: String,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::User(value) | Symbol::Defined(value) | Symbol::System(value) => {
                f.write_str(value)
            }
        }
    }
}

impl fmt::Display for NumberLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberLit::Integer(value) | NumberLit::Rational(value) | NumberLit::Real(value) => {
                f.write_str(value)
            }
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Var(var) => write!(f, "{var}"),
            Term::Const(symbol) => write!(f, "{symbol}"),
            Term::Fun { name, args } => {
                write!(f, "{name}(")?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                f.write_str(")")
            }
            Term::Number(number) => write!(f, "{number}"),
            Term::DistinctObject(value) => f.write_str(value),
        }
    }
}

impl Term {
    pub fn substitute_var(&self, variable_name: &str, replacement: &Term) -> Self {
        match self {
            Term::Var(var) if var.name == variable_name => replacement.clone(),
            Term::Var(_) => self.clone(),
            Term::Const(_) | Term::Number(_) | Term::DistinctObject(_) => self.clone(),
            Term::Fun { name, args } => Term::Fun {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|arg| arg.substitute_var(variable_name, replacement))
                    .collect(),
            },
        }
    }
}
