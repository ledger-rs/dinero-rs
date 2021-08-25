use std::convert::TryFrom;
use std::error::Error;

use crate::models::{Account, HasName, Ledger};
use crate::CommonOpts;
use std::ops::Deref;

pub fn execute(options: &CommonOpts, maybe_ledger: Option<Ledger>) -> Result<(), Box<dyn Error>> {
    let ledger = match maybe_ledger {
        Some(ledger) => ledger,
        None => Ledger::try_from(options)?,
    };
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
