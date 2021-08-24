use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use chrono::NaiveDate;
use num::rational::BigRational;

use crate::models::balance::Balance;
use crate::models::{Account, Comment, HasName, Money, Payee};
use crate::{LedgerError, List};
use num::BigInt;
use std::fmt;
use std::fmt::{Display, Formatter};

use super::Tag;
use crate::filter::preprocess_query;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Transaction<PostingType> {
    pub status: TransactionStatus,
    pub date: Option<NaiveDate>,
    pub effective_date: Option<NaiveDate>,
    pub cleared: Cleared,
    pub code: Option<String>,
    pub description: String,
    pub payee: Option<String>,
    pub postings: RefCell<Vec<PostingType>>,
    pub comments: Vec<Comment>,
    pub transaction_type: TransactionType,
    pub tags: Vec<Tag>,
    filter_query: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Posting {
    pub(crate) account: Rc<Account>,
    pub date: NaiveDate,
    pub amount: Option<Money>,
    pub balance: Option<Money>,
    pub cost: Option<Cost>,
    pub kind: PostingType,
    pub comments: Vec<Comment>,
    pub tags: RefCell<Vec<Tag>>,
    pub payee: Option<Rc<Payee>>,
    pub transaction: RefCell<Weak<Transaction<Posting>>>,
    pub origin: PostingOrigin,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PostingOrigin {
    FromTransaction,
    Automated,
    Periodic,
}

impl<T> Transaction<T> {
    pub fn get_filter_query(&mut self) -> String {
        match self.filter_query.clone() {
            None => {
                let mut parts: Vec<String> = vec![];
                let mut current = String::new();
                let mut in_regex = false;
                let mut in_string = false;
                for c in self.description.chars() {
                    if (c == ' ') & !in_string & !in_regex {
                        parts.push(current.clone());
                        current = String::new();
                    }
                    if c == '"' {
                        in_string = !in_string;
                    } else if c == '/' {
                        in_regex = !in_regex;
                        current.push(c);
                    } else {
                        current.push(c)
                    }
                }
                parts.push(current.clone());
                //self.description.split(' ').map(|x| x.to_string()).collect();
                let res = preprocess_query(&parts, &false);
                self.filter_query = Some(res.clone());
                res
            }
            Some(x) => x,
        }
    }
    pub fn get_payee(&self, payees: &List<Payee>) -> Option<Rc<Payee>> {
        match &self.payee {
            Some(payee) => match payees.get(payee) {
                Ok(x) => Some(x.clone()),
                Err(_) => panic!("Couldn't find payee {}", payee),
            },
            None => match payees.get(&self.description) {
                Ok(x) => Some(x.clone()),
                Err(_) => None,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    NotChecked,
    InternallyBalanced,
    Correct,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Real,
    Automated,
    Periodic,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Cleared {
    Unknown,
    NotCleared,
    Cleared,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PostingType {
    Real,
    Virtual,
    VirtualMustBalance,
}

impl Posting {
    pub fn new(
        account: &Rc<Account>,
        kind: PostingType,
        payee: &Payee,
        origin: PostingOrigin,
        date: NaiveDate,
    ) -> Posting {
        Posting {
            account: account.clone(),
            amount: None,
            date,
            balance: None,
            cost: None,
            kind: kind,
            comments: vec![],
            tags: RefCell::new(vec![]),
            payee: Some(Rc::new(payee.clone())),
            transaction: RefCell::new(Default::default()),
            origin,
        }
    }
    pub fn set_amount(&mut self, money: Money) {
        self.amount = Some(money)
    }
    pub fn has_tag(&self, regex: Regex) -> bool {
        for t in self.tags.borrow().iter() {
            if regex.is_match(t.get_name()) {
                return true;
            }
        }
        false
    }
    pub fn get_tag(&self, regex: Regex) -> Option<String> {
        for t in self.tags.borrow().iter() {
            if regex.is_match(t.get_name()) {
                return t.value.clone();
            }
        }
        None
    }
    pub fn get_exact_tag(&self, regex: String) -> Option<String> {
        for t in self.tags.borrow().iter() {
            if regex.as_str() == t.get_name() {
                return t.value.clone();
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cost {
    Total { amount: Money },
    PerUnit { amount: Money },
}

impl<PostingType> Transaction<PostingType> {
    pub fn new(t_type: TransactionType) -> Transaction<PostingType> {
        Transaction {
            status: TransactionStatus::NotChecked,
            date: None,
            effective_date: None,
            cleared: Cleared::Unknown,
            code: None,
            description: "".to_string(),
            payee: None,
            postings: RefCell::new(vec![]),
            comments: vec![],
            transaction_type: t_type,
            tags: vec![],
            filter_query: None,
        }
    }
}

fn total_balance(postings: &Vec<Posting>, kind: PostingType) -> Balance {
    let bal = Balance::new();
    postings
        .iter()
        .filter(|p| p.amount.is_some() & (p.kind == kind))
        .map(|p| match &p.cost {
            None => Balance::from(p.amount.as_ref().unwrap().clone()),
            Some(cost) => match cost {
                Cost::Total { amount } => {
                    if p.amount.as_ref().unwrap().clone().is_negative() {
                        Balance::from(-amount.clone())
                    } else {
                        Balance::from(amount.clone())
                    }
                }
                Cost::PerUnit { amount } => {
                    let currency = match amount {
                        Money::Zero => panic!("Cost has no currency"),
                        Money::Money { currency, .. } => currency,
                    };
                    let units = match amount {
                        Money::Zero => BigRational::new(BigInt::from(0), BigInt::from(1)),
                        Money::Money { amount, .. } => amount.clone(),
                    } * match p.amount.as_ref().unwrap() {
                        Money::Zero => BigRational::new(BigInt::from(0), BigInt::from(1)),
                        Money::Money { amount, .. } => amount.clone(),
                    };
                    let money = Money::Money {
                        amount: units
                            * (if p.amount.as_ref().unwrap().is_negative() {
                                -BigInt::from(1)
                            } else {
                                BigInt::from(1)
                            }),
                        currency: currency.clone(),
                    };
                    Balance::from(money)
                }
            },
        })
        .fold(bal, |acc, cur| acc + cur)
}

impl Transaction<Posting> {
    pub fn is_balanced(&self) -> bool {
        total_balance(&*self.postings.borrow(), PostingType::Real).can_be_zero()
    }

    pub fn num_empty_postings(&self) -> usize {
        self.postings
            .borrow()
            .iter()
            .filter(|p| p.amount.is_none() & p.balance.is_none())
            .collect::<Vec<&Posting>>()
            .len()
    }

    /// Balances the transaction
    /// There are two groups of postings that have to balance:
    /// - the *real postings*
    /// - the *virtual postings* that must balance, but not the virtual postings
    /// Real postings can have things like cost or balance assertions. However virtual postings
    /// can't.
    ///
    /// Because balance assertions depend on previous transactions, this function receives a
    /// balances Hashmap as a parameter. If the skip_balance_check flag is set to true, balance
    /// assertions are skipped.
    pub fn balance(
        &mut self,
        balances: &mut HashMap<Rc<Account>, Balance>,
        skip_balance_check: bool,
    ) -> Result<Balance, LedgerError> {
        let mut transaction_balance = Balance::new();

        // 1. Check the virtual postings
        match total_balance(&*self.postings.borrow(), PostingType::VirtualMustBalance).can_be_zero()
        {
            true => {}
            false => return Err(LedgerError::TransactionIsNotBalanced),
        }

        // 1. Iterate over postings
        let mut fill_account = Rc::new(Account::from("this will never be used"));
        let mut fill_payee = None;
        let mut fill_date: NaiveDate = NaiveDate::from_ymd(1900, 1, 1); // it will be overwritten
        let mut postings: Vec<Posting> = Vec::new();

        for p in self.postings.get_mut().iter() {
            if p.kind != PostingType::Real {
                continue;
            }
            // If it has money, update the balance
            if let Some(money) = &p.amount {
                let expected_balance = balances.get(p.account.deref()).unwrap().clone()  // What we had 
                    + Balance::from(money.clone()); // What we add
                if !skip_balance_check {
                    if let Some(balance) = &p.balance {
                        if Balance::from(balance.clone()) != expected_balance {
                            eprintln!("Found:       {}", balance);
                            eprintln!("Expected:    {}", expected_balance);
                            eprintln!(
                                "Difference:  {}",
                                expected_balance - Balance::from(balance.clone())
                            );
                            return Err(LedgerError::TransactionIsNotBalanced);
                        }
                    }
                }

                // Update the balance of the account
                balances.insert(p.account.clone(), expected_balance);

                // Update the balance of the transaction
                transaction_balance = transaction_balance   // What we had
                    + match &p.cost {
                    None => Balance::from(money.clone()),
                    // If it has a cost, the secondary currency is added for the balance
                    Some(cost) => match cost {
                        Cost::Total { amount } => {
                            if p.amount.as_ref().unwrap().is_negative() {
                                Balance::from(-amount.clone())
                            } else {
                                Balance::from(amount.clone())
                            }
                        }
                        Cost::PerUnit { amount } => {
                            let currency = match amount {
                                Money::Zero => panic!("Cost has no currency"),
                                Money::Money { currency, .. } => currency,
                            };
                            let units = match amount {
                                Money::Zero => BigRational::from(BigInt::from(0)),
                                Money::Money { amount, .. } => amount.clone(),
                            } * match p.amount.as_ref().unwrap() {
                                Money::Zero => BigRational::from(BigInt::from(0)),
                                Money::Money { amount, .. } => amount.clone(),
                            };
                            let money = Money::Money {
                                amount: units,
                                currency: currency.clone(),
                            };
                            Balance::from(money)
                        }
                    },
                };

                // Add the posting
                postings.push(Posting {
                    account: p.account.clone(),
                    amount: p.amount.clone(),
                    date: p.date.clone(),
                    balance: p.balance.clone(),
                    cost: p.cost.clone(),
                    kind: PostingType::Real,
                    comments: p.comments.clone(),
                    tags: p.tags.clone(),
                    payee: p.payee.clone(),
                    transaction: p.transaction.clone(),
                    origin: PostingOrigin::FromTransaction,
                });
            } else if &p.balance.is_some() & !skip_balance_check {
                // There is a balance
                let balance = p.balance.as_ref().unwrap();

                // update the amount
                let account_bal = balances.get(p.account.deref()).unwrap().clone();
                let amount_bal = Balance::from(balance.clone()) - account_bal;
                let money = amount_bal.to_money()?;
                transaction_balance = transaction_balance + Balance::from(money.clone());
                // update the account balance
                balances.insert(p.account.clone(), Balance::from(balance.clone()));
                postings.push(Posting {
                    account: p.account.clone(),
                    date: p.date.clone(),
                    amount: Some(money),
                    balance: p.balance.clone(),
                    cost: p.cost.clone(),
                    kind: PostingType::Real,
                    comments: p.comments.clone(),
                    tags: p.tags.clone(),
                    payee: p.payee.clone(),
                    transaction: p.transaction.clone(),
                    origin: PostingOrigin::FromTransaction,
                });
            } else {
                // We do nothing, but this is the account for the empty post
                fill_account = p.account.clone();
                fill_payee = p.payee.clone();
                fill_date = p.date.clone();
            }
        }

        let empties = self
            .postings
            .borrow()
            .iter()
            .filter(|p| p.kind == PostingType::Real)
            .count()
            - postings.len();
        if empties > 1 {
            Err(LedgerError::TooManyEmptyPostings(empties))
        } else if empties == 0 {
            match transaction_balance.can_be_zero() {
                true => {
                    //self.postings = RefCell::new(postings);
                    postings.append(
                        &mut self
                            .postings
                            .borrow_mut()
                            .iter()
                            .filter(|p| p.kind != PostingType::Real)
                            .map(|p| p.clone())
                            .collect(),
                    );
                    self.postings.replace(postings);
                    Ok(transaction_balance)
                }
                false => Err(LedgerError::TransactionIsNotBalanced),
            }
        } else {
            // Fill the empty posting
            // let account = &self.postings.last().unwrap().account;
            for (_, money) in (-transaction_balance).iter() {
                let expected_balance = balances.get(&fill_account.clone()).unwrap().clone()
                    + Balance::from(money.clone());

                balances.insert(fill_account.clone(), expected_balance);

                postings.push(Posting {
                    account: fill_account.clone(),
                    amount: Some(money.clone()),
                    balance: None,
                    cost: None,
                    kind: PostingType::Real,
                    comments: self.comments.clone(),
                    tags: RefCell::new(self.tags.clone()),
                    payee: fill_payee.clone(),
                    date: fill_date.clone(),
                    transaction: self.postings.borrow()[0].transaction.clone(),
                    origin: PostingOrigin::FromTransaction,
                });
            }
            // self.postings = RefCell::new(postings);
            postings.append(
                &mut self
                    .postings
                    .get_mut()
                    .iter()
                    .filter(|p| p.kind != PostingType::Real)
                    .map(|p| p.clone())
                    .collect(),
            );
            self.postings.replace(postings);
            // self_postings = postings;
            Ok(Balance::new())
        }
    }
}

impl Display for Transaction<Posting> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut message = String::new();
        message.push_str(format!("{} {}", self.date.unwrap(), self.description).as_str());
        for p in self.postings.borrow().iter() {
            if p.amount.as_ref().is_some() {
                message.push_str(
                    format!(
                        "\n\t{:50}{}",
                        p.account.get_name(),
                        p.amount.as_ref().unwrap()
                    )
                    .as_str(),
                );
            } else {
                message.push_str(format!("\n\t{:50}", p.account.get_name()).as_str());
            }
        }
        write!(f, "{}", message)
    }
}
