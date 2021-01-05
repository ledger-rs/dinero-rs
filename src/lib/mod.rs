use std::path::{Path, PathBuf};
use std::io;

pub mod ledger;
pub mod parser;
pub mod commands;

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
