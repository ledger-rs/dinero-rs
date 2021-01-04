pub mod ledger;
mod parser;

#[derive(Debug)]
pub enum Error {
    CommodityNotInList,
    TooManyEmptyPostings(usize),
    TransactionIsNotBalanced,
    EmptyPostingShouldBeLast,
    CannotReadFile { message: String },
    ParserError,
}