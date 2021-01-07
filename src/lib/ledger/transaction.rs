use crate::ledger::{Account, Money, Balance, Comment};
use crate::ErrorType;
use chrono::NaiveDate;

#[derive(Debug)]
pub struct Transaction<PostingType> {
    status: TransactionStatus,
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

#[derive(Debug)]
enum TransactionStatus {
    NotChecked,
    InternallyBalanced,
    Correct,
}

#[derive(Debug)]
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

#[derive(Debug, Clone, Copy)]
pub struct Posting<'a> {
    account: &'a Account,
    amount: Option<Money<'a>>,
    cost: Option<Cost<'a>>,
}
impl<'a> Posting<'a> {
    fn set_amount(&mut self, money: Money<'a>) {
        self.amount = Some(money)
    }
}

#[derive(Debug, Clone, Copy)]
enum Cost<'a> {
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
    fn total_balance(&self) -> Balance {
        let bal = Balance::new();
        self.postings.iter()
            .map(|p| Balance::from(p.amount.unwrap()))
            .fold(bal, |acc, cur| acc + cur)
    }
    fn is_balanced(&self) -> bool {
        self.total_balance().can_be_zero()
    }
    fn balance_postings(&self, account: &'a Account) -> Vec<Posting> {
        self.total_balance().balance.iter()
            .map(|(_, v)| Posting {
                account,
                amount: Some(-*v),
                cost: None,
            })
            .collect::<Vec<Posting>>()
            .clone()
    }
    pub fn add_empty_posting(&mut self, account: &'a Account) {
        self.postings.push(Posting {
            account,
            amount: None,
            cost: None,
        })
    }
    fn num_empty_postings(&self) -> usize {
        self.postings.iter()
            .filter(|p| p.amount.is_none())
            .collect::<Vec<&Posting>>().len()
    }
    pub fn balance(&'a mut self) -> Result<(), ErrorType> {
        let empties = self.num_empty_postings();
        if empties > 1 {
            Err(ErrorType::TooManyEmptyPostings(empties))
        } else if empties == 0 {
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
                    let extra_postings = self.balance_postings(account);
                    self.postings.to_owned().extend(extra_postings);
                    self.status = TransactionStatus::InternallyBalanced;
                    Ok(())
                }
            }
        }
    }
}