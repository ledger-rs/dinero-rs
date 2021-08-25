use std::convert::TryFrom;
use std::error::Error;

use crate::models::Ledger;
use crate::{
    models::{Currency, HasName},
    CommonOpts,
};
use std::ops::Deref;

pub fn execute(options: &CommonOpts, maybe_ledger: Option<Ledger>) -> Result<(), Box<dyn Error>> {
    let ledger = match maybe_ledger {
        Some(ledger) => ledger,
        None => Ledger::try_from(options)?,
    };
    let mut commodities = ledger
        .commodities
        .iter()
        .map(|x| x.1.deref().to_owned())
        .collect::<Vec<Currency>>();
    commodities.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    for cur in commodities {
        println!("{}", cur);
    }
    Ok(())
}
