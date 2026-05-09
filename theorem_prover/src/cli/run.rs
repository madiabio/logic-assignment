//! Command dispatch and execution for the CLI.
//!
//! This module coordinates the high-level flow of CLI commands, including:
//! - Parsing command-line options
//! - Resolving configuration from CLI flags and config.toml
//! - Dispatching to appropriate proof or rule inspection handlers
//!
//! ## Configuration Resolution
//!
//! When a subset-based run is requested (i.e., no direct target file is provided),
//! the command resolves TPTP configuration in the following order:
//!
//! 1. CLI flags: `--tptp-root` and `--subset-file`
//! 2. Config.toml values: `tptp_root` and `default_subset_file`
//! 3. Error: If neither source provides both values
//!
//! See `resolve_tptp_config_or_exit` for implementation details.
//!
//! ## Direct Target Handling
//!
//! When a direct target (file or directory) is provided via positional argument,
//! configuration is not used, and the target is processed directly.

use crate::cli::args::{OutputFormat, ProveCommand, RulesCommand};
use crate::cli::cancel::{CancellationState, EXIT_FAILURE};
use crate::cli::config::{
    EnsureConfigError, TptpConfigError, biconditional_policy_from_cli, ensure_config,
    load_config_if_present, prover_options_from_cli, resolve_persist_path,
    validate_and_merge_tptp_config,
};
use crate::cli::output::{print_prove_preamble, print_rules_preamble};
use crate::cli::prove::{
    prove_directory, prove_file, prove_paths, report_single_prove_file,
    result_record_for_problem,
};
use crate::cli::rules::{
    inspect_rules_directory, inspect_rules_file, inspect_rules_paths, report_single_file,
};
use crate::cli::subset::{ProblemRun, resolve_subset_targets_with_paths};
use chrono::Local;
use std::path::Path;
use theorem_prover::{BiconditionalPolicy, ProofOptions, SearchEngine};
use theorem_prover::persistence::{self, RunRecord};

/// Formats one key/value pair for the `% settings` comment line.
fn setting(name: &str, value: impl std::fmt::Display) -> String {
    format!("{name}={value}")
}

/// Formats an optional integer setting for the `% settings` comment line.
fn optional_usize_setting(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => setting(name, value),
        None => setting(name, "none"),
    }
}

/// Formats the effective `prove` settings after config and CLI overrides.
fn prove_settings_comment(
    options: &ProveCommand,
    proof_options: &ProofOptions,
    biconditional_policy: BiconditionalPolicy,
) -> String {
    [
        setting(
            "format",
            match options.format {
                OutputFormat::Human => "human",
                OutputFormat::Tsv => "tsv",
            },
        ),
        setting("retry_parse_failed", options.run.retry_parse_failed),
        setting("show_sequent", options.display.show_sequent),
        setting("timeout_ms", proof_options.timeout.as_millis()),
        setting("max_depth", proof_options.max_depth),
        setting("max_steps", proof_options.max_steps),
        setting(
            "max_fresh_terms_per_quantifier",
            proof_options.max_fresh_terms_per_quantifier,
        ),
        optional_usize_setting(
            "max_biconditionals",
            biconditional_policy.max_biconditionals,
        ),
    ]
    .join(" ")
}

/// Formats the effective `rules` settings after config and CLI overrides.
fn rules_settings_comment(
    options: &RulesCommand,
    biconditional_policy: BiconditionalPolicy,
) -> String {
    [
        setting(
            "format",
            match options.format {
                OutputFormat::Human => "human",
                OutputFormat::Tsv => "tsv",
            },
        ),
        setting("retry_parse_failed", options.run.retry_parse_failed),
        setting("show_sequent", options.display.show_sequent),
        optional_usize_setting(
            "max_biconditionals",
            biconditional_policy.max_biconditionals,
        ),
    ]
    .join(" ")
}

/// Dispatches the `prove` command across direct targets or configured subset
/// runs. Handles CLI overrides for `--tptp-root` and `--subset-file`.
pub(crate) fn run_prover_mode(options: &ProveCommand) {
    let cancellation = CancellationState::install();
    let config = load_config_if_present();
    let proof_options = prover_options_from_cli(options);
    let biconditional_policy = biconditional_policy_from_cli(options.run.max_biconditionals);
    let settings = prove_settings_comment(options, &proof_options, biconditional_policy);
    if let Some(target) = &options.target {
        cancellation.defer_exit_until_summary();
        let mut db_state = prepare_persistence_state(options, &proof_options, config.as_ref());
        let target = Path::new(target);
        if target.is_dir() {
            prove_directory(target, options, &cancellation, &settings, db_state.take());
        } else {
            print_prove_preamble(options.format, None, &settings);
            let problem_run = ProblemRun {
                path: target.to_path_buf(),
                subset_stats: None,
            };
            let (result, elapsed_ms) = prove_file(
                &problem_run,
                options,
                &cancellation,
                1,
                1,
            );
            if let Some((conn, run_id)) = db_state.as_ref() {
                if let Some(record) = result_record_for_problem(&problem_run, &result, elapsed_ms) {
                    if let Err(err) = persistence::insert_result(conn, *run_id, &record) {
                        eprintln!(
                            "warning: failed to persist result for {}: {err}",
                            problem_run.problem_id()
                        );
                    }
                }
            }
            report_single_prove_file(result);
        }
        return;
    }

    let (tptp_root, subset_file) =
        resolve_tptp_config_or_exit(options.tptp_root.as_ref(), options.subset_file.as_ref());
    cancellation.defer_exit_until_summary();
    let mut db_state = prepare_persistence_state(options, &proof_options, config.as_ref());
    let targets = resolve_subset_targets_with_paths(&tptp_root, &subset_file);
    print_prove_preamble(options.format, Some(targets.len()), &settings);
    prove_paths(&targets, options, &cancellation, db_state.take());
}

