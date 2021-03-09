extern crate pest;
#[macro_use]
extern crate pest_derive;
use pest::Parser;

use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser)]
#[grammar = "grammar/grammar.pest"]
pub struct GrammarParser;

fn main() {
    let file = env::args().skip(1).next();
    let path = PathBuf::from(file.unwrap());

    let content = read_to_string(path).unwrap();

    let mut parsed = GrammarParser::parse(Rule::journal, content.as_str())
        .expect("Could not parse transaction!") // unwrap the parse result
        .next()
        .unwrap()
        .into_inner();
    while let Some(element) = parsed.next() {
        println!("{:?}: {}", element.as_rule(), element.as_str());
    }
}
