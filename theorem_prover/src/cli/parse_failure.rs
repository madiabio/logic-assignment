//! Filesystem helpers for `.parse_failed` retry markers used by the CLI.

use crate::cli::args::ParseFailureOptions;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Returns the path of the `.parse_failed` marker associated with a `.p` file.
pub(crate) fn parse_failure_marker_path(path: &Path) -> Option<PathBuf> {
    (path.extension().and_then(|ext| ext.to_str()) == Some("p"))
        .then(|| PathBuf::from(format!("{}.parse_failed", path.display())))
}

/// Writes a `.parse_failed` marker alongside an input file after parse failure.
pub(crate) fn write_parse_failure_marker(path: &Path, err: &str) {
    let Some(marker_path) = parse_failure_marker_path(path) else {
        return;
    };

    let contents = format!("{}\nparse failed\n{err}\n", path.display());
    if let Err(write_err) = fs::write(&marker_path, contents) {
        eprintln!(
            "{}: failed to write parse-failure marker {}",
            path.display(),
            marker_path.display()
        );
        eprintln!("{write_err}");
    }
}

/// Removes any stale `.parse_failed` marker for a file after successful processing.
pub(crate) fn clear_parse_failure_marker(path: &Path) {
    let Some(marker_path) = parse_failure_marker_path(path) else {
        return;
    };

    match fs::remove_file(&marker_path) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => {
            eprintln!(
                "{}: failed to remove parse-failure marker {}",
                path.display(),
                marker_path.display()
            );
            eprintln!("{err}");
        }
    }
}

/// Returns whether a file should be skipped because it already has a
/// parse-failure marker and retry was not requested.
pub(crate) fn should_skip_parse_failed_file(
    path: &Path,
    options: &impl ParseFailureOptions,
) -> bool {
    !options.retry_parse_failed()
        && parse_failure_marker_path(path).is_some_and(|marker_path| marker_path.exists())
}
