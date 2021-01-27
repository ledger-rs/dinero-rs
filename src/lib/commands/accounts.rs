use std::path::PathBuf;

use crate::error::Error;
use crate::models::{Account, HasName, Ledger};
use crate::parser::Tokenizer;
use std::convert::TryFrom;
use std::ops::Deref;

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize()?;
    let ledger = Ledger::try_from(items)?;
    let mut accounts = ledger
        .accounts
        .iter()
        .map(|x| x.1.deref().to_owned())
        .collect::<Vec<Account>>();
    accounts.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    for acc in accounts {
        println!("{}", acc);
    }
    Ok(())
}
