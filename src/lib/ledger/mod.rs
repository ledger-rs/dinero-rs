pub use currency::{Currency};
pub use account::Account;
pub use money::{Money, Balance, Price, CostType};
pub use transaction::{Transaction, Posting, Cleared, TransactionStatus};
use crate::parser::Item;
use crate::{Error, List, ErrorType};
use crate::parser;
use std::collections::{HashSet, HashMap};
use std::hash::{Hash, Hasher};
use crate::ledger::transaction::Cost;
use colored::Colorize;
use std::mem;
use std::collections::hash_map::RandomState;

mod money;
mod currency;
mod transaction;
mod account;

/// A lib.ledger has (journal) entries. Each of those entries has postings
/// lib.ledger > entry > posting
/// Each of those postings has an amount of money (a commodity) paired with an account
/// Each entry has to be balanced
/// Commodities can change price over time
#[derive(Debug, Clone)]
pub struct LedgerElements<'a> {
    // pub transactions: Vec<Transaction<Posting<'a>>>,
    pub currencies: List<'a, Currency<'a>>,
    pub accounts: List<'a, Account<'a>>,
}

impl<'a> LedgerElements<'a> {
    pub fn new() -> LedgerElements<'a> {
        LedgerElements {
            //transactions: vec![],
            currencies: List::<Currency>::new(),
            accounts: List::<Account>::new(),
        }
    }
}

pub fn build_ledger(items: &Vec<Item>) -> Result<LedgerElements, Error> {
    let mut currencies = List::<Currency>::new();
    let mut accounts = List::<Account>::new();
    let mut prices: Vec<Price> = Vec::new();


    // 1. Populate the lists
    for item in items.iter() {
        match item {
            Item::Comment(_) => {}
            Item::Transaction(parsed) => {
                for p in parsed.postings.iter() {
                    let account = Account::from(p.account.clone());
                    accounts.push(account);

                    // Currencies
                    if let Some(c) = &p.money_currency {
                        currencies.push(Currency::from(c.as_str()));
                    }
                    if let Some(c) = &p.cost_currency {
                        let currency = Currency::from(c.as_str());
                        currencies.push(currency);
                    }
                    if let Some(c) = &p.balance_currency {
                        let currency = Currency::from(c.as_str());
                        currencies.push(currency);
                    }
                }
            }
            Item::Directive => {}
        }
    }


    return Ok(LedgerElements {
        currencies,
        accounts,
    });
}


pub fn populate_transactions<'a>(items: &Vec<Item>, elements: &'a LedgerElements<'a>) -> Result<(Vec<Transaction<Posting<'a>>>, HashMap<&'a Account<'a>, Balance<'a>>), Error> {
    let mut transactions = vec![];
    let accounts = &elements.accounts;
    let currencies = &elements.currencies;
    // 2. Get the right postings
    for item in items.iter() {
        match item {
            Item::Transaction(parsed) => {
                let mut transaction = Transaction::<Posting>::new();
                transaction.description = parsed.description.clone();
                transaction.code = parsed.code.clone();
                transaction.note = parsed.note.clone();
                transaction.date = parsed.date;
                transaction.effective_date = parsed.effective_date;

                // Go posting by posting
                for p in parsed.postings.iter() {
                    let account = accounts.get(&p.account)?;

                    let mut posting: Posting = Posting::new(account);

                    // Modify posting with amounts
                    if let Some(c) = &p.money_currency {
                        posting.amount = Some(Money::from((
                            currencies.get(&c.as_str()).unwrap(),
                            p.money_amount.unwrap()
                        )));
                    }
                    if let Some(c) = &p.cost_currency {
                        posting.cost = Some(Cost::PerUnit { // Todo Perunit or total?
                            amount: Money::from((
                                currencies.get(c.as_str()).unwrap(),
                                p.cost_amount.unwrap()
                            ))
                        });
                    }
                    if let Some(c) = &p.balance_currency {
                        posting.balance = Some(Money::from((
                            currencies.get(c.as_str()).unwrap(),
                            p.balance_amount.unwrap()
                        )));
                    }
                    transaction.postings.push(posting.to_owned());
                }
                match transaction.clone().is_balanced() {
                    true => { transaction.status = TransactionStatus::InternallyBalanced; }
                    false => {}
                }
                transactions.push(transaction);
            }
            _ => {}
        }
    }

    // Now sort the transactions vector by date
    transactions.sort_by(|a, b| a.date.unwrap().cmp(&b.date.unwrap()));

    // Populate balances
    let mut balances: HashMap<&Account, Balance> = HashMap::new();
    for account in accounts.list.values() {
        balances.insert(account, Balance::new());
    }

    Ok((transactions, balances))
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

pub trait FromDirective {
    fn is_from_directive(&self) -> bool;
}
