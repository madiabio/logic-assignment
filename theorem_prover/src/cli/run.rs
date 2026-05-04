use crate::cli::args::{OutputFormat, ProveCommand, RulesCommand};
use crate::cli::cancel::{CancellationState, EXIT_FAILURE};
use crate::cli::config::{
    EnsureConfigError, biconditional_policy_from_cli, ensure_config, prover_options_from_cli,
};
use crate::cli::output::{print_prove_preamble, print_rules_preamble};
use crate::cli::prove::{prove_directory, prove_file, prove_paths, report_single_prove_file};
use crate::cli::rules::{
    inspect_rules_directory, inspect_rules_file, inspect_rules_paths, report_single_file,
};
use crate::cli::subset::{ProblemRun, resolve_subset_targets};
use std::path::Path;
use theorem_prover::{BiconditionalPolicy, ProofOptions};

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
    proof_options: ProofOptions,
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
/// runs.
pub(crate) fn run_prover_mode(options: &ProveCommand) {
    let cancellation = CancellationState::install();
    let proof_options = prover_options_from_cli(options);
    let biconditional_policy = biconditional_policy_from_cli(options.run.max_biconditionals);
    let settings = prove_settings_comment(options, proof_options, biconditional_policy);
    if let Some(target) = &options.target {
        cancellation.defer_exit_until_summary();
        let target = Path::new(target);
        if target.is_dir() {
            prove_directory(target, options, &cancellation, &settings);
        } else {
            print_prove_preamble(options.format, None, &settings);
            let result = prove_file(
                &ProblemRun {
                    path: target.to_path_buf(),
                    subset_stats: None,
                },
                options,
                &cancellation,
                1,
                1,
            );
            report_single_prove_file(result);
        }
        return;
    }

    let config = require_config_or_exit();
    cancellation.defer_exit_until_summary();
    let targets = resolve_subset_targets(&config);
    print_prove_preamble(options.format, Some(targets.len()), &settings);
    prove_paths(&targets, options, &cancellation);
}

/// Dispatches the `rules` command across direct targets or configured subset
/// runs.
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

    let config = require_config_or_exit();
    cancellation.defer_exit_until_summary();
    let targets = resolve_subset_targets(&config);
    print_rules_preamble(options.format, Some(targets.len()), &settings);
    inspect_rules_paths(&targets, options, &cancellation);
}

/// Loads or creates the local CLI config, exiting when the user declines repair.
fn require_config_or_exit() -> crate::cli::config::AppConfig {
    match ensure_config() {
        Ok(config) => config,
        Err(EnsureConfigError::Aborted) => std::process::exit(EXIT_FAILURE),
    }
}
