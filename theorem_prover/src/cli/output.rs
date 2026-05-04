use crate::cli::args::OutputFormat;
use std::path::Path;
use theorem_prover::{ProofResult, ProofStatus, UnknownReason};

/// Returns the TSV header row for `prove` output.
pub(crate) fn prove_tsv_header() -> &'static str {
    "kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tstatus\telapsed_ms\tdetail"
}

/// Returns the TSV header row for `rules` output.
pub(crate) fn rules_tsv_header() -> &'static str {
    "kind\tindex\ttotal\tproblem_id\tpath\tformulae\tatoms\tsuccess\thad_rule_match\tdetail"
}

/// Prints the human-readable `prove` table header.
pub(crate) fn print_prove_human_header() {
    println!(
        "{:<8}  {:<16}  {:<20}  {:>8}  {:>5}  {:>5}  path",
        "idx", "problem", "status", "time_ms", "frm", "atoms"
    );
}

/// Prints the human-readable `rules` table header.
pub(crate) fn print_rules_human_header() {
    println!(
        "{:<8}  {:<16}  {:<3}  {:<5}  {:>5}  {:>5}  path",
        "idx", "problem", "ok", "match", "frm", "atoms"
    );
}

/// Prints one human-readable `prove` row.
pub(crate) fn print_prove_human_row(
    current: usize,
    total: usize,
    problem_id: &str,
    status: &str,
    elapsed_ms: u128,
    formulae: String,
    atoms: String,
    path: &Path,
) {
    println!(
        "{:<8}  {:<16}  {:<20}  {:>8}  {:>5}  {:>5}  {}",
        format!("{current}/{total}"),
        problem_id,
        status,
        elapsed_ms,
        formulae,
        atoms,
        path.display()
    );
}

/// Prints one human-readable `rules` row.
pub(crate) fn print_rules_human_row(
    current: usize,
    total: usize,
    problem_id: &str,
    success: bool,
    had_rule_match: bool,
    formulae: String,
    atoms: String,
    path: &Path,
) {
    println!(
        "{:<8}  {:<16}  {:<3}  {:<5}  {:>5}  {:>5}  {}",
        format!("{current}/{total}"),
        problem_id,
        yes_no(success),
        yes_no(had_rule_match),
        formulae,
        atoms,
        path.display()
    );
}

/// Prints a summary section title in human output mode.
pub(crate) fn print_summary_header(title: &str) {
    println!();
    println!("{title}");
}

/// Prints an aligned human-readable summary row block.
pub(crate) fn print_summary_row(values: &[(&str, String)]) {
    let labels = values
        .iter()
        .map(|(label, _)| format!("{:<17}", label))
        .collect::<Vec<_>>()
        .join(" ");
    let row = values
        .iter()
        .map(|(_, value)| format!("{:<17}", value))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{labels}");
    println!("{row}");
}

/// Maps proof statuses to stable human-readable labels.
pub(crate) fn human_proof_status(status: &ProofStatus) -> &'static str {
    match status {
        ProofStatus::NotImplemented => "not_implemented",
        ProofStatus::Provable => "provable",
        ProofStatus::NotProvable => "not_provable",
        ProofStatus::Timeout => "timeout",
        ProofStatus::Unknown => "unknown",
        ProofStatus::Cancelled => "cancelled",
        ProofStatus::Error => "error",
    }
}

/// Returns the stable printable detail label for an unknown proof reason.
pub(crate) fn human_unknown_reason(reason: UnknownReason) -> &'static str {
    match reason {
        UnknownReason::BiconditionalCapExceeded => "biconditional_cap",
        UnknownReason::UnsupportedInclude => "unsupported_include",
        UnknownReason::MaxDepthExceeded => "max_depth",
        UnknownReason::MaxStepsExceeded => "max_steps",
        UnknownReason::QuantifierBudgetExceeded => "quantifier_budget",
    }
}

/// Formats the proof status with any available detail for human-readable output.
pub(crate) fn human_proof_result(result: &ProofResult) -> String {
    let status = human_proof_status(&result.status);
    match result.unknown_reason {
        Some(reason) => format!("{status} ({})", human_unknown_reason(reason)),
        None => status.to_string(),
    }
}

/// Prints a `%`-prefixed comment line describing the effective run settings.
pub(crate) fn print_settings_comment_line(settings: &str) {
    println!("% settings {settings}");
}

/// Prints the correct table header for the selected command/format.
pub(crate) fn print_prove_preamble(
    format: OutputFormat,
    subset_size: Option<usize>,
    settings: &str,
) {
    print_settings_comment_line(settings);
    match format {
        OutputFormat::Human => {
            if let Some(count) = subset_size {
                println!("Loaded {count} problem(s) from subset");
            }
            print_prove_human_header();
        }
        OutputFormat::Tsv => println!("{}", prove_tsv_header()),
    }
}

/// Prints the correct table header for the selected command/format.
pub(crate) fn print_rules_preamble(
    format: OutputFormat,
    subset_size: Option<usize>,
    settings: &str,
) {
    print_settings_comment_line(settings);
    match format {
        OutputFormat::Human => {
            if let Some(count) = subset_size {
                println!("Loaded {count} problem(s) from subset");
            }
            print_rules_human_header();
        }
        OutputFormat::Tsv => println!("{}", rules_tsv_header()),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::human_proof_status;
    use theorem_prover::ProofStatus;

    #[test]
    fn human_proof_status_uses_stable_labels() {
        assert_eq!(human_proof_status(&ProofStatus::Provable), "provable");
        assert_eq!(
            human_proof_status(&ProofStatus::NotProvable),
            "not_provable"
        );
        assert_eq!(human_proof_status(&ProofStatus::Unknown), "unknown");
        assert_eq!(human_proof_status(&ProofStatus::Cancelled), "cancelled");
    }
}
