use std::ops::Deref;
use std::path::PathBuf;

use crate::parser::Tokenizer;
use crate::Error;

pub fn execute(path: PathBuf, no_balance_check: bool) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize()?;

    let ledger = items.to_ledger(no_balance_check)?;
    for price in ledger.prices.deref() {
        println!("{}", price);
    }
    Ok(())
}
