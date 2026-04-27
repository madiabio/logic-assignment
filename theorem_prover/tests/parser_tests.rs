use std::fs;
use theorem_prover::{parse_problem, parse_tptp};

fn read_problem(path: &str) -> String {
    fs::read_to_string(path).expect("failed to read problem file")
}

#[test]
fn parses_syn000_assignment_subset() {
    let input = read_problem("../tptp_problems/SYN000+1.p");
    parse_tptp(&input).expect("SYN000+1.p should parse");
}

#[test]
fn parses_syn001_biconditional_problem() {
    let input = read_problem("../tptp_problems/SYN001+1.p");
    parse_tptp(&input).expect("SYN001+1.p should parse");
}

#[test]
fn parses_later_quantified_problem() {
    let input = read_problem("../tptp_problems/SYN036+1.p");
    parse_tptp(&input).expect("SYN036+1.p should parse");
}

#[test]
fn separates_premises_from_conjecture() {
    let input = read_problem("../tptp_problems/SYN000+1.p");
    let parsed = parse_problem(&input).expect("SYN000+1.p should parse into problem structure");

    assert_eq!(parsed.premises.len(), 8);
    assert_eq!(
        parsed
            .premises
            .iter()
            .map(|record| record.role.as_str())
            .collect::<Vec<_>>(),
        vec![
            "axiom",
            "axiom",
            "axiom",
            "axiom",
            "axiom",
            "axiom",
            "axiom",
            "hypothesis"
        ]
    );
    assert_eq!(
        parsed.conjecture.as_ref().map(|record| record.name.as_str()),
        Some("role_conjecture")
    );
}
