use std::path::PathBuf;

use colored::Colorize;

use crate::error::Error;
use crate::parser::Tokenizer;

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let parsed = tokenizer.tokenize()?;

    match parsed.to_ledger(false) {
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
