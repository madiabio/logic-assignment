mod cli;

use clap::Parser;
use env_logger::Target;
use cli::{CliOptions, Command, run_prover_mode, run_rules_mode};

fn main() {
    // init a logger
    env_logger::Builder::new().target(Target::Stdout).init();

    match CliOptions::parse().command {
        Command::Prove(options) => run_prover_mode(&options),
        Command::Rules(options) => run_rules_mode(&options),
    }
}
