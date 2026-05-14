//! First-order logic theorem prover for TPTP-formatted problems.
//!
//! This crate implements Hou's LK′ backward sequent search with three
//! interchangeable engines that share a single proof kernel:
//!
//! - [`SearchEngine::Naive`]: Hou's depth-first baseline.
//! - [`SearchEngine::IterativeDeepening`]: the same DFS wrapped in
//!   iterative deepening, retrying at depth limits 1, 2, … up to
//!   [`ProofOptions::max_depth`].
//! - [`SearchEngine::PriorityId`]: iterative deepening paired with a
//!   six-class rule-priority schedule.
//!
//! Only the scheduler differs between engines; parsing, the sequent
//! representation, rule application, and the [`prove_with_cancel`]
//! kernel are shared.
//!
//! # Quick start
//!
//! ```ignore
//! use theorem_prover::{parse_problem, prove, ProofOptions, Sequent};
//!
//! let parsed = parse_problem("fof(g, conjecture, p | ~p).").unwrap();
//! let sequent = Sequent::from_parsed_problem(parsed).unwrap();
//! let result = prove(&sequent, ProofOptions::default());
//! println!("{:?}", result.status);
//! ```
//!
//! For end-to-end use, the CLI in `bin` exposes `prove` and `rules`
//! subcommands over a directory of `.p` files or a TPTP subset
//! description; see `theorem_prover/README.md` and run
//! `cargo run -- prove --help`.
//!
//! # Module map
//!
//! - [`parser`]: pest grammar (`tptp.pest`) and AST builder for TPTP FOF.
//! - [`ast`]: [`ast::Formula`] and [`ast::Term`] definitions.
//! - [`proof`]: sequent representation, rule layer, search kernel, and
//!   the three engines. The most useful entry points are re-exported at
//!   the crate root ([`prove`], [`prove_with_cancel`], [`ProofOptions`],
//!   [`ProofResult`], [`Sequent`]).
//! - [`pipeline`]: orchestrates parse → prove → persist for one problem,
//!   including the biconditional gate and structured error reporting.
//! - [`persistence`]: SQLite schema and helpers for the `runs`/`results`
//!   tables that back the report's figures.
//!
//! # Further reading
//!
//! The calculus, engine design rationale, and the empirical evaluation
//! across TPTP, FOLIO, and a synthetic FOF benchmark are written up in
//! the project report under `report/sections/`. The repository README
//! gives a top-level overview, and `theorem_prover/README.md` documents
//! the CLI, configuration, and result schema in detail.

pub mod ast;
pub mod parser;
pub mod persistence;
pub mod pipeline;
pub mod proof;

pub use parser::{FormulaRecord, IncludeDirective, ParsedProblem, parse_problem, parse_tptp};
pub use pipeline::*;
pub use proof::prover::*;
pub use proof::sequent::*;

// Re-exports used only in integration tests. Not part of the stable public API.
#[doc(hidden)]
pub use proof::search::scheduler::{ScheduleResult, ScheduledRule, quantified_occurrence_key, schedule_next_rules_lk_priority};
#[doc(hidden)]
pub use proof::search::branch_state::{BranchState, record_quantifier_term};
#[doc(hidden)]
pub use proof::rules::{Rule, Side, RuleMatch};
