use clap::Parser;
use crate::cli::args::{CliOptions, Command, ProveCommand, PersistOpt};

/// Helper to parse a `prove` subcommand from argument list.
fn parse_prove(args: &[&str]) -> ProveCommand {
    let mut full_args = vec!["theorem_prover"];
    full_args.extend_from_slice(args);
    match CliOptions::parse_from(full_args).command {
        Command::Prove(cmd) => cmd,
        _ => panic!("expected Prove command"),
    }
}

#[test]
fn persist_false_parses_correctly() {
    let prove = parse_prove(&["prove", "--problem-class", "provable", "--persist", "false"]);
    assert!(matches!(prove.persist, Some(PersistOpt::Disabled)));
}

#[test]
fn persist_path_parses_correctly() {
    let prove = parse_prove(&["prove", "--problem-class", "provable", "--persist", "./results.db"]);
    assert!(matches!(prove.persist, Some(PersistOpt::Path(p)) if p == "./results.db"));
}

#[test]
fn run_label_parses_correctly() {
    let prove = parse_prove(&["prove", "--problem-class", "provable", "--run-label", "my-experiment"]);
    assert_eq!(prove.run_label, Some("my-experiment".to_string()));
}

#[test]
fn persist_absent_is_none() {
    let prove = parse_prove(&["prove", "--problem-class", "provable"]);
    assert_eq!(prove.persist, None);
}

#[test]
fn run_label_absent_is_none() {
    let prove = parse_prove(&["prove", "--problem-class", "provable"]);
    assert_eq!(prove.run_label, None);
}
