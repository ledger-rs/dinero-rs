use std::path::PathBuf;

use crate::ledger;
use crate::ledger::{Account, HasName};
use crate::parser::Tokenizer;
use crate::Error;
use std::ops::Deref;

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.parse()?;
    let ledger = ledger::build_ledger(&items)?;
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
