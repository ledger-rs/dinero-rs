use crate::models::{Account, Money, Payee};
use chrono::NaiveDate;

use colored::{Color, Styles};
use num::BigRational;
use regex::Regex;
use std::cmp::Ordering;
use std::rc::Rc;
use std::{
    borrow::Borrow,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone)]
pub enum EvalResult {
    Number(BigRational),
    Money(Money),
    Boolean(bool),
    Account(Rc<Account>),
    Payee(Rc<Payee>),
    Regex(Regex),
    String(String),
    MaybeString(Option<String>),
    Date(NaiveDate),
    Usize(usize),
    Style(Styles),
    Color(Color),
    Note,
    Result(Box<Option<EvalResult>>),
}

/// Only use this for >, <, >=, <=, not for equality
impl PartialOrd for EvalResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            EvalResult::Number(left) => match other {
                EvalResult::Number(right) => Some(left.cmp(right)),
                EvalResult::Money(right) => Some(left.cmp(right.get_amount().borrow())),
                _ => None,
            },
            EvalResult::Money(left) => match other {
                EvalResult::Number(right) => Some(left.get_amount().cmp(right)),
                EvalResult::Money(right) => Some(left.cmp(&right)),
                _ => None,
            },
            EvalResult::String(left) => match other {
                EvalResult::String(right) => Some(left.cmp(&right)),
                _ => None,
            },
            EvalResult::Date(left) => match other {
                EvalResult::Date(right) => Some(left.cmp(&right)),
                _ => None,
            },
            _ => None,
        }
    }
}

impl PartialEq for EvalResult {
    fn eq(&self, other: &Self) -> bool {
        match self {
            EvalResult::Number(left) => match other {
                EvalResult::Number(right) => left == right,
                EvalResult::Money(right) => left == &right.get_amount(),
                x => panic!("Expected number, found {:?}", x),
            },
            EvalResult::Money(left) => match other {
                EvalResult::Number(right) => &left.get_amount() == right,
                EvalResult::Money(right) => left == right,
                x => panic!("Can't compare money and {:?}", x),
            },

            EvalResult::Boolean(left) => match other {
                EvalResult::Boolean(right) => left == right,
                x => panic!("Expected boolean, found {:?}", x),
            },
            EvalResult::String(left) => match other {
                EvalResult::String(right) => left == right,
                x => panic!("Expected string, found {:?}", x),
            },
            EvalResult::Date(left) => match other {
                EvalResult::Date(right) => left == right,
                x => panic!("Expected string, found {:?}", x),
            },
            x => panic!("Can't compare {:?}", x),
        }
    }
}

impl Display for EvalResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = match self {
            EvalResult::Number(x) => x.to_string(),
            EvalResult::Money(x) => x.to_string(),
            EvalResult::Boolean(x) => x.to_string(),
            EvalResult::Account(x) => x.to_string(),
            EvalResult::Payee(x) => x.to_string(),
            EvalResult::Regex(x) => x.to_string(),
            EvalResult::String(x) => x.to_string(),
            EvalResult::Date(x) => x.to_string(),
            EvalResult::Usize(x) => x.to_string(),
            EvalResult::MaybeString(x) => match x {
                Some(s) => s.to_string(),
                None => "".to_string(),
            },
            x => {
                panic!("Don't know what to do with {:?}", x)
            }
        };
        write!(f, "{}", string)
    }
}
