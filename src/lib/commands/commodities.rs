use crate::ledger;
use crate::ledger::{Currency, HasName};
use crate::parser::Tokenizer;
use crate::Error;
use std::path::PathBuf;

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.parse()?;
    let ledger = ledger::build_ledger(&items)?;
    let mut commodities = ledger
        .currencies
        .iter()
        .map(|x| x.1.to_owned())
        .collect::<Vec<Currency>>();
    commodities.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    for cur in commodities {
        println!("{}", cur);
    }
    Ok(())
}
