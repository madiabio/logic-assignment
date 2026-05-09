//! Configuration management for the CLI.
//!
//! This module handles loading and merging configuration from `config.toml` and CLI overrides.
//!
//! ## Configuration Sources
//!
//! The CLI accepts configuration from multiple sources with the following precedence
//! (highest wins):
//!
//! 1. **CLI flags**
//!    - `--tptp-root <PATH>`
//!    - `--subset-file <PATH>`
//!    - `--timeout-ms <MS>`
//!    - `--max-depth <N>`
//!    - `--max-steps <N>`
//!    - `--max-fresh-terms-per-quantifier <N>`
//!    - `--engine naive|id`
//!
//! 2. **`config.toml`** (keys accepted)
//!    - `tptp_root` — path to TPTP-v9.x.x root directory *(required)*
//!    - `default_subset_file` — path to default subset file *(required)*
//!    - `timeout_ms` — wall-clock timeout in milliseconds
//!    - `max_depth` — maximum recursive proof-search depth
//!    - `max_steps` — maximum proof-search steps
//!    - `max_fresh_terms_per_quantifier` — fresh fallback terms per quantifier occurrence
//!    - `max_biconditionals` — biconditional gate before parsing
//!    - `engine` — proof-search strategy: `"naive"` or `"id"`
//!    - `results_db` — path to SQLite database for persisting proof results
//!      (`..\results.db` is used when this field is omitted)
//!
//! 3. **Interactive prompts** (if `config.toml` is missing)
//!    — prompts for the required fields on first run
//!
//! ## Configuration Requirements
//!
//! `tptp_root` and `default_subset_file` must be present in either the CLI flags or
//! `config.toml`. All other keys are optional and fall back to library defaults when absent.

use crate::cli::args::{CliSearchEngine, PersistOpt, ProveCommand};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Mutex;
use theorem_prover::proof::defaults::{
    DEFAULT_MAX_DEPTH, DEFAULT_MAX_FRESH_TERMS_PER_QUANTIFIER, DEFAULT_MAX_STEPS,
    DEFAULT_PROVE_TIMEOUT,
};
use theorem_prover::{BiconditionalPolicy, ProofOptions, SearchEngine};

const DEFAULT_RESULTS_DB: &str = r"..\results.db";

#[cfg(test)]
static CWD_LOCK: Mutex<()> = Mutex::new(());

/// Persistent defaults used by config-backed CLI runs.
#[derive(Clone, Debug)]
pub(crate) struct AppConfig {
    pub(crate) tptp_root: PathBuf,
    pub(crate) default_subset_file: PathBuf,
    pub(crate) timeout_ms: Option<u64>,
    pub(crate) max_depth: Option<usize>,
    pub(crate) max_steps: Option<usize>,
    pub(crate) max_fresh_terms_per_quantifier: Option<usize>,
    pub(crate) max_biconditionals: Option<usize>,
    /// Proof-search engine. `None` means the library default (`naive`) applies.
    pub(crate) engine: Option<CliSearchEngine>,
    /// Path to the SQLite database file for persisting proof results.
    ///
    /// When this field is omitted, the runtime falls back to
    /// [`default_results_db_path()`].
    pub(crate) results_db: Option<String>,
}

#[derive(Debug)]
enum ConfigLoadState {
    Missing,
    Available(AppConfig),
    Invalid(String),
}

#[derive(Debug)]
pub(crate) enum EnsureConfigError {
    Aborted,
}

/// Errors that can occur when validating TPTP configuration.
#[derive(Debug)]
pub(crate) enum TptpConfigError {
    /// Missing TPTP root directory from both CLI and config.
    MissingTptpRoot,
    /// Missing subset file from both CLI and config.
    MissingSubsetFile,
}

/// Loads `config.toml` when it is valid, otherwise returns `None`.
pub(crate) fn load_config_if_present() -> Option<AppConfig> {
    load_config().ok()
}

/// Returns a usable config, prompting and writing one on first run if needed.
pub(crate) fn ensure_config() -> Result<AppConfig, EnsureConfigError> {
    match classify_config_load() {
        ConfigLoadState::Available(config) => Ok(config),
        ConfigLoadState::Missing => Ok(prompt_for_config()),
        ConfigLoadState::Invalid(err) => {
            println!("config.toml is unusable: {err}");
            if confirm_repair() {
                Ok(prompt_for_config())
            } else {
                Err(EnsureConfigError::Aborted)
            }
        }
    }
}

/// Returns the built-in SQLite database path used when no config override is present.
pub(crate) fn default_results_db_path() -> PathBuf {
    PathBuf::from(DEFAULT_RESULTS_DB)
}

