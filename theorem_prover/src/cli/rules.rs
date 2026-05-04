//! Execution flow for the `rules` CLI subcommand.

use crate::cli::args::{OutputFormat, RulesCommand};
use crate::cli::cancel::{CancellationState, EXIT_FAILURE, rules_batch_exit_code};
use crate::cli::config::biconditional_policy_from_cli;
use crate::cli::output::{
    print_rules_human_row, print_rules_preamble, print_summary_header, print_summary_row,
};
use crate::cli::parse_failure::{
    clear_parse_failure_marker, should_skip_parse_failed_file, write_parse_failure_marker,
};
use crate::cli::subset::{ProblemRun, subset_stats_fields};
use std::fs;
use std::path::{Path, PathBuf};
use theorem_prover::proof::rules::{RuleMatch, find_applicable_rules};
use theorem_prover::{ProblemPipelineError, build_problem_sequent_from_path};

/// Outcome of running rule inspection on one file.
#[derive(Clone, Copy)]
pub(crate) struct RulesInspectionResult {
    pub(crate) success: bool,
    had_rule_match: bool,
    skipped_by_policy: bool,
}

/// Running counts and metadata for `rules` batch execution.
#[derive(Default)]
pub(crate) struct RulesBatchSummary {
    processed: usize,
    skipped: usize,
    skipped_by_policy: usize,
    succeeded: usize,
    failed: usize,
    rule_matches: usize,
    cancelled: bool,
    failed_files: Vec<PathBuf>,
}

/// Runs rule inspection over every `.p` file in a directory and prints aggregate counts.
pub(crate) fn inspect_rules_directory(
    dir: &Path,
    options: &RulesCommand,
    cancellation: &CancellationState,
    settings: &str,
) {
    let entries = fs::read_dir(dir).expect("Failed to read directory");
    let mut problem_runs = Vec::new();
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("p") {
            continue;
        }

        problem_runs.push(ProblemRun {
            path,
            subset_stats: None,
        });
    }

    print_rules_preamble(options.format, None, settings);
    inspect_rules_paths(&problem_runs, options, cancellation);
}

/// Processes many problems through the rule matcher and emits aggregate results.
pub(crate) fn inspect_rules_paths(
    problem_runs: &[ProblemRun],
    options: &RulesCommand,
    cancellation: &CancellationState,
) {
    let mut summary = RulesBatchSummary::default();
    let total = problem_runs.len();
    for (index, problem_run) in problem_runs.iter().enumerate() {
        if cancellation.is_requested() {
            summary.cancelled = true;
            break;
        }

        if should_skip_parse_failed_file(&problem_run.path, options) {
            summary.skipped += 1;
            continue;
        }

        summary.processed += 1;
        let inspection = inspect_rules_file(problem_run, options, current_index(index), total);
        if inspection.skipped_by_policy {
            summary.skipped_by_policy += 1;
        }
        if inspection.had_rule_match {
            summary.rule_matches += 1;
        }
        if inspection.success {
            summary.succeeded += 1;
        } else {
            summary.failed += 1;
            summary.failed_files.push(problem_run.path.clone());
        }
    }

    match options.format {
        OutputFormat::Human => {
            print_summary_header("summary");
            print_summary_row(&[
                ("processed", summary.processed.to_string()),
                ("skipped", summary.skipped.to_string()),
                ("skipped_by_policy", summary.skipped_by_policy.to_string()),
                ("succeeded", summary.succeeded.to_string()),
                ("failed", summary.failed.to_string()),
                ("rule_matches", summary.rule_matches.to_string()),
                ("cancelled", yes_no(summary.cancelled).to_string()),
            ]);
            if summary.cancelled {
                eprintln!("Cancelled before starting the next problem");
            }
        }
        OutputFormat::Tsv => {
            println!(
                "summary\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                summary.processed,
                summary.skipped,
                summary.skipped_by_policy,
                summary.succeeded,
                summary.failed,
                summary.rule_matches,
                summary.cancelled
            );
        }
    }

    if options.format == OutputFormat::Human && !summary.failed_files.is_empty() {
        eprintln!("Failed files:");
        for path in &summary.failed_files {
            eprintln!("  {}", path.display());
        }
    }

    if let Some(code) = rules_batch_exit_code(summary.cancelled, summary.failed) {
        std::process::exit(code);
    }
}

