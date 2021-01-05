use std::path::{Path, PathBuf};

pub mod ledger;
pub mod parser;

#[derive(Debug)]
pub enum Error {
    CommodityNotInList,
    TooManyEmptyPostings(usize),
    TransactionIsNotBalanced,
    EmptyPostingShouldBeLast,
    CannotReadFile { message: String },
    ParserError,
    UnexpectedInput,
    IncludeLoop(PathBuf)
}