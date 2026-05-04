use clap::{Args, Parser, Subcommand, ValueEnum};

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

/// Top-level CLI options for the theorem prover executable.
#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Theorem prover CLI.\n\nUse `prove` to run proof search with configurable timeout, depth, and step limits.\nUse `rules` to inspect which sequent-calculus rules apply to a problem."
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
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub(crate) format: OutputFormat,
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
