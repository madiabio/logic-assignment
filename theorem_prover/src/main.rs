mod cli;

use clap::Parser;
use cli::{CliOptions, Command, run_prover_mode, run_rules_mode};
use env_logger::Target;

fn main() {
    // init a logger
    env_logger::Builder::new().target(Target::Stdout).init();

    match CliOptions::parse().command {
        Command::Prove(options) => run_prover_mode(&options),
        Command::Rules(options) => run_rules_mode(&options),
    }
}
