use std::sync::{Arc, atomic::AtomicBool};
use theorem_prover::ast::Formula;
use theorem_prover::{
    ProofOptions, ProofStatus, SearchEngine, Sequent, UnknownReason,
    prove, prove_with_cancel,
};

fn priority_options() -> ProofOptions {
    ProofOptions { engine: SearchEngine::Priority, ..ProofOptions::default() }
}

fn priority_id_options() -> ProofOptions {
    ProofOptions { engine: SearchEngine::PriorityId, ..ProofOptions::default() }
}

#[test]
fn priority_proves_identity_sequent() {
    let sequent = Sequent {
        left: vec![Formula::atom("p")],
        right: vec![Formula::atom("p")],
    };
    assert_eq!(prove(&sequent, priority_options()).status, ProofStatus::Provable);
}

#[test]
fn priority_returns_not_provable_for_atomic_dead_end() {
    let sequent = Sequent {
        left: vec![Formula::atom("p")],
        right: vec![Formula::atom("q")],
    };
    assert_eq!(prove(&sequent, priority_options()).status, ProofStatus::NotProvable);
}

#[test]
fn priority_id_proves_identity_sequent() {
    let sequent = Sequent {
        left: vec![Formula::atom("p")],
        right: vec![Formula::atom("p")],
    };
    assert_eq!(prove(&sequent, priority_id_options()).status, ProofStatus::Provable);
}

#[test]
fn priority_id_returns_not_provable_for_atomic_dead_end() {
    let sequent = Sequent {
        left: vec![Formula::atom("p")],
        right: vec![Formula::atom("q")],
    };
    assert_eq!(prove(&sequent, priority_id_options()).status, ProofStatus::NotProvable);
}

#[test]
fn priority_agrees_with_naive_on_propositional_tautology() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![Formula::atom("a"), Formula::atom("b")])],
        right: vec![Formula::or(vec![Formula::atom("a"), Formula::atom("c")])],
    };
    let naive = prove(&sequent, ProofOptions::default()).status;
    let priority = prove(&sequent, priority_options()).status;
    assert_eq!(naive, priority);
}

#[test]
fn priority_id_agrees_with_id_on_propositional_tautology() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![Formula::atom("a"), Formula::atom("b")])],
        right: vec![Formula::or(vec![Formula::atom("a"), Formula::atom("c")])],
    };
    let id = prove(
        &sequent,
        ProofOptions { engine: SearchEngine::IterativeDeepening, ..ProofOptions::default() },
    ).status;
    let priority_id = prove(&sequent, priority_id_options()).status;
    assert_eq!(id, priority_id);
}

#[test]
fn priority_id_respects_cancellation() {
    let cancel = Arc::new(AtomicBool::new(true));
    let sequent = Sequent {
        left: vec![Formula::atom("p")],
        right: vec![Formula::atom("p")],
    };
    let result = prove_with_cancel(&sequent, priority_id_options(), &cancel);
    assert_eq!(result.status, ProofStatus::Cancelled);
}

#[test]
fn priority_id_respects_max_steps() {
    let left = (0..20)
        .map(|i| Formula::or(vec![Formula::atom(&format!("p{i}")), Formula::atom(&format!("q{i}"))]))
        .collect();
    let sequent = Sequent { left, right: vec![Formula::atom("goal")] };
    let options = ProofOptions {
        engine: SearchEngine::PriorityId,
        max_steps: 1,
        ..ProofOptions::default()
    };
    let result = prove(&sequent, options);
    assert_eq!(result.status, ProofStatus::Unknown);
    assert_eq!(result.unknown_reason, Some(UnknownReason::MaxStepsExceeded));
}
