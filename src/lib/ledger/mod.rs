use crate::ledger::transaction::Cost;
use crate::parser::{Item, Directive};
use crate::{Error, List};
pub use account::Account;
pub use currency::Currency;
pub use money::{Balance, CostType, Money, Price};
use std::collections::{HashMap, HashSet};
pub use transaction::{Cleared, Posting, Transaction, TransactionStatus};

mod account;
mod currency;
mod money;
mod transaction;

/// A lib.ledger has (journal) entries. Each of those entries has postings
/// lib.ledger > entry > posting
/// Each of those postings has an amount of money (a commodity) paired with an account
/// Each entry has to be balanced
/// Commodities can change price over time
#[derive(Debug, Clone)]
pub struct LedgerElements<'a> {
    // pub transactions: Vec<Transaction<Posting<'a>>>,
    pub currencies: List<Currency>,
    pub accounts: List<Account>,
    pub prices: Vec<Price<'a>>,
}

impl<'a> LedgerElements<'a> {
    pub fn new() -> LedgerElements<'a> {
        LedgerElements {
            //transactions: vec![],
            currencies: List::<Currency>::new(),
            accounts: List::<Account>::new(),
            prices: vec![]
        }
    }
}

pub fn build_ledger<'a>(items: &'a Vec<Item>) -> Result<LedgerElements, Error> {
    let mut currencies = List::<Currency>::new();
    let mut accounts = List::<Account>::new();
    let mut commodity_strs = HashSet::<String>::new();
    let mut account_strs = HashSet::<String>::new();
    let mut prices: Vec<Price> = Vec::new();

    // 1. Populate the lists
    for item in items.iter() {
        match item {
            Item::Comment(_) => {}
            Item::Transaction(parsed) => {
                for p in parsed.postings.iter() {
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
            // todo
            Item::Directive(d) => match d {
                Directive::Commodity(c) => currencies.insert(c.clone()),
                Directive::Payee { .. } => {}
                Directive::Tag { .. } => {}
                Directive::Account(c) => accounts.insert(c.clone()),
            },
            Item::Price(p) => {}
        }
    }

    // Commodities
    for alias in commodity_strs {
        match currencies.get(&alias) {
            Ok(cur) => {}   // do nothing
            Err(_) => currencies.insert(Currency::from(alias.as_str())),
        }
    }
    // Accounts
    for alias in account_strs {
        match accounts.get(&alias) {
            Ok(cur) => {}   // do nothing
            Err(_) => accounts.insert(Account::from(alias.as_str())),
        }
    }


    return Ok(LedgerElements {
        currencies,
        accounts,
        prices
    });
}

pub fn populate_transactions<'a>(
    items: &Vec<Item>,
    elements: &'a LedgerElements,
) -> Result<
    (
        Vec<Transaction<Posting<'a>>>,
        HashMap<&'a Account, Balance<'a>>,
    ),
    Error,
> {
    let mut transactions = vec![];
    let accounts = &elements.accounts;
    let currencies = &elements.currencies;
    let prices = &elements.prices;

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
                            p.money_amount.unwrap(),
                        )));
                    }
                    if let Some(c) = &p.cost_currency {
                        posting.cost = Some(Cost::PerUnit {
                            // Todo Perunit or total?
                            amount: Money::from((
                                currencies.get(c.as_str()).unwrap(),
                                p.cost_amount.unwrap(),
                            )),
                        });
                    }
                    if let Some(c) = &p.balance_currency {
                        posting.balance = Some(Money::from((
                            currencies.get(c.as_str()).unwrap(),
                            p.balance_amount.unwrap(),
                        )));
                    }
                    transaction.postings.push(posting.to_owned());
                }
                match transaction.clone().is_balanced() {
                    true => {
                        transaction.status = TransactionStatus::InternallyBalanced;
                    }
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
    for account in accounts.values() {
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

pub trait HasAliases {
    fn get_aliases(&self) -> &HashSet<String>;
}

pub trait FromDirective {
    fn is_from_directive(&self) -> bool;
}
