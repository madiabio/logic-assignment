use super::{ProveFileResult, result_record_for_problem};
use crate::cli::subset::ProblemRun;
use std::path::PathBuf;
use theorem_prover::{ProofStatus, UnknownReason};

fn problem_run(path: &str) -> ProblemRun {
    ProblemRun {
        path: PathBuf::from(path),
        subset_stats: None,
    }
}

#[test]
fn result_record_for_problem_maps_all_unknown_reasons() {
    let cases = [
        (UnknownReason::BiconditionalCapExceeded, "biconditional_cap"),
        (UnknownReason::MaxDepthExceeded, "max_depth"),
        (UnknownReason::MaxStepsExceeded, "max_steps"),
        (UnknownReason::QuantifierBudgetExceeded, "quantifier_budget"),
    ];

    for (reason, expected_label) in cases {
        let result = ProveFileResult::Status(ProofStatus::Unknown, Some(reason));
        let record = result_record_for_problem(&problem_run("sample.p"), &result, 123)
            .expect("status results should be persisted");

        assert_eq!(record.problem_id, "sample");
        assert_eq!(record.path, "sample.p");
        assert_eq!(record.status, "unknown");
        assert_eq!(record.elapsed_ms, 123);
        assert_eq!(record.unknown_reason.as_deref(), Some(expected_label));
    }
}

#[test]
fn result_record_for_problem_skips_processing_failures() {
    let result = ProveFileResult::ProcessingFailure;
    assert!(result_record_for_problem(&problem_run("sample.p"), &result, 0).is_none());
}