/// Validates and merges TPTP configuration from CLI arguments and config.toml.
///
/// Precedence:
/// 1. CLI flags (`tptp_root` and `subset_file` from options)
/// 2. Config.toml values
/// 3. Error if both sources are incomplete
///
/// # Arguments
/// * `options` - CLI arguments that may contain overrides
/// * `config` - Loaded configuration from config.toml (if available)
///
/// # Returns
/// A tuple `(tptp_root, subset_file)` with resolved paths, or an error if validation fails.
pub(crate) fn validate_and_merge_tptp_config(
    cli_tptp_root: Option<&PathBuf>,
    cli_subset_file: Option<&PathBuf>,
    config: Option<&AppConfig>,
) -> Result<(PathBuf, PathBuf), TptpConfigError> {
    let tptp_root = cli_tptp_root
        .cloned()
        .or_else(|| config.map(|c| c.tptp_root.clone()))
        .ok_or(TptpConfigError::MissingTptpRoot)?;

    let subset_file = cli_subset_file
        .cloned()
        .or_else(|| config.map(|c| c.default_subset_file.clone()))
        .ok_or(TptpConfigError::MissingSubsetFile)?;

    Ok((tptp_root, subset_file))
}

/// Resolves the effective persistence path for a run, considering CLI override and config.
///
/// Persistence is enabled by default. `false` disables it, an explicit path
/// overrides everything, and the fallback path is either `config.toml`'s
/// `results_db` value or the built-in `..\results.db`.
pub(crate) fn resolve_persist_path(
    cli_persist: Option<&PersistOpt>,
    config: Option<&AppConfig>,
) -> Option<PathBuf> {
    match cli_persist {
        Some(PersistOpt::Disabled) => None,
        Some(PersistOpt::Path(p)) => Some(PathBuf::from(p)),
        None => config
            .and_then(|config| config.results_db.as_ref().map(PathBuf::from))
            .or_else(|| Some(default_results_db_path())),
    }
}

/// Builds prover options by merging library defaults, `config.toml` settings,
/// and CLI flags in that order (CLI flags take highest precedence).
pub(crate) fn prover_options_from_cli(options: &ProveCommand) -> ProofOptions {
    let mut proof_options = ProofOptions::default();
    if let Some(config) = load_config_if_present() {
        if let Some(timeout_ms) = config.timeout_ms {
            proof_options.timeout = std::time::Duration::from_millis(timeout_ms);
        }
        if let Some(max_depth) = config.max_depth {
            proof_options.max_depth = max_depth;
        }
        if let Some(max_steps) = config.max_steps {
            proof_options.max_steps = max_steps;
        }
        if let Some(max_fresh_terms_per_quantifier) = config.max_fresh_terms_per_quantifier {
            proof_options.max_fresh_terms_per_quantifier = max_fresh_terms_per_quantifier;
        }
        if let Some(engine) = config.engine {
            proof_options.engine = cli_engine_to_search_engine(engine);
        }
    }

    if let Some(timeout_ms) = options.timeout_ms {
        proof_options.timeout = std::time::Duration::from_millis(timeout_ms);
    }
    if let Some(max_depth) = options.max_depth {
        proof_options.max_depth = max_depth;
    }
    if let Some(max_steps) = options.max_steps {
        proof_options.max_steps = max_steps;
    }
    if let Some(max_fresh_terms_per_quantifier) = options.max_fresh_terms_per_quantifier {
        proof_options.max_fresh_terms_per_quantifier = max_fresh_terms_per_quantifier;
    }
    if let Some(engine) = options.engine {
        proof_options.engine = cli_engine_to_search_engine(engine);
    }
    proof_options
}

fn cli_engine_to_search_engine(engine: CliSearchEngine) -> SearchEngine {
    match engine {
        CliSearchEngine::Naive => SearchEngine::Naive,
        CliSearchEngine::Id => SearchEngine::IterativeDeepening,
        CliSearchEngine::Priority => SearchEngine::Priority,
        CliSearchEngine::PriorityId => SearchEngine::PriorityId,
    }
}

/// Builds the biconditional input policy using CLI overrides, then config defaults.
pub(crate) fn biconditional_policy_from_cli(
    max_biconditionals: Option<usize>,
) -> BiconditionalPolicy {
    let mut policy = BiconditionalPolicy::default();
    if let Some(config) = load_config_if_present() {
        policy.max_biconditionals = config.max_biconditionals;
    }
    if let Some(max_biconditionals) = max_biconditionals {
        policy.max_biconditionals = Some(max_biconditionals);
    }
    policy
}

