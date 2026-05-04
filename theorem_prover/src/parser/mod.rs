mod ast_builder;

use crate::ast::Formula;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/tptp.pest"]
struct TptpParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaRecord {
    pub name: String,
    pub role: String,
    pub formula: Formula,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeDirective {
    pub path: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParsedProblem {
    pub premises: Vec<FormulaRecord>,
    pub conjecture: Option<FormulaRecord>,
    pub includes: Vec<IncludeDirective>,
}

pub fn parse_tptp(input: &str) -> Result<Pairs<'_, Rule>, pest::error::Error<Rule>> {
    TptpParser::parse(Rule::file, input)
}

pub fn parse_problem(input: &str) -> Result<ParsedProblem, pest::error::Error<Rule>> {
    let mut parsed = ParsedProblem::default();
    let pairs = parse_tptp(input)?;

    for pair in pairs {
        if pair.as_rule() != Rule::file {
            continue;
        }

        for entry in pair.into_inner() {
            if entry.as_rule() != Rule::tptp_input {
                continue;
            }

            for input in entry.into_inner() {
                match input.as_rule() {
                    Rule::annotated_formula => {
                        let record = parse_formula_record(input);
                        match record.role.as_str() {
                            "conjecture" => parsed.conjecture = Some(record),
                            "axiom" | "hypothesis" => parsed.premises.push(record),
                            _ => {}
                        }
                    }
                    Rule::include_directive => parsed.includes.push(parse_include_directive(input)),
                    _ => {}
                }
            }
        }
    }

    Ok(parsed)
}

fn parse_formula_record(pair: Pair<'_, Rule>) -> FormulaRecord {
    let fof = pair
        .into_inner()
        .next()
        .expect("annotated formula should contain fof");
    let mut inner = fof.into_inner();
    let name = inner.next().expect("fof name").as_str().to_owned();
    let role = inner.next().expect("fof role").as_str().to_owned();
    let formula_pair = inner.next().expect("fof formula");
    let formula = ast_builder::build_formula(formula_pair);

    FormulaRecord {
        name,
        role,
        formula,
    }
}

fn parse_include_directive(pair: Pair<'_, Rule>) -> IncludeDirective {
    let path = pair
        .into_inner()
        .next()
        .expect("include directive should contain a quoted path")
        .as_str()
        .trim_matches('\'')
        .to_owned();

    IncludeDirective { path }
}
