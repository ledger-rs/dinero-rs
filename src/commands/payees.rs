use crate::models::Ledger;
use crate::GenericError;
use crate::{
    models::{HasName, Payee},
    CommonOpts,
};
use std::convert::TryFrom;
use std::ops::Deref;

pub fn execute(options: &CommonOpts, maybe_ledger: Option<Ledger>) -> Result<(), GenericError> {
    let ledger = match maybe_ledger {
        Some(ledger) => ledger,
        None => Ledger::try_from(options)?,
    };
    let mut payees = ledger
        .payees
        .iter()
        .map(|x| x.1.deref().to_owned())
        .collect::<Vec<Payee>>();
    payees.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    for payee in payees.iter() {
        println!("{}", payee);
    }
    Ok(())
}
