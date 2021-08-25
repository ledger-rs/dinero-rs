use std::convert::TryFrom;
use std::ops::Deref;

use crate::models::Ledger;
use crate::CommonOpts;
use crate::GenericError;

pub fn execute(options: &CommonOpts, maybe_ledger: Option<Ledger>) -> Result<(), GenericError> {
    let ledger = match maybe_ledger {
        Some(ledger) => ledger,
        None => Ledger::try_from(options)?,
    };
    for price in ledger.prices.deref() {
        println!("{}", price);
    }
    Ok(())
}
