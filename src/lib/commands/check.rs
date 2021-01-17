use crate::parser::{Tokenizer};
use crate::Error;
use std::path::PathBuf;
use colored::Colorize;
use crate::ledger;
use crate::ledger::LedgerElements;

pub fn execute(file: &str) -> Result<(), Error> {
    let path = PathBuf::from(file);
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let mut items = tokenizer.parse()?;
    let mut ledger = ledger::build_ledger(&items);
    match ledger {
        Ok(_) => println!("Input file {} is {}", file.bold(), "OK".bright_green().bold()),
        Err(e) => {
            println!("Input file {} is {}", file.bold(), "KO".bright_red().bold());
            return Err(e);
        }
    }
    Ok(())
}