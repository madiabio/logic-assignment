// Implements the logic to apply a rule on a sequent.
use std::collections::BTreeSet;

use crate::ast::{Atom, Formula, Symbol, Term};
use crate::proof::rules::Rule;
use crate::proof::rules::RuleMatch;
use crate::proof::sequent::Sequent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleApplication {
    Closed,
    Premises(Vec<Sequent>),
    NotImplemented,
    Error,
}

pub fn apply_rule(sequent: &Sequent, rule_match: &RuleMatch) -> RuleApplication {
    match rule_match.rule {
        // All of these rules close a branch.
        Rule::Id | Rule::TopR | Rule::BottomL => RuleApplication::Closed,

        // These are not branch closing, and don't create a new branch
        Rule::AndL => apply_and_l(sequent, rule_match.index),
        Rule::OrR => apply_or_r(sequent, rule_match.index),
        Rule::ImpliesR => apply_implies_r(sequent, rule_match.index),
        Rule::NotL => apply_not_l(sequent, rule_match.index),
        Rule::NotR => apply_not_r(sequent, rule_match.index),
        Rule::ForAllR => apply_forall_r(sequent, rule_match.index),
        Rule::ExistsL => apply_exists_l(sequent, rule_match.index),

        // These are branch closing, and create new branch
        Rule::AndR => apply_and_r(sequent, rule_match.index),
        Rule::OrL => apply_or_l(sequent, rule_match.index),
        Rule::ImpliesL => apply_implies_l(sequent, rule_match.index),

        _ => RuleApplication::NotImplemented,
    }
}

fn apply_and_l(sequent: &Sequent, index: usize) -> RuleApplication {
    // TODO: Check the NotImplemented stuff, it probs shouldnt be returning NotImplemented
    let Some(Formula::And(items)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left[..index].iter().cloned());

    // This prover keeps conjunctions in a binary step form, so an n-ary
    // conjunction is peeled from the front and the remainder stays grouped.
    left.push(items[0].clone());
    if items.len() == 2 {
        left.push(items[1].clone());
    } else {
        left.push(Formula::And(items[1..].to_vec()));
    }

    left.extend(sequent.left[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left,
        right: sequent.right.clone(),
    }])
}

fn apply_and_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::And(items)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    // Gamma |- A /\ B, Delta branches into Gamma |- A, Delta and
    // Gamma |- B, Delta. For n-ary conjunctions, keep the tail grouped so
    // repeated AndR applications continue shrinking the formula.
    let mut first_right = Vec::with_capacity(sequent.right.len());
    first_right.extend(sequent.right[..index].iter().cloned());
    first_right.push(items[0].clone());
    first_right.extend(sequent.right[index + 1..].iter().cloned());

    let mut second_right = Vec::with_capacity(sequent.right.len());
    second_right.extend(sequent.right[..index].iter().cloned());
    if items.len() == 2 {
        second_right.push(items[1].clone());
    } else {
        second_right.push(Formula::And(items[1..].to_vec()));
    }
    second_right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![
        Sequent {
            left: sequent.left.clone(),
            right: first_right,
        },
        Sequent {
            left: sequent.left.clone(),
            right: second_right,
        },
    ])
}

fn apply_or_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Or(items)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    // Gamma, A \/ B |- Delta branches into Gamma, A |- Delta and
    // Gamma, B |- Delta. For n-ary disjunctions, keep the tail grouped so
    // repeated OrL applications continue shrinking the formula.
    let mut first_left = Vec::with_capacity(sequent.left.len());
    first_left.extend(sequent.left[..index].iter().cloned());
    first_left.push(items[0].clone());
    first_left.extend(sequent.left[index + 1..].iter().cloned());

    let mut second_left = Vec::with_capacity(sequent.left.len());
    second_left.extend(sequent.left[..index].iter().cloned());
    if items.len() == 2 {
        second_left.push(items[1].clone());
    } else {
        second_left.push(Formula::Or(items[1..].to_vec()));
    }
    second_left.extend(sequent.left[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![
        Sequent {
            left: first_left,
            right: sequent.right.clone(),
        },
        Sequent {
            left: second_left,
            right: sequent.right.clone(),
        },
    ])
}

fn apply_or_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Or(items)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    if items.len() < 2 {
        return RuleApplication::Error;
    }

    let mut right = Vec::with_capacity(sequent.right.len() + 1);
    right.extend(sequent.right[..index].iter().cloned());

    // Mirror AndL on the right: split off one disjunct and keep the tail as a
    // single formula so repeated rule applications continue reducing it.
    right.push(items[0].clone());
    if items.len() == 2 {
        right.push(items[1].clone());
    } else {
        right.push(Formula::Or(items[1..].to_vec()));
    }

    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left: sequent.left.clone(),
        right,
    }])
}

fn apply_implies_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Implies(left_formula, right_formula)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left.iter().cloned());
    // Gamma |- A -> B becomes Gamma, A |- B.
    left.push((**left_formula).clone());

    let mut right = Vec::with_capacity(sequent.right.len());
    right.extend(sequent.right[..index].iter().cloned());
    right.push((**right_formula).clone());
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent { left, right }])
}

fn apply_implies_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Implies(left_formula, right_formula)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let mut left_without_implication = Vec::with_capacity(sequent.left.len().saturating_sub(1));
    left_without_implication.extend(sequent.left[..index].iter().cloned());
    left_without_implication.extend(sequent.left[index + 1..].iter().cloned());

    let mut first_right = Vec::with_capacity(sequent.right.len() + 1);
    first_right.extend(sequent.right.iter().cloned());
    // Gamma, A -> B |- Delta becomes Gamma |- Delta, A.
    first_right.push((**left_formula).clone());

    let mut second_left = Vec::with_capacity(left_without_implication.len() + 1);
    second_left.extend(left_without_implication.iter().cloned());
    // The second branch keeps Gamma and adds the consequent B on the left.
    second_left.push((**right_formula).clone());

    RuleApplication::Premises(vec![
        Sequent {
            left: left_without_implication,
            right: first_right,
        },
        Sequent {
            left: second_left,
            right: sequent.right.clone(),
        },
    ])
}

