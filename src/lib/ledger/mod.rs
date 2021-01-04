pub use currency::{Currency, CurrencyList};
pub use money::{Money, Balance, Price};
pub use transaction::{Transaction, Posting};
mod money;
mod currency;
mod transaction;

/// A lib.ledger has (journal) entries. Each of those entries has postings
/// lib.ledger > entry > posting
/// Each of those postings has an amount of money (a commodity) paired with an account
/// Each entry has to be balanced
/// Commodities can change price over time
struct Ledger;
pub struct Account;
