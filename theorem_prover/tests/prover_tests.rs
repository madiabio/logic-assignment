use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use theorem_prover::ast::{Formula, Symbol, Term, Var};
use theorem_prover::{
    ProofOptions, ProofResult, ProofStatus, Sequent, parse_problem, prove, prove_with_cancel,
};

fn predicate_formula(name: &str) -> Formula {
    Formula::atom(name)
}

fn var(name: &str) -> Var {
    Var {
        name: name.to_owned(),
    }
}

fn variable(name: &str) -> Term {
    Term::Var(var(name))
}

fn constant(name: &str) -> Term {
    Term::Const(Symbol::User(name.to_owned()))
}

fn predicate_formula_with_args(name: &str, args: Vec<Term>) -> Formula {
    Formula::predicate(name, args)
}

fn left_disjunction_timeout_sequent(width: usize) -> Sequent {
    let left = (0..width)
        .map(|index| {
            Formula::or(vec![
                predicate_formula(&format!("p{index}")),
                predicate_formula(&format!("q{index}")),
            ])
        })
        .collect();

    Sequent {
        left,
        right: vec![predicate_formula("goal")],
    }
}

fn default_options() -> ProofOptions {
    ProofOptions::default()
}

#[test]
fn prove_returns_not_provable_for_atomic_dead_end_sequent() {
    let sequent = Sequent {
        left: vec![predicate_formula("p"), predicate_formula("q")],
        right: vec![predicate_formula("r")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(
        result,
        ProofResult {
            status: ProofStatus::NotProvable,
        }
    );
}

#[test]
fn prove_returns_not_provable_for_empty_left_atomic_goal() {
    let sequent = Sequent {
        left: Vec::new(),
        right: vec![predicate_formula("goal")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_not_provable_after_applying_left_connective_rule() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("r")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_not_provable_when_andl_cannot_expose_identity() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
        right: vec![predicate_formula("goal")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_provable_when_andl_exposes_identity() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("p")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_provable_when_orl_exposes_identity_on_both_branches() {
    let sequent = Sequent {
        left: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("p"), predicate_formula("q")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_not_provable_when_only_one_orl_branch_closes() {
    let sequent = Sequent {
        left: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("p")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_reduces_multiway_orl_recursively() {
    let sequent = Sequent {
        left: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
        right: vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_provable_when_andr_exposes_identity_on_both_branches() {
    let sequent = Sequent {
        left: vec![predicate_formula("p"), predicate_formula("q")],
        right: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_not_provable_when_only_one_andr_branch_closes() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_reduces_multiway_andr_recursively() {
    let sequent = Sequent {
        left: vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ],
        right: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_not_provable_after_applying_right_connective_rule() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::or(vec![
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_not_provable_when_orr_cannot_expose_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("source")],
        right: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
            predicate_formula("r"),
        ])],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_provable_when_orr_exposes_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_not_provable_after_applying_implies_right_rule() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::implies(
            predicate_formula("p"),
            predicate_formula("r"),
        )],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_reuses_visible_term_for_exists_right_before_fresh_fallback() {
    let sequent = Sequent {
        left: vec![predicate_formula_with_args("p", vec![constant("a")])],
        right: vec![Formula::Exists(
            vec![var("X")],
            Box::new(predicate_formula_with_args("p", vec![variable("X")])),
        )],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_reuses_visible_term_for_forall_left_before_fresh_fallback() {
    let sequent = Sequent {
        left: vec![Formula::ForAll(
            vec![var("X")],
            Box::new(predicate_formula_with_args("p", vec![variable("X")])),
        )],
        right: vec![predicate_formula_with_args("p", vec![constant("a")])],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_unknown_when_fresh_exists_right_fallback_is_exhausted() {
    let sequent = Sequent {
        left: Vec::new(),
        right: vec![Formula::Exists(
            vec![var("X")],
            Box::new(predicate_formula_with_args("p", vec![variable("X")])),
        )],
    };

    let result = prove(
        &sequent,
        ProofOptions {
            timeout: Duration::from_millis(50),
            ..default_options()
        },
    );

    assert_eq!(result.status, ProofStatus::Unknown);
}

#[test]
fn prove_promotes_generated_branch_terms_and_reports_limit_exhaustion_as_unknown() {
    let sequent = Sequent {
        left: vec![
            Formula::ForAll(
                vec![var("X")],
                Box::new(predicate_formula_with_args(
                    "p",
                    vec![Term::Fun {
                        name: Symbol::User("f".to_owned()),
                        args: vec![variable("X")],
                    }],
                )),
            ),
            predicate_formula_with_args("p", vec![constant("a")]),
        ],
        right: vec![predicate_formula("goal")],
    };

    let result = prove(
        &sequent,
        ProofOptions {
            timeout: Duration::from_millis(50),
            max_steps: 8,
            ..default_options()
        },
    );

    assert_eq!(result.status, ProofStatus::Unknown);
}

#[test]
fn prove_reconsiders_exists_right_after_forall_right_introduces_eigen_term() {
    let sequent = Sequent {
        left: Vec::new(),
        right: vec![Formula::Exists(
            vec![var("X")],
            Box::new(Formula::ForAll(
                vec![var("Y")],
                Box::new(Formula::implies(
                    predicate_formula_with_args("p", vec![variable("X")]),
                    predicate_formula_with_args("p", vec![variable("Y")]),
                )),
            )),
        )],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_provable_when_impliesr_exposes_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::implies(
            predicate_formula("p"),
            predicate_formula("q"),
        )],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_provable_for_modus_ponens_shape_via_impliesl() {
    let sequent = Sequent {
        left: vec![
            Formula::implies(predicate_formula("p"), predicate_formula("q")),
            predicate_formula("p"),
        ],
        right: vec![predicate_formula("q")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_not_provable_when_impliesl_leaves_an_open_branch() {
    let sequent = Sequent {
        left: vec![Formula::implies(
            predicate_formula("p"),
            predicate_formula("q"),
        )],
        right: vec![predicate_formula("q")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_timeout_for_large_left_branching_search() {
    let sequent = left_disjunction_timeout_sequent(24);

    let result = prove(
        &sequent,
        ProofOptions {
            timeout: Duration::from_millis(1),
            max_depth: usize::MAX,
            max_steps: usize::MAX,
            ..default_options()
        },
    );

    assert_eq!(result.status, ProofStatus::Timeout);
}

#[test]
fn prove_respects_custom_timeout_options() {
    let sequent = left_disjunction_timeout_sequent(24);
    let options = ProofOptions {
        timeout: Duration::from_millis(1),
        ..default_options()
    };

    let result = prove(&sequent, options);

    assert_eq!(result.status, ProofStatus::Timeout);
}

#[test]
fn prove_returns_unknown_when_depth_limit_is_hit() {
    let sequent = Sequent {
        left: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("p"), predicate_formula("q")],
    };

    let result = prove(
        &sequent,
        ProofOptions {
            max_depth: 0,
            timeout: Duration::from_secs(1),
            max_steps: usize::MAX,
            ..default_options()
        },
    );

    assert_eq!(result.status, ProofStatus::Unknown);
}

#[test]
fn prove_returns_unknown_when_step_limit_is_hit() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("p")],
    };

    let result = prove(
        &sequent,
        ProofOptions {
            max_depth: usize::MAX,
            timeout: Duration::from_secs(1),
            max_steps: 0,
            ..default_options()
        },
    );

    assert_eq!(result.status, ProofStatus::Unknown);
}

#[test]
fn prove_returns_cancelled_when_flag_is_already_set() {
    let sequent = Sequent {
        left: vec![Formula::or(vec![
            predicate_formula("p"),
            predicate_formula("q"),
        ])],
        right: vec![predicate_formula("p"), predicate_formula("q")],
    };
    let cancelled = AtomicBool::new(true);

    let result = prove_with_cancel(&sequent, default_options(), &cancelled);

    assert_eq!(result.status, ProofStatus::Cancelled);
}

#[test]
fn prove_returns_cancelled_when_flag_is_raised_during_search() {
    let sequent = left_disjunction_timeout_sequent(24);
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancel_for_thread = Arc::clone(&cancelled);
    let _signal_thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1));
        cancel_for_thread.store(true, Ordering::Relaxed);
    });

    let result = prove_with_cancel(
        &sequent,
        ProofOptions {
            timeout: Duration::from_secs(1),
            max_depth: usize::MAX,
            max_steps: usize::MAX,
        },
        cancelled.as_ref(),
    );

    assert_eq!(result.status, ProofStatus::Cancelled);
}

#[test]
fn prove_returns_provable_when_notr_exposes_identity() {
    let sequent = Sequent {
        left: vec![predicate_formula("q")],
        right: vec![Formula::not(predicate_formula("p")), predicate_formula("p")],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_returns_provable_when_notl_exposes_identity() {
    let sequent = Sequent {
        left: vec![
            predicate_formula("p"),
            Formula::not(predicate_formula("q")),
            predicate_formula("q"),
        ],
        right: vec![],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}

#[test]
fn prove_does_not_mutate_the_borrowed_sequent() {
    let sequent = Sequent {
        left: vec![predicate_formula("p")],
        right: vec![predicate_formula("q")],
    };
    let before = sequent.clone();

    let first = prove(&sequent, default_options());
    let second = prove(&sequent, default_options());

    assert_eq!(first.status, ProofStatus::NotProvable);
    assert_eq!(second.status, ProofStatus::NotProvable);
    assert_eq!(sequent, before);
}

#[test]
fn prove_returns_not_provable_for_atomic_sequent_built_from_parsed_problem() {
    let parsed = parse_problem(
        r#"
fof(ax_1,axiom,p).
fof(hyp_1,hypothesis,q).
fof(conj_1,conjecture,r).
"#,
    )
    .expect("problem should parse");
    let sequent = Sequent::from_parsed_problem(parsed).expect("sequent should build");

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn prove_returns_not_provable_for_sequent_built_from_parsed_problem_with_connective() {
    let parsed = parse_problem(
        r#"
fof(ax_1,axiom,(p & q)).
fof(conj_1,conjecture,r).
"#,
    )
    .expect("problem should parse");
    let sequent = Sequent::from_parsed_problem(parsed).expect("sequent should build");

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::NotProvable);
}

#[test]
fn proves_identity_sequent() {
    let p = predicate_formula("p");

    let sequent = Sequent {
        left: vec![p.clone()],
        right: vec![p],
    };

    let result = prove(&sequent, default_options());

    assert_eq!(result.status, ProofStatus::Provable);
}
