pub use currency::{Currency};
pub use money::{Money, Balance, Price,CostType};
pub use transaction::{Transaction, Posting, Cleared};
use crate::parser::Item;
use crate::Error;

mod money;
mod currency;
mod transaction;

/// A lib.ledger has (journal) entries. Each of those entries has postings
/// lib.ledger > entry > posting
/// Each of those postings has an amount of money (a commodity) paired with an account
/// Each entry has to be balanced
/// Commodities can change price over time
pub struct Ledger<'a> {
    transactions: Vec<Transaction<Posting<'a>>>,
}

impl <'a>Ledger<'a> {
    pub(crate) fn new(items: Vec<Item>) -> Result<Self, Error> {
        // 1. Parse all the transactions
        for item in items {
            match item {
                Item::Comment(_) => {}
                Item::Transaction(_) => {}
                Item::Directive => {}
            }
        }
        todo!("not implemented");
    }
}
#[derive(Debug)]
pub struct Account;

#[derive(Debug)]
pub struct Comment {
    pub comment: String,
}
