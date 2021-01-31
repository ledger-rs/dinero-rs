use std::collections::HashMap;
use std::iter::Chain;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;

use chrono::NaiveDate;
use num::rational::Rational64;

use crate::models::balance::Balance;
use crate::models::{Account, Comment, HasName, Money};
use crate::LedgerError;
use std::fmt;
use std::fmt::{Display, Formatter};

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
    pub virtual_postings_balance: Vec<PostingType>,
    pub comments: Vec<Comment>,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Copy, Clone)]
pub enum TransactionStatus {
    NotChecked,
    InternallyBalanced,
    Correct,
}

#[derive(Debug, Copy, Clone)]
pub enum TransactionType {
    Real,
    Automated,
    Periodic,
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Clone)]
pub struct Posting {
    pub(crate) account: Rc<Account>,
    pub amount: Option<Money>,
    pub balance: Option<Money>,
    pub cost: Option<Cost>,
    pub kind: PostingType,
}

impl Posting {
    pub fn new(account: &Account, kind: PostingType) -> Posting {
        Posting {
            account: Rc::new(account.clone()),
            amount: None,
            balance: None,
            cost: None,
            kind: kind,
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

impl<PostingType> Transaction<PostingType> {
    pub fn new(t_type: TransactionType) -> Transaction<PostingType> {
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
            virtual_postings_balance: vec![],
            comments: vec![],
            transaction_type: t_type,
        }
    }
    /// Iterator over all the postings
    pub fn postings_iter(
        &self,
    ) -> Chain<Chain<Iter<'_, PostingType>, Iter<'_, PostingType>>, Iter<'_, PostingType>> {
        self.postings
            .iter()
            .chain(self.virtual_postings.iter())
            .chain(self.virtual_postings_balance.iter())
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

    pub fn num_empty_postings(&self) -> usize {
        self.postings
            .iter()
            .filter(|p| p.amount.is_none() & p.balance.is_none())
            .collect::<Vec<&Posting>>()
            .len()
    }

    /// Balances the transaction
    /// There are two groups of postings that have to balance: the real postings and the [virtual postings]
    /// but not the (virtual postings)
    pub fn balance(
        &mut self,
        balances: &mut HashMap<Rc<Account>, Balance>,
        skip_balance_check: bool,
    ) -> Result<Balance, LedgerError> {
        let mut transaction_balance = Balance::new();
        match total_balance(self.virtual_postings_balance.as_ref()).can_be_zero() {
            true => {}
            false => return Err(LedgerError::TransactionIsNotBalanced),
        }
        // 1. update the amount of every posting if it has a balance
        let mut fill_account = &Rc::new(Account::from("this will never be used"));
        let mut postings: Vec<Posting> = Vec::new();
        for p in self.postings.iter() {
            // If it has money, update the balance
            if let Some(money) = &p.amount {
                let expected_balance =
                    balances.get(p.account.deref()).unwrap().clone() + Balance::from(money.clone());
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
                                    amount: units,
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
                    kind: PostingType::Real,
                });
            } else if &p.balance.is_some() & !skip_balance_check {
                let balance = p.balance.as_ref().unwrap();
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
                    kind: PostingType::Real,
                });
            } else {
                fill_account = &p.account;
            }
        }

        let empties = self.postings.len() - postings.len();
        if empties > 1 {
            Err(LedgerError::TooManyEmptyPostings(empties))
        } else if empties == 0 {
            match transaction_balance.can_be_zero() {
                true => {
                    self.postings = postings;
                    Ok(transaction_balance)
                }
                false => Err(LedgerError::TransactionIsNotBalanced),
            }
        } else {
            // Fill the empty posting
            // let account = &self.postings.last().unwrap().account;
            for (_, money) in (-transaction_balance).iter() {
                let expected_balance =
                    balances.get(fill_account).unwrap().clone() + Balance::from(money.clone());

                balances.insert(fill_account.clone(), expected_balance);

                postings.push(Posting {
                    account: fill_account.clone(),
                    amount: Some(money.clone()),
                    balance: None,
                    cost: None,
                    kind: PostingType::Real,
                });
            }
            self.postings = postings;
            Ok(Balance::new())
        }
    }
}

impl Display for Transaction<Posting> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut message = String::new();
        message.push_str(format!("{} {}", self.date.unwrap(), self.description).as_str());
        for p in self.postings_iter() {
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
