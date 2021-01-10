pub use currency::{Currency};
pub use account::Account;
pub use money::{Money, Balance, Price, CostType};
pub use transaction::{Transaction, Posting, Cleared, TransactionStatus};
use crate::parser::Item;
use crate::{Error, List};
use crate::parser;
use std::collections::{HashSet, HashMap};
use std::hash::{Hash, Hasher};
use crate::ledger::transaction::Cost;

mod money;
mod currency;
mod transaction;
mod account;

/// A lib.ledger has (journal) entries. Each of those entries has postings
/// lib.ledger > entry > posting
/// Each of those postings has an amount of money (a commodity) paired with an account
/// Each entry has to be balanced
/// Commodities can change price over time
pub struct Ledger<'a> {
    transactions: Vec<Transaction<Posting<'a>>>,
}

impl<'a> Ledger<'a> {
    pub(crate) fn new(items: &Vec<Item>) -> Result<Self, Error> {
        let mut currencies = List::<Currency>::new();
        let mut accounts = List::<Account>::new();
        let mut transactions: Vec<Transaction::<Posting>> = Vec::new();
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

        println!("{:?}", &accounts);
        println!("{:?}", &currencies);

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
        for t in transactions {
            println!("{:?} {:?}", t.date.unwrap(), t.description);
            for p in t.postings.iter() {
                println!("\t{:?}    {:?}", p.account, p.amount.unwrap())
            }
        }


        todo!("not implemented");
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

pub trait FromDirective {
    fn is_from_directive(&self) -> bool;
}
