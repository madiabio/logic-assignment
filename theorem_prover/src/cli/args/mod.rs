use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::str::FromStr;

/// Controls where (if anywhere) proof results are persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PersistOpt {
    /// Disable persistence for this run.
    Disabled,
    /// Persist to the given file path.
    Path(String),
}

impl FromStr for PersistOpt {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "false" {
            Ok(PersistOpt::Disabled)
        } else {
            Ok(PersistOpt::Path(s.to_string()))
        }
    }
}

/// Retry/display flags shared across CLI subcommands.
#[derive(Clone, Args)]
pub(crate) struct SharedRunOptions {
    /// Reprocess files that already have a `.parse_failed` marker.
    #[arg(long)]
    pub(crate) retry_parse_failed: bool,
    /// Skip inputs whose non-comment `<=>` count exceeds this limit.
    ///
    /// The gate runs before parsing so large biconditional chains can be
    /// reported as an intentional policy limit rather than consuming proof
    /// search resources. `prove` reports `unknown` with a specific reason, and
    /// `rules` skips inspection without creating a `.parse_failed` marker.
    #[arg(long)]
    pub(crate) max_biconditionals: Option<usize>,
}

/// Display-related flags shared across CLI subcommands.
#[derive(Clone, Args)]
pub(crate) struct SharedDisplayOptions {
    /// Print the constructed sequent before running the selected command.
    #[arg(long)]
    pub(crate) show_sequent: bool,
}

/// Supported CLI output formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum OutputFormat {
    Human,
    Tsv,
}

/// Proof-search strategy selectable via `--engine` or `engine` in `config.toml`.
///
/// The two variants map directly to the library's [`theorem_prover::SearchEngine`]:
/// - `naive`  → [`theorem_prover::SearchEngine::Naive`]
/// - `id`     → [`theorem_prover::SearchEngine::IterativeDeepening`]
///
/// When the flag is absent from both the command line and `config.toml`, the
/// prover falls back to [`CliSearchEngine::Naive`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum CliSearchEngine {
    /// Depth-first backward search.
    ///
    /// Explores the proof tree using a single depth-limited DFS pass.
    /// This is the default strategy.
    Naive,
    /// Iterative-deepening backward search.
    ///
    /// Repeatedly runs depth-limited DFS with depth limits 1, 2, 3, … up to
    /// the configured `--max-depth`, returning as soon as a proof is found.
    /// Guarantees that the shallowest proof is always found first.
    Id,
}

/// Top-level CLI options for the theorem prover executable.
#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Theorem prover CLI.\n\nUse `prove` to run proof search with configurable timeout, depth, step, and quantifier-fallback limits.\nUse `rules` to inspect which sequent-calculus rules apply to a problem."
)]
pub(crate) struct CliOptions {
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Supported top-level commands.
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Run the prover on a file or directory of `.p` problems.
    Prove(ProveCommand),
    /// Show which rules apply to a file or directory of `.p` problems.
    Rules(RulesCommand),
}

/// Arguments for the `prove` subcommand.
#[derive(Clone, Args)]
pub(crate) struct ProveCommand {
    #[command(flatten)]
    pub(crate) run: SharedRunOptions,
    #[command(flatten)]
    pub(crate) display: SharedDisplayOptions,
    /// Wall-clock timeout in milliseconds.
    #[arg(long)]
    pub(crate) timeout_ms: Option<u64>,
    /// Maximum recursive proof-search depth before returning `Unknown`.
    ///
    /// Use this to cap branch nesting during backward search. When this bound
    /// triggers, the output detail reports `max_depth`.
    #[arg(long)]
    pub(crate) max_depth: Option<usize>,
    /// Maximum proof-search steps before returning `Unknown`.
    ///
    /// Use this to cap total search work across one proof attempt. When this
    /// bound triggers, the output detail reports `max_steps`.
    #[arg(long)]
    pub(crate) max_steps: Option<usize>,
    /// Maximum fresh fallback terms allowed per quantified occurrence.
    ///
    /// This bounds how many fresh witness or instance terms search may invent
    /// after visible terms have been reused. When this bound triggers, the
    /// output detail reports `quantifier_budget`.
    #[arg(long)]
    pub(crate) max_fresh_terms_per_quantifier: Option<usize>,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub(crate) format: OutputFormat,
    /// Path to the TPTP-v9.x.x root directory.
    ///
    /// This overrides the `tptp_root` setting from config.toml. When provided,
    /// must be used together with `--subset-file`. If neither is provided, the
    /// tool falls back to config.toml. If config.toml is missing or incomplete,
    /// the tool will prompt or exit with an error.
    #[arg(long, value_name = "PATH")]
    pub(crate) tptp_root: Option<PathBuf>,
    /// Path to the subset file describing which TPTP problems to process.
    ///
    /// This overrides the `default_subset_file` setting from config.toml.
    /// When provided, must be used together with `--tptp-root`. If neither
    /// is provided, the tool falls back to config.toml. If config.toml is
    /// missing or incomplete, the tool will prompt or exit with an error.
    #[arg(long, value_name = "PATH")]
    pub(crate) subset_file: Option<PathBuf>,
    /// Proof-search strategy: `naive` for depth-first, `id` for iterative deepening.
    ///
    /// When omitted, falls back to the `engine` key in `config.toml`, or `naive`
    /// if that is also absent.
    #[arg(long = "engine", value_enum)]
    pub(crate) engine: Option<CliSearchEngine>,
    /// SQLite DB path to persist results, or "false" to disable. Defaults to config.toml value.
    #[arg(long, value_name = "PATH|false")]
    pub(crate) persist: Option<PersistOpt>,
    /// Human-readable label for this run stored in the DB.
    #[arg(long, value_name = "LABEL")]
    pub(crate) run_label: Option<String>,
    /// Input `.p` file or directory of `.p` files to prove.
    pub(crate) target: Option<String>,
}

/// Arguments for the `rules` subcommand.
#[derive(Clone, Args)]
pub(crate) struct RulesCommand {
    #[command(flatten)]
    pub(crate) run: SharedRunOptions,
    #[command(flatten)]
    pub(crate) display: SharedDisplayOptions,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub(crate) format: OutputFormat,
    /// Path to the TPTP-v9.x.x root directory.
    ///
    /// This overrides the `tptp_root` setting from config.toml. When provided,
    /// must be used together with `--subset-file`. If neither is provided, the
    /// tool falls back to config.toml. If config.toml is missing or incomplete,
    /// the tool will prompt or exit with an error.
    #[arg(long, value_name = "PATH")]
    pub(crate) tptp_root: Option<PathBuf>,
    /// Path to the subset file describing which TPTP problems to process.
    ///
    /// This overrides the `default_subset_file` setting from config.toml.
    /// When provided, must be used together with `--tptp-root`. If neither
    /// is provided, the tool falls back to config.toml. If config.toml is
    /// missing or incomplete, the tool will prompt or exit with an error.
    #[arg(long, value_name = "PATH")]
    pub(crate) subset_file: Option<PathBuf>,
    /// Input `.p` file or directory of `.p` files to inspect.
    pub(crate) target: Option<String>,
}

/// Shared access to retry-parse-failed semantics across subcommands.
pub(crate) trait ParseFailureOptions {
    fn retry_parse_failed(&self) -> bool;
}

impl ParseFailureOptions for ProveCommand {
    fn retry_parse_failed(&self) -> bool {
        self.run.retry_parse_failed
    }
}

impl ParseFailureOptions for RulesCommand {
    fn retry_parse_failed(&self) -> bool {
        self.run.retry_parse_failed
    }
}

#[cfg(test)]
mod args_persist_tests;
