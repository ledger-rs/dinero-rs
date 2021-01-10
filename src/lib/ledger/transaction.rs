use crate::ledger::{Account, Money, Balance, Comment, Currency};
use crate::{ErrorType, parser, List};
use chrono::NaiveDate;
use num::rational::Ratio;

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

#[derive(Debug, Clone, Copy)]
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
    fn set_amount(&mut self, money: Money<'a>) {
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
    fn new_from(
        other: &'a mut Transaction<parser::transaction::Posting>,
        accounts: &'a mut List<Account>,
        currencies: &'a mut List<Currency<'a>>) -> Self {
        let mut t = Transaction::<Posting>::new();
        t.status = other.status;
        t.date = other.date;
        t.effective_date = other.effective_date;
        t.cleared = other.cleared;
        t.code = match &other.code {
            None => None,
            Some(x) => Some(x.clone())
        };
        t.description = other.description.clone();

        t.note = match &other.note {
            None => None,
            Some(x) => Some(x.clone())
        };
        t.comments = Vec::new();
        t.comments.append(&mut other.comments);
        // for p in other.postings.iter() {
        //     let mut account = match accounts.get(p.account.as_str()) {
        //         None => {
        //             let account = Account::from(p.account);
        //             accounts.add_element(&account);
        //             &account
        //         }
        //         Some(x) => x
        //     };
        //
        //     let amount =
        //         match p.money_amount {
        //             None => None,
        //             Some(amount) => {
        //                 let mut currency = currencies.get(p.money_currency.unwrap().as_str())
        //                     .unwrap_or(&Currency::from(p.money_currency.unwrap().as_str()));
        //                 currencies.add_element(currency);
        //                 Some(Money::from((currency, amount)))
        //             }
        //         };
        //
        //     t.postings.push(Posting {
        //         account: &account,
        //         amount,
        //         cost: None,
        //     })
        // }
        t
    }
    fn total_balance(&self) -> Balance {
        let bal = Balance::new();
        self.postings.iter()
            .map(|p| Balance::from(p.amount.unwrap()))
            .fold(bal, |acc, cur| acc + cur)
    }
    pub fn is_balanced(&self) -> bool {
        self.total_balance().can_be_zero()
    }
    fn balance_postings(&'a self, account: &'a Account<'a>) -> Vec<Posting> {
        self.total_balance().balance.iter()
            .map(|(_, v)| Posting {
                account,
                amount: Some(-*v),
                cost: None,
                balance: None,
            })
            .collect::<Vec<Posting>>()
            .clone()
    }
    pub fn add_empty_posting(&mut self, account: &'a Account<'a>) {
        self.postings.push(Posting {
            account,
            amount: None,
            cost: None,
            balance: None,
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