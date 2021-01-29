use colored::ColoredString;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ParserError {
    CannotReadFile(String),
    UnexpectedInput(Option<String>),
    IncludeLoop(String),
}

#[derive(Debug)]
pub enum LedgerError {
    TransactionIsNotBalanced,
    EmptyPostingShouldBeLast,
    AliasNotInList(String),
    TooManyEmptyPostings(usize),
}

#[derive(Debug)]
pub struct Error {
    pub message: Vec<ColoredString>,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ColoredStrings(&self.message))
    }
}

impl From<ParserError> for Error {
    fn from(error: ParserError) -> Self {
        match error {
            ParserError::CannotReadFile(s) => Error {
                message: vec![ColoredString::from(s.as_str())],
            },
            ParserError::IncludeLoop(s) => Error {
                message: vec![ColoredString::from(s.as_str())],
            },
            ParserError::UnexpectedInput(s) => Error {
                message: match s {
                    None => vec![],
                    Some(s) => vec![ColoredString::from(s.as_str())],
                },
            },
        }
    }
}

impl From<LedgerError> for Error {
    fn from(error: LedgerError) -> Self {
        eprintln!("{:?}", error);
        todo!()
    }
}

// https://medium.com/apolitical-engineering/how-do-you-impl-display-for-vec-b8dbb21d814f
struct ColoredStrings<'a>(pub &'a Vec<ColoredString>);

impl<'a> fmt::Display for ColoredStrings<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.iter().fold(Ok(()), |result, partial| {
            result.and_then(|_| write!(f, "{}", partial))
        })
    }
}
