pub use currency::{Currency, CurrencyList};
pub use money::{Money, Balance, Price};
mod money;
mod currency;

/// A ledger has (journal) entries. Each of those entries has postings
/// ledger > entry > posting
/// Each of those postings has an amount of money (a commodity) paired with an account
/// Each entry has to be balanced
/// Commodities can change price over time
struct Ledger;
