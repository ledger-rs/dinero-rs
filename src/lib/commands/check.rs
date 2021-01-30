use std::path::PathBuf;

use colored::Colorize;

use crate::error::Error;
use crate::models::Ledger;
use crate::parser::Tokenizer;
use std::convert::TryFrom;

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let parsed = tokenizer.tokenize()?;

    match Ledger::try_from(parsed) {
        Ok(_) => println!(
            "Input file {} is {}",
            path.to_str().unwrap().bold(),
            "OK".bright_green().bold()
        ),
        Err(e) => {
            println!(
                "Input file {} is {}",
                path.to_str().unwrap().bold(),
                "KO".bright_red().bold()
            );
            return Err(e);
        }
    }
    Ok(())
}
