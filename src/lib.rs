pub mod ledger;

#[derive(Debug)]
pub enum Error {
    CommodityNotInList,
    TooManyEmptyPostings(usize),
    TransactionIsNotBalanced,
    EmptyPostingShouldBeLast,
}