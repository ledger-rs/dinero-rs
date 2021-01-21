use colored::ColoredString;
pub use list::List;
use std::fmt;
use std::fmt::Debug;

pub mod commands;
pub mod ledger;
mod list;
pub mod parser;

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
        write!(
            f,
            "{:?} {}",
            self.error_type,
            ColoredStrings(&self.message)
        )
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
            message: vec![],
        }
    }
}