/// Parses `config.toml` from the current working directory.
pub(crate) fn load_config() -> Result<AppConfig, String> {
    let config_path = Path::new("config.toml");
    let config_contents = fs::read_to_string(config_path)
        .map_err(|err| format!("failed to read {}: {err}", config_path.display()))?;

    let mut tptp_root = None;
    let mut default_subset_file = None;
    let mut timeout_ms = None;
    let mut max_depth = None;
    let mut max_steps = None;
    let mut max_fresh_terms_per_quantifier = None;
    let mut max_biconditionals = None;
    let mut engine: Option<CliSearchEngine> = None;
    let mut results_db = None;

    for raw_line in config_contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');

        match key {
            "tptp_root" => tptp_root = Some(PathBuf::from(value)),
            "default_subset_file" => default_subset_file = Some(PathBuf::from(value)),
            "timeout_ms" => {
                timeout_ms = Some(
                    value
                        .parse::<u64>()
                        .map_err(|err| format!("invalid timeout_ms in config.toml: {err}"))?,
                )
            }
            "max_depth" => {
                max_depth = Some(
                    value
                        .parse::<usize>()
                        .map_err(|err| format!("invalid max_depth in config.toml: {err}"))?,
                )
            }
            "max_steps" => {
                max_steps = Some(
                    value
                        .parse::<usize>()
                        .map_err(|err| format!("invalid max_steps in config.toml: {err}"))?,
                )
            }
            "max_fresh_terms_per_quantifier" => {
                max_fresh_terms_per_quantifier = Some(value.parse::<usize>().map_err(|err| {
                    format!("invalid max_fresh_terms_per_quantifier in config.toml: {err}")
                })?)
            }
            "max_biconditionals" => {
                max_biconditionals =
                    Some(value.parse::<usize>().map_err(|err| {
                        format!("invalid max_biconditionals in config.toml: {err}")
                    })?)
            }
            "engine" => {
                engine = Some(match value {
                    "naive" => CliSearchEngine::Naive,
                    "id" => CliSearchEngine::Id,
                    "priority" => CliSearchEngine::Priority,
                    "priority-id" => CliSearchEngine::PriorityId,
                    other => {
                        return Err(format!(
                            "invalid engine in config.toml: '{other}', expected 'naive', 'id', 'priority', or 'priority-id'"
                        ))
                    }
                });
            }
            "results_db" => {
                results_db = Some(value.to_string());
            }
            _ => {}
        }
    }

    Ok(AppConfig {
        tptp_root: tptp_root.ok_or_else(|| "config.toml is missing tptp_root".to_string())?,
        default_subset_file: default_subset_file
            .ok_or_else(|| "config.toml is missing default_subset_file".to_string())?,
        timeout_ms,
        max_depth,
        max_steps,
        max_fresh_terms_per_quantifier,
        max_biconditionals,
        engine,
        results_db,
    })
}

fn classify_config_load() -> ConfigLoadState {
    let config_path = Path::new("config.toml");
    if !config_path.exists() {
        return ConfigLoadState::Missing;
    }

    match load_config() {
        Ok(config) => ConfigLoadState::Available(config),
        Err(err) => ConfigLoadState::Invalid(err),
    }
}

/// Prompts for config values and persists them as `config.toml`.
fn prompt_for_config() -> AppConfig {
    println!("No usable config.toml found. Enter values to create one.");
    let default_results_db = default_results_db_path();
    let default_results_db_display = default_results_db.display().to_string();

    let config = AppConfig {
        tptp_root: PathBuf::from(prompt("TPTP root path")),
        default_subset_file: PathBuf::from(prompt("Default subset file path")),
        timeout_ms: Some(
            prompt("Default timeout in milliseconds")
                .parse::<u64>()
                .expect("timeout_ms must be an integer"),
        ),
        max_depth: Some(
            prompt("Default max depth")
                .parse::<usize>()
                .expect("max_depth must be an integer"),
        ),
        max_steps: Some(
            prompt("Default max steps")
                .parse::<usize>()
                .expect("max_steps must be an integer"),
        ),
        max_fresh_terms_per_quantifier: Some(
            prompt("Default max fresh terms per quantifier")
                .parse::<usize>()
                .expect("max_fresh_terms_per_quantifier must be an integer"),
        ),
        max_biconditionals: None,
        engine: None,
        results_db: Some(prompt_or_default("SQLite results DB path", &default_results_db_display)),
    };

    write_config(&config).expect("failed to write config.toml");
    config
}