/// Runs rule inspection on one file and reports whether parsing/building succeeded.
pub(crate) fn inspect_rules_file(
    problem_run: &ProblemRun,
    options: &RulesCommand,
    current: usize,
    total: usize,
) -> RulesInspectionResult {
    let input = fs::read_to_string(&problem_run.path).expect("Failed to read input file");
    let problem_id = problem_run.problem_id();
    let (formulae, atoms) = subset_stats_fields(problem_run.subset_stats);
    let biconditional_policy = biconditional_policy_from_cli(options.run.max_biconditionals);

    if biconditional_policy.is_exceeded_by(&input) {
        match options.format {
            OutputFormat::Human => print_rules_human_row(
                current,
                total,
                &problem_id,
                true,
                false,
                problem_run.human_formulae(),
                problem_run.human_atoms(),
                &problem_run.path,
            ),
            OutputFormat::Tsv => println!(
                "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\ttrue\tfalse\tbiconditional_cap",
                problem_run.path.display()
            ),
        }
        if options.format == OutputFormat::Human {
            println!("  biconditional_cap");
        }
        return RulesInspectionResult {
            success: true,
            had_rule_match: false,
            skipped_by_policy: true,
        };
    }

    match build_problem_sequent_from_path(&problem_run.path) {
        Ok(sequent) => {
            clear_parse_failure_marker(&problem_run.path);
            let matches = find_applicable_rules(&sequent);
            match options.format {
                OutputFormat::Human => print_rules_human_row(
                    current,
                    total,
                    &problem_id,
                    true,
                    !matches.is_empty(),
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => println!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\t{}\t{}",
                    problem_run.path.display(),
                    true,
                    !matches.is_empty()
                ),
            }
            if options.format == OutputFormat::Human && options.display.show_sequent {
                println!("  {sequent}");
            }
            if options.format == OutputFormat::Human {
                if matches.is_empty() {
                    println!("  no applicable rules");
                } else {
                    for rule_match in &matches {
                        println!("  {}", format_rule_match(*rule_match));
                    }
                }
            }
            RulesInspectionResult {
                success: true,
                had_rule_match: !matches.is_empty(),
                skipped_by_policy: false,
            }
        }
        Err(ProblemPipelineError::Parse(err)) => {
            write_parse_failure_marker(&problem_run.path, &err);
            match options.format {
                OutputFormat::Human => print_rules_human_row(
                    current,
                    total,
                    &problem_id,
                    false,
                    false,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => eprintln!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\tfalse\tfalse\tparse_failed",
                    problem_run.path.display()
                ),
            }
            eprintln!("{err}");
            RulesInspectionResult {
                success: false,
                had_rule_match: false,
                skipped_by_policy: false,
            }
        }
        Err(ProblemPipelineError::Include(err)) => {
            clear_parse_failure_marker(&problem_run.path);
            match options.format {
                OutputFormat::Human => print_rules_human_row(
                    current,
                    total,
                    &problem_id,
                    false,
                    false,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => eprintln!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\tfalse\tfalse\tinclude_failed",
                    problem_run.path.display()
                ),
            }
            eprintln!("{err}");
            RulesInspectionResult {
                success: false,
                had_rule_match: false,
                skipped_by_policy: false,
            }
        }
        Err(ProblemPipelineError::SequentBuild(err)) => {
            match options.format {
                OutputFormat::Human => print_rules_human_row(
                    current,
                    total,
                    &problem_id,
                    false,
                    false,
                    problem_run.human_formulae(),
                    problem_run.human_atoms(),
                    &problem_run.path,
                ),
                OutputFormat::Tsv => eprintln!(
                    "problem\t{current}\t{total}\t{problem_id}\t{}\t{formulae}\t{atoms}\tfalse\tfalse\tsequent_build_failed",
                    problem_run.path.display()
                ),
            }
            eprintln!("sequent construction failed: {err:?}");
            RulesInspectionResult {
                success: false,
                had_rule_match: false,
                skipped_by_policy: false,
            }
        }
    }
}

/// Formats a matched rule occurrence for CLI output.
pub(crate) fn format_rule_match(rule_match: RuleMatch) -> String {
    format!(
        "{:?} on {:?}[{}]",
        rule_match.rule, rule_match.side, rule_match.index
    )
}

/// Prints single-file rule-inspection status and exits non-zero on failure.
pub(crate) fn report_single_file(success: bool) {
    if !success {
        std::process::exit(EXIT_FAILURE);
    }
}

fn current_index(index: usize) -> usize {
    index + 1
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
