use crate::ledger::transaction::Cost;
use crate::parser::{Directive, Item, ParsedPrice};
use crate::{Error, List};
pub use account::Account;
pub use currency::Currency;
pub use money::{Balance, CostType, Money, Price};
use num::rational::Rational64;
use std::cell::RefCell;
use std::collections::hash_map::RandomState;
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
pub struct LedgerElements {
    // pub transactions: Vec<Transaction<Posting<'a>>>,
    pub currencies: List<Currency>,
    pub accounts: List<Account>,
    pub prices: Vec<ParsedPrice>,
}

impl<'a> LedgerElements {
    pub fn new() -> LedgerElements {
        LedgerElements {
            //transactions: vec![],
            currencies: List::<Currency>::new(),
            accounts: List::<Account>::new(),
            prices: vec![],
        }
    }
}

pub fn build_ledger<'a>(items: &'a Vec<Item>) -> Result<LedgerElements, Error> {
    let mut currencies = List::<Currency>::new();
    let mut accounts = List::<Account>::new();
    let mut commodity_strs = HashSet::<String>::new();
    let mut account_strs = HashSet::<String>::new();
    let mut prices_parsed: Vec<ParsedPrice> = Vec::new();

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
            Item::Price(p) => {
                commodity_strs.insert(p.clone().commodity);
                commodity_strs.insert(p.clone().other_commodity);
                prices_parsed.push(p.clone());
            }
        }
    }

    // Commodities
    for alias in commodity_strs {
        match currencies.get(&alias) {
            Ok(cur) => {} // do nothing
            Err(_) => currencies.insert(Currency::from(alias.as_str())),
        }
    }
    // Accounts
    for alias in account_strs {
        match accounts.get(&alias) {
            Ok(cur) => {} // do nothing
            Err(_) => accounts.insert(Account::from(alias.as_str())),
        }
    }

    return Ok(LedgerElements {
        currencies,
        accounts,
        prices: prices_parsed,
    });
}

pub fn populate_transactions<'a>(
    items: &Vec<Item>,
    elements: &'a mut LedgerElements,
) -> Result<
    (
        Vec<Transaction<Posting<'a>>>,
        HashMap<&'a Account, Balance<'a>, RandomState>,
        Vec<Price<'a>>,
    ),
    Error,
> {
    let mut transactions = Vec::new();
    let accounts = &elements.accounts;
    let currencies = &elements.currencies;
    let mut prices: Vec<Price> = Vec::new();

    // Update the prices
    // Prices
    for price in elements.prices.iter() {
        prices.push(Price {
            date: price.date,
            commodity: Money::Money {
                amount: Rational64::new(1, 1),
                currency: currencies.get(price.commodity.as_str())?,
            },
            price: Money::Money {
                amount: price.other_quantity,
                currency: currencies.get(price.other_commodity.as_str())?,
            },
        });
    }

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
                        let posting_currency = currencies
                            .get(&p.money_currency.as_ref().unwrap().as_str())
                            .unwrap();
                        let amount = Money::from((
                            currencies.get(c.as_str()).unwrap(),
                            p.cost_amount.unwrap(),
                        ));
                        posting.cost = match p.cost_type.as_ref().unwrap() {
                            CostType::Total => Some(Cost::Total { amount }),
                            CostType::PerUnit => Some(Cost::PerUnit { amount }),
                        };
                        prices.push(Price {
                            date: transaction.date.unwrap(),
                            commodity: match p.cost_type.as_ref().unwrap() {
                                CostType::Total => posting.amount.unwrap().abs(),
                                CostType::PerUnit => {
                                    Money::from((posting_currency, Rational64::new(1, 1)))
                                }
                            },
                            price: amount,
                        })
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

    Ok((transactions, balances, prices))
}

pub fn balance_transactions<'a>(
    transactions: &'a mut Vec<Transaction<Posting<'a>>>,
    balances: &'a mut HashMap<&'a Account, Balance<'a>, RandomState>,
    prices: &'a mut Vec<Price<'a>>,
) {
    // Balance the transactions
    for t in transactions.iter_mut() {
        let date = t.date.unwrap().clone();
        let balance = t.balance(balances).unwrap();
        if balance.len() == 2 {
            let vec = balance.iter().map(|(_, x)| x.abs()).collect::<Vec<Money>>();
            prices.push(Price {
                date: date,
                commodity: vec[0],
                price: vec[1],
            });
        }
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
