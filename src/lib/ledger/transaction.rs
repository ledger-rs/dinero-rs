use crate::ledger::{Account, Money, Balance};
use crate::ErrorType;

pub struct Transaction<'a> {
    status: TransactionStatus,
    postings: Vec::<Posting<'a>>,
    virtual_postings: Vec::<Posting<'a>>,
}

enum TransactionStatus {
    NotChecked,
    InternallyBalanced,
    Correct,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
enum Cost<'a> {
    Total { amount: Money<'a> },
    PerUnit { amount: Money<'a> },
}

impl<'a> From<Vec<Posting<'a>>> for Transaction<'a> {
    fn from(postings: Vec<Posting<'a>>) -> Self {
        Transaction { status: TransactionStatus::NotChecked, postings, virtual_postings: vec![] }
    }
}

impl<'a> Transaction<'a> {
    fn total_balance(&self) -> Balance {
        let bal = Balance::new();
        self.postings.iter()
            .map(|p| Balance::from(p.amount.unwrap()))
            .fold(bal, |acc, cur| acc + cur)
    }
    fn is_balanced(&self) -> bool {
        self.total_balance().can_be_zero()
    }
    fn balance_postings(& self, account: &'a Account) -> Vec<Posting> {
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
                },
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