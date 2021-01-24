use std::path::PathBuf;

use crate::ledger;
use crate::ledger::{Account, HasName, Money, Price};
use crate::parser::Tokenizer;
use crate::Error;
use std::ops::Deref;

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.parse()?;
    let mut ledgerelements = ledger::build_ledger(&items)?;
    let (mut transactions, mut balances, mut prices) = ledger::populate_transactions(&items, &mut ledgerelements)?;
    // Balance the transactions
    for t in transactions.iter_mut() {
        let date = t.date.unwrap().clone();
        let balance = t.balance(&mut balances).unwrap();
        if balance.len() == 2 {
            let vec = balance.iter()
                .map(|(_, x)| x.abs())
                .collect::<Vec<Money>>();
            prices.push(Price {
                date: date,
                commodity: vec[0],
                price: vec[1],
            });
        }
    }
    for price in prices {
        println!("{}", price);
    }
    Ok(())
}