fn confirm_repair() -> bool {
    matches!(
        prompt("Replace or repair config.toml now? [y/N]")
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "y" | "yes"
    )
}

/// Prompts for a single config value on stdin/stdout.
fn prompt(label: &str) -> String {
    print!("{label}: ");
    io::stdout().flush().expect("stdout should flush");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("stdin should provide a value");
    input.trim().to_string()
}

/// Prompts for a value and falls back to a default when the user presses Enter.
fn prompt_or_default(label: &str, default: &str) -> String {
    let value = prompt(&format!("{label} [{default}]"));
    if value.is_empty() {
        default.to_string()
    } else {
        value
    }
}

/// Writes the config file in the repository-local TOML-like format expected by
/// the CLI.
fn write_config(config: &AppConfig) -> Result<(), String> {
    let results_db = config
        .results_db
        .as_deref()
        .unwrap_or(DEFAULT_RESULTS_DB);
    let mut contents = format!(
        "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = {}\nmax_depth = {}\nmax_steps = {}\nmax_fresh_terms_per_quantifier = {}\nresults_db = \"{}\"\n",
        config.tptp_root.display(),
        config.default_subset_file.display(),
        config
            .timeout_ms
            .unwrap_or(DEFAULT_PROVE_TIMEOUT.as_millis() as u64),
        config.max_depth.unwrap_or(DEFAULT_MAX_DEPTH),
        config.max_steps.unwrap_or(DEFAULT_MAX_STEPS),
        config
            .max_fresh_terms_per_quantifier
            .unwrap_or(DEFAULT_MAX_FRESH_TERMS_PER_QUANTIFIER),
        results_db,
    );
    if let Some(max_biconditionals) = config.max_biconditionals {
        contents.push_str(&format!("max_biconditionals = {max_biconditionals}\n"));
    }
    if let Some(engine) = config.engine {
        let engine_str = match engine {
            CliSearchEngine::Naive => "naive",
            CliSearchEngine::Id => "id",
            CliSearchEngine::Priority => "priority",
            CliSearchEngine::PriorityId => "priority-id",
        };
        contents.push_str(&format!("engine = \"{engine_str}\"\n"));
    }

    fs::write("config.toml", contents).map_err(|err| format!("failed to write config.toml: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{load_config, validate_and_merge_tptp_config, AppConfig, TptpConfigError};
    use crate::cli::args::CliSearchEngine;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn load_config_parses_expected_fields() {
        let temp_dir = std::env::temp_dir().join(format!(
            "theorem_prover_config_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be valid")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");
        fs::write(
            temp_dir.join("config.toml"),
            "tptp_root = \"..\\\\TPTP\"\ndefault_subset_file = \"subset.txt\"\ntimeout_ms = 10\nmax_depth = 20\nmax_steps = 30\nmax_fresh_terms_per_quantifier = 2\nmax_biconditionals = 12\nengine = \"id\"\n",
        )
        .expect("config should be written");

        let original_dir = std::env::current_dir().expect("cwd should exist");
        let _guard = super::CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_current_dir(&temp_dir).expect("cwd should be switched");
        let config = load_config().expect("config should parse");
        std::env::set_current_dir(original_dir).expect("cwd should be restored");

        assert_eq!(config.timeout_ms, Some(10));
        assert_eq!(config.max_depth, Some(20));
        assert_eq!(config.max_steps, Some(30));
        assert_eq!(config.max_fresh_terms_per_quantifier, Some(2));
        assert_eq!(config.max_biconditionals, Some(12));
        assert_eq!(config.engine, Some(CliSearchEngine::Id));
        assert_eq!(config.default_subset_file.to_string_lossy(), "subset.txt");
    }

    #[test]
    fn load_config_returns_error_for_invalid_engine_value() {
        let temp_dir = std::env::temp_dir().join(format!(
            "theorem_prover_config_engine_err_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be valid")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");
        fs::write(
            temp_dir.join("config.toml"),
            "tptp_root = \".\"\ndefault_subset_file = \"subset.txt\"\nengine = \"bogus\"\n",
        )
        .expect("config should be written");

        let original_dir = std::env::current_dir().expect("cwd should exist");
        let _guard = super::CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_current_dir(&temp_dir).expect("cwd should be switched");
        let result = load_config();
        std::env::set_current_dir(original_dir).expect("cwd should be restored");

        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("bogus"),
            "error message should mention the invalid value"
        );
    }

    #[test]
    fn validate_and_merge_tptp_config_prefers_cli_overrides() {
        let tptp_root = PathBuf::from("/cli/tptp");
        let subset_file = PathBuf::from("/cli/subset.txt");
        let config = AppConfig {
            tptp_root: PathBuf::from("/config/tptp"),
            default_subset_file: PathBuf::from("/config/subset.txt"),
            timeout_ms: None,
            max_depth: None,
            max_steps: None,
            max_fresh_terms_per_quantifier: None,
            max_biconditionals: None,
            engine: None,
            results_db: None,
        };

        let result = validate_and_merge_tptp_config(
            Some(&tptp_root),
            Some(&subset_file),
            Some(&config),
        );

        assert!(result.is_ok());
        let (resolved_root, resolved_file) = result.unwrap();
        assert_eq!(resolved_root, tptp_root);
        assert_eq!(resolved_file, subset_file);
    }

    #[test]
    fn validate_and_merge_tptp_config_falls_back_to_config() {
        let config = AppConfig {
            tptp_root: PathBuf::from("/config/tptp"),
            default_subset_file: PathBuf::from("/config/subset.txt"),
            timeout_ms: None,
            max_depth: None,
            max_steps: None,
            max_fresh_terms_per_quantifier: None,
            max_biconditionals: None,
            engine: None,
            results_db: None,
        };

        let result = validate_and_merge_tptp_config(None, None, Some(&config));

        assert!(result.is_ok());
        let (resolved_root, resolved_file) = result.unwrap();
        assert_eq!(resolved_root, config.tptp_root);
        assert_eq!(resolved_file, config.default_subset_file);
    }

    #[test]
    fn validate_and_merge_tptp_config_fails_when_missing_tptp_root() {
        let result = validate_and_merge_tptp_config(None, None, None);
        assert!(matches!(result, Err(TptpConfigError::MissingTptpRoot)));
    }

    #[test]
    fn validate_and_merge_tptp_config_fails_when_missing_subset_file() {
        let tptp_root = PathBuf::from("/cli/tptp");
        let result = validate_and_merge_tptp_config(Some(&tptp_root), None, None);
        assert!(matches!(result, Err(TptpConfigError::MissingSubsetFile)));
    }

    #[test]
    fn validate_and_merge_tptp_config_cli_partial_falls_back_to_config() {
        let cli_tptp_root = PathBuf::from("/cli/tptp");
        let config = AppConfig {
            tptp_root: PathBuf::from("/config/tptp"),
            default_subset_file: PathBuf::from("/config/subset.txt"),
            timeout_ms: None,
            max_depth: None,
            max_steps: None,
            max_fresh_terms_per_quantifier: None,
            max_biconditionals: None,
            engine: None,
            results_db: None,
        };

        let result = validate_and_merge_tptp_config(Some(&cli_tptp_root), None, Some(&config));

        assert!(result.is_ok());
        let (resolved_root, resolved_file) = result.unwrap();
        assert_eq!(resolved_root, cli_tptp_root);
        assert_eq!(resolved_file, config.default_subset_file);
    }

    #[test]
    fn load_config_parses_priority_engine() {
        let temp_dir = std::env::temp_dir().join(format!(
            "theorem_prover_config_priority_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be valid")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");
        fs::write(
            temp_dir.join("config.toml"),
            "tptp_root = \".\"\ndefault_subset_file = \"subset.txt\"\nengine = \"priority\"\n",
        )
        .expect("config should be written");

        let original_dir = std::env::current_dir().expect("cwd should exist");
        let _guard = super::CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_current_dir(&temp_dir).expect("cwd should be switched");
        let config = load_config().expect("priority engine should parse");
        std::env::set_current_dir(original_dir).expect("cwd should be restored");

        assert_eq!(config.engine, Some(CliSearchEngine::Priority));
    }

    #[test]
    fn load_config_parses_priority_id_engine() {
        let temp_dir = std::env::temp_dir().join(format!(
            "theorem_prover_config_priority_id_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be valid")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");
        fs::write(
            temp_dir.join("config.toml"),
            "tptp_root = \".\"\ndefault_subset_file = \"subset.txt\"\nengine = \"priority-id\"\n",
        )
        .expect("config should be written");

        let original_dir = std::env::current_dir().expect("cwd should exist");
        let _guard = super::CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_current_dir(&temp_dir).expect("cwd should be switched");
        let config = load_config().expect("priority-id engine should parse");
        std::env::set_current_dir(original_dir).expect("cwd should be restored");

        assert_eq!(config.engine, Some(CliSearchEngine::PriorityId));
    }
}

#[cfg(test)]
#[path = "config_persist_tests.rs"]
mod config_persist_tests;
