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
        let mut output = String::new();
        for m in self.message.iter() {
            output.push_str(m);
        }
        write!(f, "{}", output)
    }
}

impl From<ParserError> for Error {
    fn from(_: ParserError) -> Self {
        todo!()
    }
}

impl From<LedgerError> for Error {
    fn from(_: LedgerError) -> Self {
        todo!()
    }
}
