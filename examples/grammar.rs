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

    match GrammarParser::parse(Rule::journal, content.as_str()) {
        Ok(mut parsed) => {
            let mut elements = parsed.next().unwrap().into_inner();
            for element in elements {
                println!("{:?}: {}", element.as_rule(), element.as_str());
            }
        }
        Err(e) => eprintln!("{:?}", e),
    }
}
