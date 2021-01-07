pub use currency::{Currency, CurrencyList};
pub use money::{Money, Balance, Price,CostType};
pub use transaction::{Transaction, Posting, Cleared};

mod money;
mod currency;
pub(crate) mod transaction;

/// A lib.ledger has (journal) entries. Each of those entries has postings
/// lib.ledger > entry > posting
/// Each of those postings has an amount of money (a commodity) paired with an account
/// Each entry has to be balanced
/// Commodities can change price over time
struct Ledger;

#[derive(Debug)]
pub struct Account;

#[derive(Debug)]
pub struct Comment {
    pub comment: String,
}
