use std::path::PathBuf;

use crate::models::{Account, HasName};
use crate::parser::Tokenizer;
use crate::{error::Error, CommonOpts};
use std::ops::Deref;

pub fn execute(path: PathBuf, options: &CommonOpts) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize(options);
    let ledger = items.to_ledger(options)?;
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
