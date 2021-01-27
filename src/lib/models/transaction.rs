use std::collections::HashMap;

use chrono::NaiveDate;
use num::rational::Rational64;

use crate::models::balance::Balance;
use crate::models::{Account, Comment, Money};
use crate::{LedgerError, ParserError};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Transaction<PostingType> {
    pub status: TransactionStatus,
    pub date: Option<NaiveDate>,
    pub effective_date: Option<NaiveDate>,
    pub cleared: Cleared,
    pub code: Option<String>,
    pub description: String,
    pub note: Option<String>,
    pub postings: Vec<PostingType>,
    pub virtual_postings: Vec<PostingType>,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Copy, Clone)]
pub enum TransactionStatus {
    NotChecked,
    InternallyBalanced,
    Correct,
}

#[derive(Debug, Copy, Clone)]
pub enum Cleared {
    Unknown,
    NotCleared,
    Cleared,
}

#[derive(Debug, Clone, Copy)]
pub enum PostingType {
    // todo Real,
// todo VirtualMustBalance,
// todo Virtual,
}

#[derive(Debug, Clone)]
pub struct Posting {
    pub(crate) account: Rc<Account>,
    pub amount: Option<Money>,
    pub balance: Option<Money>,
    pub cost: Option<Cost>,
}

impl<'a> Posting {
    pub fn new(account: &Account) -> Posting {
        Posting {
            account: Rc::new(account.clone()),
            amount: None,
            balance: None,
            cost: None,
        }
    }
    pub fn set_amount(&mut self, money: Money) {
        self.amount = Some(money)
    }
}

#[derive(Debug, Clone)]
pub enum Cost {
    Total { amount: Money },
    PerUnit { amount: Money },
}

impl<'a, PostingType> Transaction<PostingType> {
    pub fn new() -> Transaction<PostingType> {
        Transaction {
            status: TransactionStatus::NotChecked,
            date: None,
            effective_date: None,
            cleared: Cleared::Unknown,
            code: None,
            description: "".to_string(),
            note: None,
            postings: vec![],
            virtual_postings: vec![],
            comments: vec![],
        }
    }
}

fn total_balance(postings: &Vec<Posting>) -> Balance {
    let bal = Balance::new();
    postings
        .iter()
        .filter(|p| p.amount.is_some())
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
                        Money::Zero => Rational64::new(0, 1),
                        Money::Money { amount, .. } => amount.clone(),
                    } * match p.amount.as_ref().unwrap() {
                        Money::Zero => Rational64::new(0, 1),
                        Money::Money { amount, .. } => amount.clone(),
                    };
                    let money = Money::Money {
                        amount: units
                            * (if p.amount.as_ref().unwrap().is_negative() {
                                -1
                            } else {
                                1
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
        total_balance(self.postings.as_ref()).can_be_zero()
    }
    pub fn add_empty_posting(&mut self, account: Rc<Account>) {
        self.postings.push(Posting {
            account,
            amount: None,
            cost: None,
            balance: None,
        })
    }
    pub fn num_empty_postings(&self) -> usize {
        self.postings
            .iter()
            .filter(|p| p.amount.is_none() & p.balance.is_none())
            .collect::<Vec<&Posting>>()
            .len()
    }

    /// Balances the transaction
    pub fn balance(
        &mut self,
        balances: &mut HashMap<Rc<Account>, Balance>,
    ) -> Result<(Balance), LedgerError> {
        let mut transaction_balance = Balance::new();
        // 1. update the amount of every posting if it has a balance
        let mut postings: Vec<Posting> = Vec::new();
        for p in self.postings.iter() {
            // If it has money, update the balance
            if let Some(money) = &p.amount {
                let expected_balance =
                    balances.get(p.account.deref()).unwrap().clone() + Balance::from(money.clone());
                if let Some(balance) = &p.balance {
                    if Balance::from(balance.clone()) != expected_balance {
                        return Err(LedgerError::TransactionIsNotBalanced);
                    }
                }
                balances.insert(p.account.clone(), expected_balance);
                transaction_balance = transaction_balance
                    + match &p.cost {
                        None => Balance::from(p.amount.as_ref().unwrap().clone()),
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
                                    Money::Zero => Rational64::new(0, 1),
                                    Money::Money { amount, .. } => amount.clone(),
                                } * match p.amount.as_ref().unwrap() {
                                    Money::Zero => Rational64::new(0, 1),
                                    Money::Money { amount, .. } => amount.clone(),
                                };
                                let money = Money::Money {
                                    amount: units
                                        * (if p.amount.as_ref().unwrap().is_negative() {
                                            -1
                                        } else {
                                            1
                                        }),
                                    currency: currency.clone(),
                                };
                                Balance::from(money)
                            }
                        },
                    };

                postings.push(Posting {
                    account: p.account.clone(),
                    amount: p.amount.clone(),
                    balance: None,
                    cost: p.cost.clone(),
                });
            } else if let Some(balance) = &p.balance {
                // update the amount
                let account_bal = balances.get(p.account.deref()).unwrap().clone();
                let amount_bal = Balance::from(balance.clone()) - account_bal;
                let money = amount_bal.to_money()?;
                transaction_balance = transaction_balance + Balance::from(money.clone());
                postings.push(Posting {
                    account: p.account.clone(),
                    amount: Some(money),
                    balance: None,
                    cost: p.cost.clone(),
                });
            }
        }

        let empties = self.postings.len() - postings.len();
        if empties > 1 {
            Err(LedgerError::TooManyEmptyPostings(empties))
        } else if empties == 0 {
            match transaction_balance.can_be_zero() {
                true => {
                    self.postings = postings;
                    Ok((transaction_balance))
                }
                false => Err(LedgerError::TransactionIsNotBalanced),
            }
        } else {
            // Delete the empty posting
            match self.postings.last().unwrap().amount {
                Some(_) => Err(LedgerError::EmptyPostingShouldBeLast),
                None => {
                    let account = &self.postings.last().unwrap().account;
                    let money = -transaction_balance.to_money()?;
                    postings.push(Posting {
                        account: account.clone(),
                        amount: Some(money),
                        balance: None,
                        cost: None,
                    });
                    self.postings = postings;
                    Ok((transaction_balance))
                }
            }
        }
    }
}
