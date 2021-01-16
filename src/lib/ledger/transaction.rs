use crate::ledger::{Account, Money, Balance, Comment, Currency};
use crate::{ErrorType, parser, List, Error};
use chrono::NaiveDate;
use num::rational::Ratio;
use std::collections::HashMap;

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
    Real,
    VirtualMustBalance,
    Virtual,
}

#[derive(Debug, Clone)]
pub struct Posting<'a> {
    pub(crate) account: &'a Account<'a>,
    pub amount: Option<Money<'a>>,
    pub balance: Option<Money<'a>>,
    pub cost: Option<Cost<'a>>,
}

impl<'a> Posting<'a> {
    pub fn new(account: &'a Account<'a>) -> Posting<'a> {
        Posting {
            account,
            amount: None,
            balance: None,
            cost: None,
        }
    }
    pub fn set_amount(&mut self, money: Money<'a>) {
        self.amount = Some(money)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Cost<'a> {
    Total { amount: Money<'a> },
    PerUnit { amount: Money<'a> },
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

impl<'a> Transaction<Posting<'a>> {
    fn total_balance(&'a self) -> Balance {
        let bal = Balance::new();
        self.postings.iter()
            .filter(|p| p.amount.is_some())
            .map(|p| Balance::from(p.amount.unwrap()))
            .fold(bal, |acc, cur| acc + cur)
    }
    pub fn is_balanced(&self) -> bool {
        self.total_balance().can_be_zero()
    }
    fn balance_postings(&mut self, account: &Account) {
        let extra_postings = self.total_balance().balance.iter()
            .map(|(_, v)| Posting {
                account,
                amount: Some(-*v),
                cost: None,
                balance: None,
            })
            .collect::<Vec<Posting>>();
        self.postings.to_owned().extend(extra_postings)
    }
    pub fn add_empty_posting(&mut self, account: &'a Account<'a>) {
        self.postings.push(Posting {
            account,
            amount: None,
            cost: None,
            balance: None,
        })
    }
    pub fn num_empty_postings(&self) -> usize {
        self.postings.iter()
            .filter(|p| p.amount.is_none() & p.balance.is_none())
            .collect::<Vec<&Posting>>().len()
    }
    /// Balances the transaction
    pub fn balance(&'a mut self, balances: &mut HashMap<&Account<'a>, Balance<'a>>) -> Result<(), ErrorType> {
        // 1. update the amount of every posting if it has a balance
        let mut postings: Vec<Posting> = Vec::new();
        for p in self.postings.iter() {
            // If it has money, update the balance
            if let Some(money) = p.amount {
                let expected_balance = balances.get(p.account).unwrap().clone() + Balance::from(money);
                if let Some(balance) = p.balance {
                    if Balance::from(balance) != expected_balance {
                        return Err(ErrorType::TransactionIsNotBalanced);
                    }
                }
                balances.insert(p.account, expected_balance);
                postings.push(Posting {
                    account: p.account,
                    amount: p.amount,
                    balance: None,
                    cost: p.cost,
                });
            } else if let Some(balance) = p.balance {
                // update the amount
                let account_bal = balances.get(p.account).unwrap().clone();
                let amount_bal = Balance::from(balance) - account_bal;
                let money = amount_bal.to_money()?;
                postings.push(Posting {
                    account: p.account,
                    amount: Some(money),
                    balance: None,
                    cost: p.cost,
                });
            }
        }

        let empties = self.postings.len() - postings.len();
        if empties > 1 {
            Err(ErrorType::TooManyEmptyPostings(empties))
        } else if empties == 0 {
            self.postings = postings;
            match self.is_balanced() {
                true => {
                    self.status = TransactionStatus::InternallyBalanced;
                    Ok(())
                }
                false => Err(ErrorType::TransactionIsNotBalanced)
            }
        } else {
            // Delete the empty posting
            match self.postings.last().unwrap().amount {
                None => Err(ErrorType::EmptyPostingShouldBeLast),
                Some(_) => {
                    let account = self.postings.pop().unwrap().account;
                    self.status = TransactionStatus::InternallyBalanced;
                    self.postings = postings;
                    self.balance_postings(account);

                    Ok(())
                }
            }
        }
    }
}