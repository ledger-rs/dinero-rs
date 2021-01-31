use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

use num::rational::Rational64;

pub use account::Account;
pub use balance::Balance;
pub use currency::Currency;
pub use models::{ParsedPrice, Tag};
pub use money::Money;
pub use payee::Payee;
pub use price::{Price, PriceType};
pub use transaction::{
    Cleared, Posting, PostingType, Transaction, TransactionStatus, TransactionType,
};

use crate::models::transaction::Cost;
use crate::parser::ParsedLedger;
use crate::{Error, LedgerError, List};
use std::rc::Rc;

mod account;
mod balance;
mod currency;
mod models;
mod money;
mod payee;
mod price;
mod transaction;

/// The structure that defines everything there is in a ledger, one should be able to build a ledger
/// from Items resulting from the parsing action
#[derive(Debug, Clone)]
pub struct LedgerMasterData {
    pub(crate) accounts: List<Account>,
    pub(crate) commodities: List<Currency>,
}

#[derive(Debug, Clone)]
pub struct Ledger {
    pub(crate) accounts: List<Account>,
    pub(crate) commodities: List<Currency>,
    pub(crate) transactions: Vec<Transaction<Posting>>,
    pub(crate) prices: Vec<Price>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            accounts: List::<Account>::new(),
            prices: vec![],
            transactions: vec![],
            commodities: List::<Currency>::new(),
        }
    }
}

