use std::convert::TryFrom;
use std::ops::Deref;
use std::path::PathBuf;

use crate::models::Ledger;
use crate::parser::Tokenizer;
use crate::Error;

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize()?;

    let ledger = Ledger::try_from(items)?;
    for price in ledger.prices.deref() {
        println!("{}", price);
    }
    Ok(())
}