fn apply_not_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Not(inner)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len().saturating_sub(1));
    left.extend(sequent.left[..index].iter().cloned());
    left.extend(sequent.left[index + 1..].iter().cloned());

    let mut right = Vec::with_capacity(sequent.right.len() + 1);
    right.extend(sequent.right.iter().cloned());
    // Gamma, ~A |- Delta becomes Gamma |- Delta, A.
    right.push((**inner).clone());

    RuleApplication::Premises(vec![Sequent { left, right }])
}

fn apply_not_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Not(inner)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len() + 1);
    left.extend(sequent.left.iter().cloned());
    // Gamma |- ~A becomes Gamma, A |- by moving the negated formula left.
    left.push((**inner).clone());

    let mut right = Vec::with_capacity(sequent.right.len().saturating_sub(1));
    right.extend(sequent.right[..index].iter().cloned());
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent { left, right }])
}

fn apply_forall_r(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::ForAll(vars, body)) = sequent.right.get(index) else {
        return RuleApplication::Error;
    };

    let Some(replacement_formula) = instantiate_quantified_formula(
        vars,
        body,
        fresh_eigenconstant_name(sequent),
        Formula::ForAll,
    ) else {
        return RuleApplication::Error;
    };

    let mut right = Vec::with_capacity(sequent.right.len());
    right.extend(sequent.right[..index].iter().cloned());
    right.push(replacement_formula);
    right.extend(sequent.right[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left: sequent.left.clone(),
        right,
    }])
}

fn apply_exists_l(sequent: &Sequent, index: usize) -> RuleApplication {
    let Some(Formula::Exists(vars, body)) = sequent.left.get(index) else {
        return RuleApplication::Error;
    };

    let Some(replacement_formula) = instantiate_quantified_formula(
        vars,
        body,
        fresh_eigenconstant_name(sequent),
        Formula::Exists,
    ) else {
        return RuleApplication::Error;
    };

    let mut left = Vec::with_capacity(sequent.left.len());
    left.extend(sequent.left[..index].iter().cloned());
    left.push(replacement_formula);
    left.extend(sequent.left[index + 1..].iter().cloned());

    RuleApplication::Premises(vec![Sequent {
        left,
        right: sequent.right.clone(),
    }])
}

fn instantiate_quantified_formula(
    vars: &[crate::ast::Var],
    body: &Formula,
    replacement_name: String,
    wrap_remaining: fn(Vec<crate::ast::Var>, Box<Formula>) -> Formula,
) -> Option<Formula> {
    let (first_var, remaining_vars) = vars.split_first()?;
    let replacement = Term::Const(Symbol::User(replacement_name));
    let instantiated_body = body.substitute_var(&first_var.name, &replacement);

    Some(if remaining_vars.is_empty() {
        instantiated_body
    } else {
        wrap_remaining(remaining_vars.to_vec(), Box::new(instantiated_body))
    })
}

fn fresh_eigenconstant_name(sequent: &Sequent) -> String {
    let mut used = BTreeSet::new();
    for formula in &sequent.left {
        collect_formula_symbols(formula, &mut used);
    }
    for formula in &sequent.right {
        collect_formula_symbols(formula, &mut used);
    }

    for suffix in 0.. {
        for letter in b'a'..=b'z' {
            let mut candidate = String::from(char::from(letter));
            if suffix > 0 {
                candidate.push_str(&suffix.to_string());
            }
            if !used.contains(&candidate) {
                return candidate;
            }
        }
    }

    unreachable!("fresh eigenconstant generation should always find a name")
}

fn collect_formula_symbols(formula: &Formula, used: &mut BTreeSet<String>) {
    match formula {
        Formula::True | Formula::False => {}
        Formula::Atom(atom) => collect_atom_symbols(atom, used),
        Formula::Not(inner) => collect_formula_symbols(inner, used),
        Formula::And(items) | Formula::Or(items) => {
            for item in items {
                collect_formula_symbols(item, used);
            }
        }
        Formula::Implies(left, right) => {
            collect_formula_symbols(left, used);
            collect_formula_symbols(right, used);
        }
        Formula::ForAll(vars, body) | Formula::Exists(vars, body) => {
            for var in vars {
                used.insert(var.name.clone());
            }
            collect_formula_symbols(body, used);
        }
    }
}

fn collect_atom_symbols(atom: &Atom, used: &mut BTreeSet<String>) {
    match atom {
        Atom::Predicate { name, args } => {
            collect_symbol(name, used);
            for arg in args {
                collect_term_symbols(arg, used);
            }
        }
    }
}

fn collect_term_symbols(term: &Term, used: &mut BTreeSet<String>) {
    match term {
        Term::Var(var) => {
            used.insert(var.name.clone());
        }
        Term::Const(symbol) => collect_symbol(symbol, used),
        Term::Fun { name, args } => {
            collect_symbol(name, used);
            for arg in args {
                collect_term_symbols(arg, used);
            }
        }
        Term::Number(_) | Term::DistinctObject(_) => {}
    }
}

fn collect_symbol(symbol: &Symbol, used: &mut BTreeSet<String>) {
    match symbol {
        Symbol::User(value) | Symbol::Defined(value) | Symbol::System(value) => {
            used.insert(value.clone());
        }
    }
}
