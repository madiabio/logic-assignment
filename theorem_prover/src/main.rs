use pest::Parser;
use pest_derive::Parser;
use std::env;
use std::fs;

#[derive(Parser)]
#[grammar = "tptp.pest"]
struct TptpParser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <file.tptp>");
        std::process::exit(1);
    }

    let input = fs::read_to_string(&args[1])
        .expect("Failed to read input file");

    match TptpParser::parse(Rule::file, &input) {
        Ok(pairs) => {
            println!("Parse successful!");
            println!("{:#?}", pairs);
        }
        Err(e) => {
            eprintln!("Parse failed:");
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
