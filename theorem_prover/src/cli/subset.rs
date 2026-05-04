//! TPTP subset resolution and problem targeting.
//!
//! This module handles resolving subset files into concrete TPTP problem paths and managing
//! subset-based problem filtering.
//!
//! ## Subset File Format
//!
//! Subset files contain one problem per line with optional metadata:
//! - Lines starting with `%` are treated as comments
//! - Empty lines are ignored
//! - Problem IDs must contain `+` to be considered valid (e.g., `SYN001+1`)
//! - Optional metadata columns include formula count and atom count
//!
//! ## Problem Resolution
//!
//! Problems are resolved using the configured TPTP root directory:
//! - Versioned IDs (e.g., `LCL662+1.001`) prefer exact files when present
//! - Falls back to base file if versioned variant doesn't exist (e.g., `LCL662+1.p`)
//!
//! ## CLI Override Support
//!
//! The `resolve_subset_targets_with_paths` function allows CLI flags to override
//! the default configuration-based resolution. This is the primary entry point
//! when `--tptp-root` or `--subset-file` are provided.

use crate::cli::config::AppConfig;
use std::fs;
use std::path::{Path, PathBuf};

/// A single problem selected for CLI processing.
#[derive(Clone)]
pub(crate) struct ProblemRun {
    pub(crate) path: PathBuf,
    pub(crate) subset_stats: Option<SubsetStats>,
}

impl ProblemRun {
    /// Returns the logical TPTP problem id derived from the target file name.
    pub(crate) fn problem_id(&self) -> String {
        self.path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_string()
    }

    /// Returns the human-display value for the problem's formula count.
    pub(crate) fn human_formulae(&self) -> String {
        self.subset_stats
            .map(|stats| stats.formulae.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    /// Returns the human-display value for the problem's atom count.
    pub(crate) fn human_atoms(&self) -> String {
        self.subset_stats
            .map(|stats| stats.atoms.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

/// Optional metadata parsed from subset description rows.
#[derive(Clone, Copy)]
pub(crate) struct SubsetStats {
    pub(crate) formulae: usize,
    pub(crate) atoms: usize,
}

/// Resolves the configured subset file into concrete TPTP problem paths.
pub(crate) fn resolve_subset_targets(config: &AppConfig) -> Vec<ProblemRun> {
    resolve_subset_targets_with_paths(&config.tptp_root, &config.default_subset_file)
}

/// Resolves a subset file into concrete TPTP problem paths using explicit paths.
///
/// This function allows overriding the default paths from config.toml. It is used
/// when CLI flags like `--tptp-root` or `--subset-file` are provided.
///
/// # Arguments
/// * `tptp_root` - Path to the TPTP-v9.x.x root directory
/// * `subset_file` - Path to the subset file describing which problems to process
pub(crate) fn resolve_subset_targets_with_paths(
    tptp_root: &Path,
    subset_file: &Path,
) -> Vec<ProblemRun> {
    let subset_contents = fs::read_to_string(subset_file).unwrap_or_else(|err| {
        panic!(
            "failed to read subset file {}: {err}",
            subset_file.display()
        )
    });

    subset_contents
        .lines()
        .filter_map(parse_subset_problem_line)
        .map(|(problem_id, subset_stats)| ProblemRun {
            path: resolve_tptp_problem_path(tptp_root, &problem_id),
            subset_stats,
        })
        .collect()
}

/// Extracts a problem id and optional stats from a subset-description line.
pub(crate) fn parse_subset_problem_line(line: &str) -> Option<(String, Option<SubsetStats>)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('%') {
        return None;
    }

    let tokens: Vec<_> = trimmed.split_whitespace().collect();
    let problem_id = *tokens.first()?;
    if !problem_id.contains('+') {
        return None;
    }

    let subset_stats = match (tokens.get(5), tokens.get(8)) {
        (Some(formulae), Some(atoms)) => {
            match (formulae.parse::<usize>(), atoms.parse::<usize>()) {
                (Ok(formulae), Ok(atoms)) => Some(SubsetStats { formulae, atoms }),
                _ => None,
            }
        }
        _ => None,
    };

    Some((problem_id.to_string(), subset_stats))
}

/// Resolves a subset problem id to an on-disk TPTP problem file.
///
/// Versioned ids such as `LCL662+1.001` prefer the exact file when present and
/// fall back to the base file if only the unversioned variant exists.
pub(crate) fn resolve_tptp_problem_path(tptp_root: &Path, problem_id: &str) -> PathBuf {
    let domain = &problem_id[..3];
    let problems_dir = tptp_root.join("Problems").join(domain);
    let exact_path = problems_dir.join(format!("{problem_id}.p"));
    if exact_path.exists() {
        return exact_path;
    }

    let base_problem_id = problem_id.split('.').next().unwrap_or(problem_id);
    problems_dir.join(format!("{base_problem_id}.p"))
}

/// Returns numeric subset stats for machine-readable output, defaulting to zero
/// when the subset source did not provide them.
pub(crate) fn subset_stats_fields(stats: Option<SubsetStats>) -> (usize, usize) {
    stats
        .map(|stats| (stats.formulae, stats.atoms))
        .unwrap_or((0, 0))
}

#[cfg(test)]
mod tests {
    use super::{parse_subset_problem_line, resolve_tptp_problem_path};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_subset_problem_line_extracts_stats() {
        let line = "SYN001+1            FOF THM   0.00 FOF_THM_PRP                  1      1      0      1";
        let (problem_id, stats) = parse_subset_problem_line(line).expect("line should parse");
        let stats = stats.expect("stats should be present");

        assert_eq!(problem_id, "SYN001+1");
        assert_eq!(stats.formulae, 1);
        assert_eq!(stats.atoms, 1);
    }

    #[test]
    fn resolve_tptp_problem_path_prefers_exact_versioned_file() {
        let temp_dir = std::env::temp_dir().join(format!(
            "theorem_prover_subset_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be valid")
                .as_nanos()
        ));
        let problems_dir = temp_dir.join("Problems").join("LCL");
        fs::create_dir_all(&problems_dir).expect("problems dir should exist");
        fs::write(problems_dir.join("LCL662+1.001.p"), "").expect("problem should be created");

        let path = resolve_tptp_problem_path(&temp_dir, "LCL662+1.001");
        assert!(path.ends_with("LCL662+1.001.p"));
    }
}
