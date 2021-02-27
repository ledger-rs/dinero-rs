use num::rational::BigRational;
use std::collections::{HashMap, HashSet};

pub use account::Account;
pub use balance::Balance;
pub use comment::Comment;
pub use currency::Currency;
pub use models::{ParsedPrice, Tag};
pub use money::Money;
pub use payee::Payee;
pub use price::conversion;
pub use price::{Price, PriceType};
pub use transaction::{
    Cleared, Posting, PostingType, Transaction, TransactionStatus, TransactionType,
};

use crate::filter::filter_expression;
use crate::parser::value_expr::build_root_node_from_expression;
use crate::parser::ParsedLedger;
use crate::parser::{tokenizers, value_expr};
use crate::{filter::filter_predicate, models::transaction::Cost};
use crate::{Error, List};
use num::BigInt;
use std::rc::Rc;

mod account;
mod balance;
mod comment;
mod currency;
mod models;
mod money;
mod payee;
mod price;
mod transaction;

#[derive(Debug, Clone)]
pub struct Ledger {
    pub accounts: List<Account>,
    pub(crate) commodities: List<Currency>,
    pub(crate) transactions: Vec<Transaction<Posting>>,
    pub(crate) prices: Vec<Price>,
    pub(crate) payees: List<Payee>,
}

