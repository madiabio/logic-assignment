use std::collections::BTreeSet;

use crate::Sequent;
use crate::proof::quantifier::terms::collect_sequent_symbols;

pub(crate) fn fresh_eigenconstant_name(sequent: &Sequent) -> String {
    let mut used = BTreeSet::new();
    collect_sequent_symbols(sequent, &mut used);

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

pub(crate) fn fresh_branch_term_name(sequent: &Sequent) -> String {
    fresh_name_avoiding_sequent(sequent, "w")
}

fn fresh_name_avoiding_sequent(sequent: &Sequent, prefix: &str) -> String {
    let mut used = BTreeSet::new();
    collect_sequent_symbols(sequent, &mut used);

    if !prefix.is_empty() && !used.contains(prefix) {
        return prefix.to_owned();
    }

    for suffix in 0.. {
        for letter in b'a'..=b'z' {
            let mut candidate = prefix.to_owned();
            candidate.push(char::from(letter));
            if suffix > 0 {
                candidate.push_str(&suffix.to_string());
            }
            if !used.contains(&candidate) {
                return candidate;
            }
        }
    }

    unreachable!("fresh branch-term generation should always find a name")
}
