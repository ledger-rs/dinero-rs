use crate::parser::{Tokenizer, Item};
use crate::Error;
use std::path::PathBuf;
use colored::Colorize;

pub fn execute(file: &str) -> Result<(), Error> {
    let path = PathBuf::from(file);
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let res = tokenizer.parse();
    match res {
        Ok(_) => println!("Input file {} is {}", file.bold(), "OK".bright_green().bold()),
        Err(_) => println!("Input file {} is {}", file.bold(), "KO".bright_red().bold())
    }
    Ok(())
}