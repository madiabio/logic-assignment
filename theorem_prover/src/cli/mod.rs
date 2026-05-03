//! Command-line interface support for the `theorem_prover` binary.
//!
//! These modules keep argument parsing, config handling, subset resolution,
//! rendering, and command execution out of `main.rs` while remaining private
//! to the executable.

pub(crate) mod args;
pub(crate) mod config;
pub(crate) mod output;
pub(crate) mod run;
pub(crate) mod subset;

pub(crate) use args::{CliOptions, Command};
pub(crate) use run::{run_prover_mode, run_rules_mode};
