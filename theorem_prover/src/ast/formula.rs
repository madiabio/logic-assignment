use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formula {
    True,
    False,
    Atom(Atom),
    Not(Box<Formula>),
    And(Vec<Formula>),
    Or(Vec<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    ForAll(Vec<Var>, Box<Formula>),
    Exists(Vec<Var>, Box<Formula>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    Predicate { name: Symbol, args: Vec<Term> },
}

use crate::ast::term::{Symbol, Term, Var};

impl Formula {
    pub fn atom(name: &str) -> Self {
        Self::predicate(name, Vec::new())
    }

    pub fn predicate(name: &str, args: Vec<Term>) -> Self {
        Self::Atom(Atom::Predicate {
            name: Symbol::User(name.to_owned()),
            args,
        })
    }

    pub fn not(inner: Formula) -> Self {
        Self::Not(Box::new(inner))
    }

    pub fn and(items: Vec<Formula>) -> Self {
        Self::And(items)
    }

    pub fn or(items: Vec<Formula>) -> Self {
        Self::Or(items)
    }

    pub fn implies(left: Formula, right: Formula) -> Self {
        Self::Implies(Box::new(left), Box::new(right))
    }

    fn precedence(&self) -> u8 {
        match self {
            Formula::True | Formula::False | Formula::Atom(_) => 3,
            Formula::ForAll(_, _) | Formula::Exists(_, _) | Formula::Not(_) => 2,
            Formula::And(_) | Formula::Or(_) | Formula::Implies(_, _) => 1,
        }
    }

    fn fmt_with_parent(
        &self,
        f: &mut fmt::Formatter<'_>,
        parent_precedence: Option<u8>,
        wrap_on_equal: bool,
    ) -> fmt::Result {
        let my_precedence = self.precedence();
        let needs_parens = match parent_precedence {
            Some(parent) if my_precedence < parent => true,
            Some(parent) if wrap_on_equal && my_precedence == parent => true,
            _ => false,
        };

        if needs_parens {
            f.write_str("(")?;
        }

        match self {
            Formula::True => f.write_str("⊤")?,
            Formula::False => f.write_str("⊥")?,
            Formula::Atom(atom) => write!(f, "{atom}")?,
            Formula::Not(inner) => {
                f.write_str("¬")?;
                inner.fmt_with_parent(f, Some(my_precedence), false)?;
            }
            Formula::And(items) => {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        f.write_str(" ∧ ")?;
                    }
                    item.fmt_with_parent(
                        f,
                        Some(my_precedence),
                        item.is_binary() && !matches!(item, Formula::And(_)),
                    )?;
                }
            }
            Formula::Or(items) => {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        f.write_str(" ∨ ")?;
                    }
                    item.fmt_with_parent(
                        f,
                        Some(my_precedence),
                        item.is_binary() && !matches!(item, Formula::Or(_)),
                    )?;
                }
            }
            Formula::Implies(left, right) => {
                left.fmt_with_parent(f, Some(my_precedence), false)?;
                f.write_str(" ⇒ ")?;
                right.fmt_with_parent(f, Some(my_precedence), right.is_binary())?;
            }
            Formula::ForAll(vars, body) => {
                f.write_str("∀")?;
                for (index, var) in vars.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{var}")?;
                }
                f.write_str(". ")?;
                body.fmt_with_parent(f, Some(my_precedence), false)?;
            }
            Formula::Exists(vars, body) => {
                f.write_str("∃")?;
                for (index, var) in vars.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{var}")?;
                }
                f.write_str(". ")?;
                body.fmt_with_parent(f, Some(my_precedence), false)?;
            }
        }

        if needs_parens {
            f.write_str(")")?;
        }

        Ok(())
    }

    fn is_binary(&self) -> bool {
        matches!(
            self,
            Formula::And(_) | Formula::Or(_) | Formula::Implies(_, _)
        )
    }

    pub fn substitute_var(&self, variable_name: &str, replacement: &Term) -> Self {
        match self {
            Formula::True | Formula::False => self.clone(),
            Formula::Atom(Atom::Predicate { name, args }) => Formula::Atom(Atom::Predicate {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|arg| arg.substitute_var(variable_name, replacement))
                    .collect(),
            }),
            Formula::Not(inner) => Formula::Not(Box::new(
                inner.substitute_var(variable_name, replacement),
            )),
            Formula::And(items) => Formula::And(
                items
                    .iter()
                    .map(|item| item.substitute_var(variable_name, replacement))
                    .collect(),
            ),
            Formula::Or(items) => Formula::Or(
                items
                    .iter()
                    .map(|item| item.substitute_var(variable_name, replacement))
                    .collect(),
            ),
            Formula::Implies(left, right) => Formula::Implies(
                Box::new(left.substitute_var(variable_name, replacement)),
                Box::new(right.substitute_var(variable_name, replacement)),
            ),
            Formula::ForAll(vars, body) => {
                if vars.iter().any(|var| var.name == variable_name) {
                    self.clone()
                } else {
                    Formula::ForAll(
                        vars.clone(),
                        Box::new(body.substitute_var(variable_name, replacement)),
                    )
                }
            }
            Formula::Exists(vars, body) => {
                if vars.iter().any(|var| var.name == variable_name) {
                    self.clone()
                } else {
                    Formula::Exists(
                        vars.clone(),
                        Box::new(body.substitute_var(variable_name, replacement)),
                    )
                }
            }
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Predicate { name, args } => {
                write!(f, "{name}")?;
                if !args.is_empty() {
                    f.write_str("(")?;
                    for (index, arg) in args.iter().enumerate() {
                        if index > 0 {
                            f.write_str(", ")?;
                        }
                        write!(f, "{arg}")?;
                    }
                    f.write_str(")")?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_parent(f, None, false)
    }
}
