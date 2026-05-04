use crate::cli::args::ProveCommand;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use theorem_prover::{BiconditionalPolicy, ProofOptions};

/// Persistent defaults used by config-backed CLI runs.
#[derive(Clone, Debug)]
pub(crate) struct AppConfig {
    pub(crate) tptp_root: PathBuf,
    pub(crate) default_subset_file: PathBuf,
    pub(crate) timeout_ms: Option<u64>,
    pub(crate) max_depth: Option<usize>,
    pub(crate) max_steps: Option<usize>,
    pub(crate) max_biconditionals: Option<usize>,
}

/// Loads `config.toml` when it is valid, otherwise returns `None`.
pub(crate) fn load_config_if_present() -> Option<AppConfig> {
    load_config().ok()
}

/// Returns a usable config, prompting and writing one on first run if needed.
pub(crate) fn ensure_config() -> AppConfig {
    load_config().unwrap_or_else(|_| prompt_for_config())
}

/// Builds prover options using CLI overrides, then config defaults, then
/// library defaults.
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
    proof_options
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
    let mut max_biconditionals = None;

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
            "max_biconditionals" => {
                max_biconditionals =
                    Some(value.parse::<usize>().map_err(|err| {
                        format!("invalid max_biconditionals in config.toml: {err}")
                    })?)
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
        max_biconditionals,
    })
}

/// Prompts for config values and persists them as `config.toml`.
fn prompt_for_config() -> AppConfig {
    println!("No usable config.toml found. Enter values to create one.");

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
        max_biconditionals: None,
    };

    write_config(&config).expect("failed to write config.toml");
    config
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

/// Writes the config file in the repository-local TOML-like format expected by
/// the CLI.
fn write_config(config: &AppConfig) -> Result<(), String> {
    let default_options = ProofOptions::default();
    let mut contents = format!(
        "tptp_root = \"{}\"\ndefault_subset_file = \"{}\"\ntimeout_ms = {}\nmax_depth = {}\nmax_steps = {}\n",
        config.tptp_root.display(),
        config.default_subset_file.display(),
        config
            .timeout_ms
            .unwrap_or(default_options.timeout.as_millis() as u64),
        config.max_depth.unwrap_or(default_options.max_depth),
        config.max_steps.unwrap_or(default_options.max_steps),
    );
    if let Some(max_biconditionals) = config.max_biconditionals {
        contents.push_str(&format!("max_biconditionals = {max_biconditionals}\n"));
    }

    fs::write("config.toml", contents).map_err(|err| format!("failed to write config.toml: {err}"))
}

#[cfg(test)]
mod tests {
    use super::load_config;
    use std::fs;
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
            "tptp_root = \"..\\\\TPTP\"\ndefault_subset_file = \"subset.txt\"\ntimeout_ms = 10\nmax_depth = 20\nmax_steps = 30\nmax_biconditionals = 12\n",
        )
        .expect("config should be written");

        let original_dir = std::env::current_dir().expect("cwd should exist");
        std::env::set_current_dir(&temp_dir).expect("cwd should be switched");
        let config = load_config().expect("config should parse");
        std::env::set_current_dir(original_dir).expect("cwd should be restored");

        assert_eq!(config.timeout_ms, Some(10));
        assert_eq!(config.max_depth, Some(20));
        assert_eq!(config.max_steps, Some(30));
        assert_eq!(config.max_biconditionals, Some(12));
        assert_eq!(config.default_subset_file.to_string_lossy(), "subset.txt");
    }
}
