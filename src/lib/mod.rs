use std::fmt;
use colored::{ColoredString};
use std::fmt::Debug;
pub use list::List;

pub mod ledger;
pub mod parser;
pub mod commands;
mod list;

#[derive(Debug)]
pub enum ErrorType {
    CommodityNotInList,
    TooManyEmptyPostings(usize),
    TransactionIsNotBalanced,
    EmptyPostingShouldBeLast,
    CannotReadFile(String),
    ParserError,
    UnexpectedInput,
    IncludeLoop,
}

// #[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
    message: Vec<ColoredString>,
}

// https://medium.com/apolitical-engineering/how-do-you-impl-display-for-vec-b8dbb21d814f
struct ColoredStrings<'a>(pub &'a Vec<ColoredString>);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}\n{}", self.error_type, ColoredStrings(&self.message))
        //write!(f, "{}", "I am red".red())
    }
}

impl<'a> fmt::Display for ColoredStrings<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.iter().fold(Ok(()), |result, partial| {
            result.and_then(|_| write!(f, "{}", partial))
        })
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ColoredStrings(&self.message))
    }
}

impl From<ErrorType> for Error {
    fn from(error: ErrorType) -> Self {
        Error {
            error_type: error,
            message: vec![]
        }
    }
}