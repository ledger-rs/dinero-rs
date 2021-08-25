use colored::{ColoredString, Colorize};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub struct EmptyLedgerFileError;
impl Error for EmptyLedgerFileError {}
impl Display for EmptyLedgerFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "The file does not have any information")
    }
}

#[derive(Debug)]
pub enum MissingFileError {
    ConfigFileDoesNotExistError(PathBuf),
    JournalFileDoesNotExistError(PathBuf),
}
impl Error for MissingFileError{}
impl Display for MissingFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (title, file) =
        match self {
            MissingFileError::ConfigFileDoesNotExistError(x) => ("Configuration", x.to_str().unwrap()),
            MissingFileError::JournalFileDoesNotExistError(x) => ("Journal", x.to_str().unwrap()),
        };
        write!(f, "{} file does not exist: {}", title, format!("{}",file).red().bold())
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
    TransactionIsNotBalanced,
    EmptyPostingShouldBeLast,
    AliasNotInList(String),
    TooManyEmptyPostings(usize),
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