impl<'a> TryFrom<ParsedLedger> for Ledger {
    type Error = Error;

    /// Creates a proper ledger from a parsed ledger
    fn try_from(mut parsedledger: ParsedLedger) -> Result<Self, Self::Error> {
        let mut commodity_strs = HashSet::<String>::new();
        let mut account_strs = HashSet::<String>::new();

        //
        // 1. Populate the directive lists
        //
        for transaction in parsedledger.transactions.iter() {
            for p in transaction.postings.iter() {
                account_strs.insert(p.account.clone());

                // Currencies
                if let Some(c) = &p.money_currency {
                    commodity_strs.insert(c.clone());
                }
                if let Some(c) = &p.cost_currency {
                    commodity_strs.insert(c.clone());
                }
                if let Some(c) = &p.balance_currency {
                    commodity_strs.insert(c.clone());
                }
            }
        }
        for price in parsedledger.prices.iter() {
            commodity_strs.insert(price.clone().commodity);
            commodity_strs.insert(price.clone().other_commodity);
        }

        //
        // 2. Append to the parsed ledger commodities and accounts
        //
        // Commodities
        for alias in commodity_strs {
            match parsedledger.commodities.get(&alias) {
                Ok(_) => {} // do nothing
                Err(_) => parsedledger
                    .commodities
                    .insert(Currency::from(alias.as_str())),
            }
        }
        // Accounts
        for alias in account_strs {
            match parsedledger.accounts.get(&alias) {
                Ok(_) => {} // do nothing
                Err(_) => parsedledger.accounts.insert(Account::from(alias.as_str())),
            }
        }
        // TODO payees

        // 3. Prices from price statements
        let mut prices: Vec<Price> = Vec::new();
        for price in parsedledger.prices.iter() {
            prices.push(Price {
                date: price.date,
                commodity: Money::Money {
                    amount: Rational64::new(1, 1),
                    currency: parsedledger
                        .commodities
                        .get(price.commodity.as_str())
                        .unwrap()
                        .clone(),
                },
                price: Money::Money {
                    amount: price.other_quantity,
                    currency: parsedledger
                        .commodities
                        .get(price.other_commodity.as_str())
                        .unwrap()
                        .clone(),
                },
            });
        }

        //
        // 4. Get the right postings
        //
        let mut transactions = Vec::new();
        for parsed in parsedledger.transactions.iter() {
            match parsed.transaction_type {
                TransactionType::Real => {}
                TransactionType::Automated => {
                    eprintln!("Found automated transaction. Skipping.");
                    continue;
                }
                TransactionType::Periodic => {
                    eprintln!("Found periodic transaction. Skipping.");
                    continue;
                }
            }
            let mut transaction = Transaction::<Posting>::new(TransactionType::Real);
            transaction.description = parsed.description.clone();
            transaction.code = parsed.code.clone();
            transaction.note = parsed.note.clone();
            transaction.date = parsed.date;
            transaction.effective_date = parsed.effective_date;

            // Go posting by posting
            for p in parsed.postings.iter() {
                let account = parsedledger.accounts.get(&p.account)?;

                let mut posting: Posting = Posting::new(account, p.kind);

                // Modify posting with amounts
                if let Some(c) = &p.money_currency {
                    posting.amount = Some(Money::from((
                        parsedledger.commodities.get(&c.as_str()).unwrap().clone(),
                        p.money_amount.unwrap(),
                    )));
                }
                if let Some(c) = &p.cost_currency {
                    let posting_currency = parsedledger
                        .commodities
                        .get(&p.money_currency.as_ref().unwrap().as_str())
                        .unwrap();
                    let amount = Money::from((
                        parsedledger.commodities.get(c.as_str()).unwrap().clone(),
                        p.cost_amount.unwrap(),
                    ));
                    posting.cost = match p.cost_type.as_ref().unwrap() {
                        PriceType::Total => Some(Cost::Total { amount }),
                        PriceType::PerUnit => Some(Cost::PerUnit { amount }),
                    };
                    prices.push(Price {
                        date: transaction.date.unwrap(),
                        commodity: match p.cost_type.as_ref().unwrap() {
                            PriceType::Total => posting.amount.as_ref().unwrap().abs(),
                            PriceType::PerUnit => {
                                Money::from((posting_currency.clone(), Rational64::new(1, 1)))
                            }
                        },
                        price: Money::from((
                            parsedledger.commodities.get(c.as_str()).unwrap().clone(),
                            p.cost_amount.unwrap(),
                        )),
                    })
                }
                if let Some(c) = &p.balance_currency {
                    posting.balance = Some(Money::from((
                        parsedledger.commodities.get(c.as_str()).unwrap().clone(),
                        p.balance_amount.unwrap(),
                    )));
                }
                match posting.kind {
                    PostingType::Real => transaction.postings.push(posting.to_owned()),
                    PostingType::Virtual => transaction.virtual_postings.push(posting.to_owned()),
                    PostingType::VirtualMustBalance => transaction
                        .virtual_postings_balance
                        .push(posting.to_owned()),
                }
            }
            match transaction.clone().is_balanced() {
                true => {
                    transaction.status = TransactionStatus::InternallyBalanced;
                }
                false => {}
            }
            transactions.push(transaction);
        }

        // Now sort the transactions vector by date
        transactions.sort_by(|a, b| a.date.unwrap().cmp(&b.date.unwrap()));

        // Populate balances
        let mut balances: HashMap<Rc<Account>, Balance> = HashMap::new();
        for account in parsedledger.accounts.values() {
            balances.insert(account.clone(), Balance::new());
        }

        // Balance the transactions
        for t in transactions.iter_mut() {
            let date = t.date.unwrap().clone();
            let balance = match t.balance(&mut balances) {
                Ok(balance) => balance,
                Err(e) => {
                    eprintln!("{:?}", t);
                    return Err(e);
                }
            };
            if balance.len() == 2 {
                let vec = balance.iter().map(|(_, x)| x.abs()).collect::<Vec<Money>>();
                prices.push(Price {
                    date,
                    commodity: vec[0].clone(),
                    price: vec[1].clone(),
                });
            }
        }

        Ok(Ledger {
            accounts: parsedledger.accounts,
            commodities: parsedledger.commodities,
            transactions,
            prices,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub comment: String,
}

#[derive(Copy, Clone, Debug)]
pub enum Origin {
    FromDirective,
    FromTransaction,
    Other,
}

pub trait HasName {
    fn get_name(&self) -> &str;
}

pub trait HasAliases {
    fn get_aliases(&self) -> &HashSet<String>;
}

pub trait FromDirective {
    fn is_from_directive(&self) -> bool;
}
