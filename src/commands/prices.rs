use std::ops::Deref;
use std::path::PathBuf;

use crate::{CommonOpts, parser::Tokenizer};
use crate::Error;

pub fn execute(path: PathBuf, options: &CommonOpts) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize();

    let ledger = items.to_ledger(options)?;
    for price in ledger.prices.deref() {
        println!("{}", price);
    }
    Ok(())
}