/// Dispatches the `rules` command across direct targets or configured subset
/// runs. Handles CLI overrides for `--tptp-root` and `--subset-file`.
pub(crate) fn run_rules_mode(options: &RulesCommand) {
    let cancellation = CancellationState::install();
    let biconditional_policy = biconditional_policy_from_cli(options.run.max_biconditionals);
    let settings = rules_settings_comment(options, biconditional_policy);
    if let Some(target) = &options.target {
        cancellation.defer_exit_until_summary();
        let target = Path::new(target);
        if target.is_dir() {
            inspect_rules_directory(target, options, &cancellation, &settings);
        } else {
            print_rules_preamble(options.format, None, &settings);
            let result = inspect_rules_file(
                &ProblemRun {
                    path: target.to_path_buf(),
                    subset_stats: None,
                },
                options,
                1,
                1,
            );
            report_single_file(result.success);
        }
        return;
    }

    let (tptp_root, subset_file) =
        resolve_tptp_config_or_exit(options.tptp_root.as_ref(), options.subset_file.as_ref());
    cancellation.defer_exit_until_summary();
    let targets = resolve_subset_targets_with_paths(&tptp_root, &subset_file);
    print_rules_preamble(options.format, Some(targets.len()), &settings);
    inspect_rules_paths(&targets, options, &cancellation);
}

/// Resolves TPTP configuration from CLI overrides and config.toml, exiting on error.
///
/// Precedence:
/// 1. CLI flags (--tptp-root, --subset-file)
/// 2. Config.toml values
/// 3. Exit with error if both sources are incomplete
fn resolve_tptp_config_or_exit(
    cli_tptp_root: Option<&std::path::PathBuf>,
    cli_subset_file: Option<&std::path::PathBuf>,
) -> (std::path::PathBuf, std::path::PathBuf) {
    if let (Some(tptp_root), Some(subset_file)) = (cli_tptp_root, cli_subset_file) {
        return (tptp_root.clone(), subset_file.clone());
    }

    let config = match ensure_config() {
        Ok(config) => config,
        Err(EnsureConfigError::Aborted) => std::process::exit(EXIT_FAILURE),
    };

    match validate_and_merge_tptp_config(cli_tptp_root, cli_subset_file, Some(&config)) {
        Ok((tptp_root, subset_file)) => (tptp_root, subset_file),
        Err(TptpConfigError::MissingTptpRoot) => {
            eprintln!("error: TPTP root directory not found");
            eprintln!("  provide --tptp-root <PATH> or set tptp_root in config.toml");
            std::process::exit(EXIT_FAILURE);
        }
        Err(TptpConfigError::MissingSubsetFile) => {
            eprintln!("error: subset file not found");
            eprintln!("  provide --subset-file <PATH> or set default_subset_file in config.toml");
            std::process::exit(EXIT_FAILURE);
        }
    }
}

/// Resolves and opens the SQLite persistence backend for a `prove` run.
///
/// Returns `None` when persistence is disabled for this invocation. If a path
/// is selected but the database cannot be opened or initialized, the command
/// exits with an error so the user can fix the configured location.
fn prepare_persistence_state(
    options: &ProveCommand,
    proof_options: &ProofOptions,
    config: Option<&crate::cli::config::AppConfig>,
) -> Option<(rusqlite::Connection, i64)> {
    let Some(persist_path) = resolve_persist_path(options.persist.as_ref(), config) else {
        return None;
    };

    let conn = persistence::open_db(&persist_path).unwrap_or_else(|err| {
        eprintln!(
            "error: failed to open results database at {}: {err}",
            persist_path.display()
        );
        std::process::exit(EXIT_FAILURE);
    });
    persistence::ensure_schema(&conn).unwrap_or_else(|err| {
        eprintln!(
            "error: failed to initialize results database at {}: {err}",
            persist_path.display()
        );
        std::process::exit(EXIT_FAILURE);
    });

    let run = build_run_record(options, proof_options);
    let run_id = persistence::insert_run(&conn, &run).unwrap_or_else(|err| {
        eprintln!(
            "error: failed to create run record in {}: {err}",
            persist_path.display()
        );
        std::process::exit(EXIT_FAILURE);
    });

    Some((conn, run_id))
}

/// Builds the run metadata row stored in SQLite for a proof batch.
fn build_run_record(options: &ProveCommand, proof_options: &ProofOptions) -> RunRecord {
    let engine = engine_name(proof_options.engine);
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
    let label_stamp = now.format("%Y%m%d_%H%M%S").to_string();
    let label = options
        .run_label
        .clone()
        .unwrap_or_else(|| default_run_label(proof_options.engine, &label_stamp));

    let problem_class = options.problem_class.to_string();

    RunRecord {
        label,
        timestamp,
        engine: engine.to_string(),
        timeout_ms: proof_options.timeout.as_millis() as u64,
        max_depth: proof_options.max_depth as u32,
        max_steps: proof_options.max_steps as u64,
        max_fresh_terms_per_quantifier: proof_options.max_fresh_terms_per_quantifier as u32,
        problem_class,
    }
}

/// Generates the default run label when `--run-label` is not supplied.
fn default_run_label(engine: SearchEngine, timestamp: &str) -> String {
    format!("{}_{}", engine_name(engine), timestamp)
}

/// Maps the proof engine to the label stored in SQLite and used in default run labels.
fn engine_name(engine: SearchEngine) -> &'static str {
    match engine {
        SearchEngine::Naive => "naive",
        SearchEngine::IterativeDeepening => "id",
        SearchEngine::Priority => "priority",
        SearchEngine::PriorityId => "priority-id",
    }
}
