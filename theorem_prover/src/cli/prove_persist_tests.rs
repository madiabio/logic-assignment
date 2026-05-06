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
fn result_record_for_problem_maps_unknown_reason() {
    let result = ProveFileResult::Status(
        ProofStatus::Unknown,
        Some(UnknownReason::MaxDepthExceeded),
    );
    let record = result_record_for_problem(&problem_run("sample.p"), &result, 123)
        .expect("status results should be persisted");

    assert_eq!(record.problem_id, "sample");
    assert_eq!(record.path, "sample.p");
    assert_eq!(record.status, "unknown");
    assert_eq!(record.elapsed_ms, 123);
    assert_eq!(record.unknown_reason.as_deref(), Some("max_depth"));
}

#[test]
fn result_record_for_problem_skips_processing_failures() {
    let result = ProveFileResult::ProcessingFailure;
    assert!(result_record_for_problem(&problem_run("sample.p"), &result, 0).is_none());
}
