//! Dinero (spanish for money) is a command line tool that can deal with ledger files. It is inspired but not a port of John Wiegley's wonderful [ledger-cli](https://www.ledger-cli.org/).
//!
//! Note that the crate name is ```dinero-rs```but the executable is ```dinero```
//!
//! # Getting started
//!
//! ## The input files
//!
//! ```dinero``` understands ledger files, which are human-readable journal files.
//! A journal is composed of *directives* that tell ```dinero``` about miscellaneous
//! things like accounts, payees or commodities and *transactions*, which are each of the journal entries. A transaction looks like this:
//!
//! ```ledger
//! 2021-02-28 * Flights to Rome | Alitalia
//!     Expenses:Travel             153.17 EUR
//!     Assets:Checking account    -153.17 EUR
//! ```
//!
//! A transaction (if it doesn't include virtual postings) always has to be balanced, meaning the total amount of a transaction has to be zero. ```dinero``` knows this, so elliding some information is allowed, like so:
//! ```ledger
//! 2021-02-28 * Flights to Rome | Alitalia
//!     Expenses:Travel             153.17 EUR
//!     Assets:Checking account
//! ```
//! Or you can even do multi-currency, the conversion will be implicitely done. It supports unicode too, so this is valid as well:
//!
//! ```ledger
//! 2021-02-28 * Flights to Rome | Alitalia
//!     Expenses:Travel             153.17 €
//!     Assets:Checking account     $-180
//! ```
//!
//! ## The commands
//!
//! Given an input file, reports are extracted with commands, like so:
//!
//! ```zsh
//! # A balance report
//! dinero bal -f my_journal.ledger
//!
//! # A balance report in euros
//! dinero bal -f my_journal.ledger -X €
//!
//! # Print all the registers
//! dinero reg -f my_journal.ledger
//! ```

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod app;
pub mod commands;
mod error;
pub mod filter;
mod list;
pub mod models;
pub mod parser;

pub use app::{run_app, CommonOpts};
pub(crate) use error::{Error, LedgerError, ParserError};
pub use list::List;
