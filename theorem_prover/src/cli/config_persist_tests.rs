use super::{AppConfig, default_results_db_path, resolve_persist_path};
use crate::cli::args::PersistOpt;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn results_db_parses_from_config_toml() {
    let _lock = super::CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let temp_dir = std::env::temp_dir().join(format!(
        "theorem_prover_results_db_test_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).expect("temp dir should be created");
    fs::write(
        temp_dir.join("config.toml"),
        "tptp_root = \".\"\ndefault_subset_file = \"subset.txt\"\nresults_db = \"./results.db\"\n",
    )
    .expect("config should be written");

    let original_dir = std::env::current_dir().expect("cwd should exist");
    std::env::set_current_dir(&temp_dir).expect("cwd should be switched");
    let config = super::load_config().expect("config should parse");
    std::env::set_current_dir(original_dir).expect("cwd should be restored");

    assert_eq!(config.results_db, Some("./results.db".to_string()));
}

#[test]
fn results_db_absent_yields_none() {
    let _lock = super::CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let temp_dir = std::env::temp_dir().join(format!(
        "theorem_prover_results_db_absent_test_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).expect("temp dir should be created");
    fs::write(
        temp_dir.join("config.toml"),
        "tptp_root = \".\"\ndefault_subset_file = \"subset.txt\"\n",
    )
    .expect("config should be written");

    let original_dir = std::env::current_dir().expect("cwd should exist");
    std::env::set_current_dir(&temp_dir).expect("cwd should be switched");
    let config = super::load_config().expect("config should parse");
    std::env::set_current_dir(original_dir).expect("cwd should be restored");

    assert_eq!(config.results_db, None);
}

#[test]
fn resolve_persist_path_disabled_returns_none() {
    let config = AppConfig {
        tptp_root: PathBuf::from("."),
        default_subset_file: PathBuf::from("subset.txt"),
        timeout_ms: None,
        max_depth: None,
        max_steps: None,
        max_fresh_terms_per_quantifier: None,
        max_biconditionals: None,
        engine: None,
        results_db: Some("./results.db".to_string()),
    };

    let result = resolve_persist_path(Some(&PersistOpt::Disabled), Some(&config));
    assert_eq!(result, None);
}

#[test]
fn resolve_persist_path_with_path_returns_path() {
    let config = AppConfig {
        tptp_root: PathBuf::from("."),
        default_subset_file: PathBuf::from("subset.txt"),
        timeout_ms: None,
        max_depth: None,
        max_steps: None,
        max_fresh_terms_per_quantifier: None,
        max_biconditionals: None,
        engine: None,
        results_db: None,
    };

    let result = resolve_persist_path(Some(&PersistOpt::Path("./custom.db".to_string())), Some(&config));
    assert_eq!(result, Some(PathBuf::from("./custom.db")));
}

#[test]
fn resolve_persist_path_none_uses_config() {
    let config = AppConfig {
        tptp_root: PathBuf::from("."),
        default_subset_file: PathBuf::from("subset.txt"),
        timeout_ms: None,
        max_depth: None,
        max_steps: None,
        max_fresh_terms_per_quantifier: None,
        max_biconditionals: None,
        engine: None,
        results_db: Some("./results.db".to_string()),
    };

    let result = resolve_persist_path(None, Some(&config));
    assert_eq!(result, Some(PathBuf::from("./results.db")));
}

#[test]
fn resolve_persist_path_none_without_config_uses_default() {
    let result = resolve_persist_path(None, None);
    assert_eq!(result, Some(default_results_db_path()));
}
