use colored::{ColoredString, Colorize};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::models::Balance;

#[derive(Debug)]
pub struct EmptyLedgerFileError;
impl Error for EmptyLedgerFileError {}
impl Display for EmptyLedgerFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "The journal file does not have any information")
    }
}

#[derive(Debug)]
pub enum MissingFileError {
    ConfigFileDoesNotExistError(PathBuf),
    JournalFileDoesNotExistError(PathBuf),
}
impl Error for MissingFileError {}
impl Display for MissingFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (title, file) = match self {
            MissingFileError::ConfigFileDoesNotExistError(x) => {
                ("Configuration", x.to_str().unwrap())
            }
            MissingFileError::JournalFileDoesNotExistError(x) => ("Journal", x.to_str().unwrap()),
        };
        write!(f, "{} file does not exist: {}", title, file.red().bold())
    }
}

#[derive(Debug)]
pub struct TimeParseError;
impl Error for TimeParseError {}
impl Display for TimeParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Couldn't parse time.")
    }
}

#[derive(Debug)]
pub enum LedgerError {
    AliasNotInList(String),
    TooManyEmptyPostings(usize),
}
impl Error for LedgerError {}
impl Display for LedgerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::AliasNotInList(x) => write!(f, "Alias not found: {}", x),
            LedgerError::TooManyEmptyPostings(x) => {
                write!(f, "{} {}", "Too many empty postings:".red(), x)
            }
        }
    }
}
#[derive(Debug)]
pub enum BalanceError {
    TransactionIsNotBalanced,
    TooManyCurrencies(Balance),
}
impl Error for BalanceError {}

impl Display for BalanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BalanceError::TransactionIsNotBalanced => {
                write!(f, "{}", "Transaction is not balanced".red())
            }

            BalanceError::TooManyCurrencies(bal) => write!(
                f,
                "Too many currencies, probably a price is missing: {}",
                bal.iter()
                    .map(|x| x.1.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}
#[derive(Debug)]
pub struct GenericError {
    pub message: Vec<ColoredString>,
}

impl Error for GenericError {}

impl Display for GenericError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ColoredStrings(&self.message))
    }
}

impl From<LedgerError> for GenericError {
    fn from(error: LedgerError) -> Self {
        eprintln!("{:?}", error);
        // TODO prettier error conversion
        GenericError { message: vec![] }
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

#[cfg(test)]
mod tests {
    use colored::Colorize;
    use structopt::StructOpt;

    use super::{EmptyLedgerFileError, LedgerError};
    use crate::{parser::Tokenizer, CommonOpts};

    #[test]
    fn error_empty_posting_last() {
        let mut tokenizer = Tokenizer::from(
            "2021-06-05 Flight
        Assets:Checking
        Expenses:Comissions
        Expenses:Travel      200 EUR"
                .to_string(),
        );
        let options = CommonOpts::from_iter(["", "-f", ""].iter());
        let parsed = tokenizer.tokenize(&options);
        let ledger = parsed.to_ledger(&options);
        assert!(ledger.is_err());
        let mut output: String = String::new();
        if let Err(err) = ledger {
            let ledger_error = err.downcast_ref::<LedgerError>().unwrap();
            match ledger_error {
                LedgerError::TooManyEmptyPostings(x) => assert_eq!(*x, 2),
                other => {
                    dbg!(other);
                    panic!("Too many empty postings");
                }
            }
            output = err.to_string();
        }
        assert_eq!(output, format!("{} 2", "Too many empty postings:".red()));
    }

    #[test]
    fn empty_file() {
        let an_error = EmptyLedgerFileError {};

        assert_eq!(
            format!("{}", an_error),
            "The journal file does not have any information"
        );
    }
}