impl ParsedLedger {
    /// Creates a proper ledger from a parsed ledger
    pub fn to_ledger(mut self, no_checks: bool) -> Result<Ledger, Error> {
        let mut commodity_strs = HashSet::<String>::new();
        let mut account_strs = HashSet::<String>::new();
        let mut payee_strs = HashSet::<String>::new();

        // 1. Populate the directive lists

        for transaction in self.transactions.iter() {
            for p in transaction.postings.iter() {
                account_strs.insert(p.account.clone());
                if let Some(payee) = p.payee.clone() {
                    payee_strs.insert(payee);
                }
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
        for price in self.prices.iter() {
            commodity_strs.insert(price.clone().commodity);
            commodity_strs.insert(price.clone().other_commodity);
        }

        //
        // 2. Append to the parsed ledger commodities and accounts
        //
        // Commodities
        for alias in commodity_strs {
            match self.commodities.get(&alias) {
                Ok(_) => {} // do nothing
                Err(_) => self.commodities.insert(Currency::from(alias.as_str())),
            }
        }
        // Accounts
        for alias in account_strs {
            match self.accounts.get(&alias) {
                Ok(_) => {} // do nothing
                Err(_) => self.accounts.insert(Account::from(alias.as_str())),
            }
        }

        // Payees
        let payees_copy = self.payees.clone();
        for alias in payee_strs {
            match self.payees.get(&alias) {
                Ok(_) => {} // do nothing
                Err(_) => {
                    // Payees are actually matched by regex
                    let mut matched = false;
                    let mut alias_to_add = "".to_string();
                    let mut payee_to_add = None;
                    'outer: for (_, p) in payees_copy.iter() {
                        for p_alias in p.get_aliases().iter() {
                            // println!("{:?}", p_alias); // todo delete
                            if p_alias.is_match(alias.as_str()) {
                                // self.payees.add_alias(alias.to_string(), p);
                                payee_to_add = Some(p);
                                alias_to_add = alias.to_string();
                                matched = true;
                                break 'outer;
                            }
                        }
                    }
                    if !matched {
                        self.payees.insert(Payee::from(alias.as_str()))
                    } else {
                        self.payees.add_alias(alias_to_add, payee_to_add.unwrap());
                    }
                }
            }
        }

        // 3. Prices from price statements
        let mut prices: Vec<Price> = Vec::new();
        for price in self.prices.iter() {
            prices.push(Price::new(
                price.date,
                self.commodities
                    .get(price.commodity.as_str())
                    .unwrap()
                    .clone(),
                Money::Money {
                    amount: price.other_quantity.clone(),
                    currency: self
                        .commodities
                        .get(price.other_commodity.as_str())
                        .unwrap()
                        .clone(),
                },
            ));
        }

        //
        // 4. Get the right postings
        //
        let mut transactions = Vec::new();
        let mut automated_transactions = Vec::new();

        for parsed in self.transactions.iter() {
            let (mut t, mut auto, mut new_prices) = self._transaction_to_ledger(parsed)?;
            transactions.append(&mut t);
            automated_transactions.append(&mut auto);
            prices.append(&mut new_prices);
        }

        // Now sort the transactions vector by date
        transactions.sort_by(|a, b| a.date.unwrap().cmp(&b.date.unwrap()));

        // Populate balances
        let mut balances: HashMap<Rc<Account>, Balance> = HashMap::new();
        for account in self.accounts.values() {
            balances.insert(account.clone(), Balance::new());
        }

        // Balance the transactions
        for t in transactions.iter_mut() {
            let date = t.date.unwrap().clone();
            // output_balances(&balances);
            let balance = match t.balance(&mut balances, no_checks) {
                Ok(balance) => balance,
                Err(e) => {
                    eprintln!("{}", t);
                    return Err(e.into());
                }
            };
            if balance.len() == 2 {
                let vec = balance.iter().map(|(_, x)| x.abs()).collect::<Vec<Money>>();

                let commodity = vec[0].get_commodity().unwrap().clone();
                let price = Money::Money {
                    amount: vec[1].get_amount() / vec[0].get_amount(),
                    currency: vec[1].get_commodity().unwrap().clone(),
                };

                prices.push(Price::new(date, commodity, price));
            }
        }

        // 5. Go over the transactions again and see if there is something we need to do with them
        if automated_transactions.len() > 0 {
            // Build a cache of abstract value trees, it takes time to parse expressions, so better do it only once
            let mut root_nodes = HashMap::new();
            let mut regexes = HashMap::new();
            for automated in automated_transactions.iter_mut() {
                let query = automated.get_filter_query();
                let node = build_root_node_from_expression(query.as_str(), &mut regexes);
                root_nodes.insert(query, node);
            }

            for t in transactions.iter_mut() {
                for automated in automated_transactions.iter_mut() {
                    let mut extra_postings = vec![];
                    let mut extra_virtual_postings = vec![];
                    let mut extra_virtual_postings_balance = vec![];
                    let mut matched = false;

                    for comment in automated.comments.iter() {
                        t.tags.append(&mut comment.get_tags());
                        for p in t.postings.iter_mut() {
                            p.tags.append(&mut comment.get_tags());
                        }
                    }
                    for p in t.postings_iter() {
                        let node = root_nodes.get(automated.get_filter_query().as_str());
                        if filter_expression(
                            node.unwrap(), // automated.get_filter_query().as_str(),
                            p,
                            t,
                            &mut self.commodities,
                            &mut regexes,
                        )? {
                            matched = true;

                            for auto_posting in automated.postings_iter() {
                                let account_alias = auto_posting.account.clone();
                                match self.accounts.get(&account_alias) {
                                    Ok(_) => {} // do nothing
                                    Err(_) => {
                                        self.accounts.insert(Account::from(account_alias.as_str()))
                                    }
                                }
                                let payee = if let Some(payee_alias) = &auto_posting.payee {
                                    match self.payees.get(&payee_alias) {
                                        Ok(_) => {} // do nothing
                                        Err(_) => {
                                            self.payees.insert(Payee::from(payee_alias.as_str()))
                                        }
                                    }
                                    Some(self.payees.get(&payee_alias).unwrap().clone())
                                } else {
                                    p.payee.clone()
                                };
                                let account = self.accounts.get(&account_alias).unwrap();
                                let money = match &auto_posting.money_currency {
                                    None => Some(value_expr::eval_value_expression(
                                        auto_posting.amount_expr.clone().unwrap().as_str(),
                                        p,
                                        t,
                                        &mut self.commodities,
                                        &mut regexes,
                                    )),
                                    Some(alias) => {
                                        if alias == "" {
                                            Some(Money::from((
                                                p.amount.clone().unwrap().get_commodity().unwrap(),
                                                p.amount.clone().unwrap().get_amount()
                                                    * auto_posting.money_amount.clone().unwrap(),
                                            )))
                                        } else {
                                            match self.commodities.get(&alias) {
                                                Ok(_) => {} // do nothing
                                                Err(_) => self
                                                    .commodities
                                                    .insert(Currency::from(alias.as_str())),
                                            }
                                            Some(Money::from((
                                                self.commodities.get(alias).unwrap().clone(),
                                                auto_posting.money_amount.clone().unwrap(),
                                            )))
                                        }
                                    }
                                };

                                let posting = Posting {
                                    account: account.clone(),
                                    amount: money,
                                    balance: None,
                                    cost: None,
                                    kind: auto_posting.kind,
                                    comments: vec![],
                                    tags: vec![],
                                    payee,
                                };

                                match auto_posting.kind {
                                    PostingType::Real => extra_postings.push(posting),
                                    PostingType::Virtual => extra_virtual_postings.push(posting),
                                    PostingType::VirtualMustBalance => {
                                        extra_virtual_postings_balance.push(posting)
                                    }
                                }
                            }
                        }
                    }
                    t.postings.append(&mut extra_postings);
                    t.virtual_postings.append(&mut extra_virtual_postings);
                    t.virtual_postings_balance
                        .append(&mut extra_virtual_postings_balance);
                    if matched {
                        break;
                    }
                }
            }
            // Populate balances
            let mut balances: HashMap<Rc<Account>, Balance> = HashMap::new();
            for account in self.accounts.values() {
                balances.insert(account.clone(), Balance::new());
            }

            // Balance the transactions
            for t in transactions.iter_mut() {
                let date = t.date.unwrap().clone();
                // output_balances(&balances);
                let balance = match t.balance(&mut balances, no_checks) {
                    Ok(balance) => balance,
                    Err(e) => {
                        eprintln!("{}", t);
                        return Err(e.into());
                    }
                };
                if balance.len() == 2 {
                    let vec = balance.iter().map(|(_, x)| x.abs()).collect::<Vec<Money>>();

                    let commodity = vec[0].get_commodity().unwrap().clone();
                    let price = Money::Money {
                        amount: vec[1].get_amount() / vec[0].get_amount(),
                        currency: vec[1].get_commodity().unwrap().clone(),
                    };

                    prices.push(Price::new(date, commodity, price));
                }
            }
        }
        Ok(Ledger {
            accounts: self.accounts,
            commodities: self.commodities,
            transactions,
            prices,
            payees: self.payees,
        })
    }

    fn _transaction_to_ledger(
        &self,
        parsed: &Transaction<tokenizers::transaction::RawPosting>,
    ) -> Result<
        (
            Vec<Transaction<Posting>>,
            Vec<Transaction<tokenizers::transaction::RawPosting>>,
            Vec<Price>,
        ),
        Error,
    > {
        let mut automated_transactions = vec![];
        let mut prices = vec![];
        let mut transactions = vec![];
        match parsed.transaction_type {
            TransactionType::Real => {
                let mut transaction = Transaction::<Posting>::new(TransactionType::Real);
                transaction.description = parsed.description.clone();
                transaction.code = parsed.code.clone();
                transaction.comments = parsed.comments.clone();
                transaction.date = parsed.date;
                transaction.effective_date = parsed.effective_date;

                for comment in parsed.comments.iter() {
                    transaction.tags.append(&mut comment.get_tags());
                }
                // Go posting by posting
                for p in parsed.postings.iter() {
                    let payee = match &p.payee {
                        None => transaction.get_payee_inmutable(&self.payees),
                        Some(x) => self.payees.get(x).unwrap().clone(),
                    };
                    let account = if p.account.to_lowercase().ends_with("unknown") {
                        let mut account = None;
                        for (_, acc) in self.accounts.iter() {
                            for alias in acc.payees().iter() {
                                if alias.is_match(payee.get_name()) {
                                    account = Some(acc.clone());
                                    break;
                                }
                            }
                        }
                        match account {
                            Some(x) => x,
                            None => self.accounts.get(&p.account)?.clone(),
                        }
                    } else {
                        self.accounts.get(&p.account)?.clone()
                    };
                    let mut posting: Posting = Posting::new(&account, p.kind, &payee);
                    posting.tags = transaction.tags.clone();
                    for comment in p.comments.iter() {
                        posting.tags.append(&mut comment.get_tags());
                    }

                    // Modify posting with amounts
                    if let Some(c) = &p.money_currency {
                        posting.amount = Some(Money::from((
                            self.commodities.get(&c.as_str()).unwrap().clone(),
                            p.money_amount.clone().unwrap(),
                        )));
                    }
                    if let Some(c) = &p.cost_currency {
                        let posting_currency = self
                            .commodities
                            .get(&p.money_currency.as_ref().unwrap().as_str())
                            .unwrap();
                        let amount = Money::from((
                            self.commodities.get(c.as_str()).unwrap().clone(),
                            p.cost_amount.clone().unwrap(),
                        ));
                        posting.cost = match p.cost_type.as_ref().unwrap() {
                            PriceType::Total => Some(Cost::Total {
                                amount: amount.clone(),
                            }),
                            PriceType::PerUnit => Some(Cost::PerUnit {
                                amount: amount.clone(),
                            }),
                        };
                        prices.push(Price::new(
                            transaction.date.unwrap(),
                            posting_currency.clone(),
                            Money::Money {
                                amount: p.cost_amount.clone().unwrap()
                                    / match p.cost_type.as_ref().unwrap() {
                                        PriceType::Total => {
                                            posting.amount.as_ref().unwrap().get_amount()
                                        }
                                        PriceType::PerUnit => BigRational::from(BigInt::from(1)),
                                    },
                                currency: amount.get_commodity().unwrap().clone(),
                            },
                        ))
                    }
                    if let Some(c) = &p.balance_currency {
                        posting.balance = Some(Money::from((
                            self.commodities.get(c.as_str()).unwrap().clone(),
                            p.balance_amount.clone().unwrap(),
                        )));
                    }
                    match posting.kind {
                        PostingType::Real => transaction.postings.push(posting.to_owned()),
                        PostingType::Virtual => {
                            transaction.virtual_postings.push(posting.to_owned())
                        }
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
            TransactionType::Automated => {
                // Add transaction to the automated transactions queue, we'll process them
                // later.
                automated_transactions.push(parsed.clone());
            }
            TransactionType::Periodic => {
                eprintln!("Found periodic transaction. Skipping.");
            }
        }
        Ok((transactions, automated_transactions, prices))
    }
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
