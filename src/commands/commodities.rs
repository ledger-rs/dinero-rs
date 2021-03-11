use crate::models::{Currency, HasName};
use crate::parser::Tokenizer;
use crate::Error;
use std::ops::Deref;
use std::path::PathBuf;

pub fn execute(path: PathBuf, no_balance_check: bool) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize();
    let ledger = items.to_ledger(no_balance_check)?;
    let mut commodities = ledger
        .commodities
        .iter()
        .map(|x| x.1.deref().to_owned())
        .collect::<Vec<Currency>>();
    commodities.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    for cur in commodities {
        println!("{}", cur);
    }
    Ok(())
}
