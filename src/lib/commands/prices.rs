use std::convert::TryFrom;
use std::ops::Deref;
use std::path::PathBuf;

use crate::models::{Account, HasName, Ledger, Money};
use crate::parser::{ParsedLedger, Tokenizer};
use crate::Error;
use crate::{models, ParserError};

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize()?;

    let mut ledger = Ledger::try_from(items)?;
    for price in ledger.prices.deref() {
        println!("{}", price);
    }
    Ok(())
}
