use dinero::{parser::Tokenizer, CommonOpts};
use structopt::StructOpt;
#[test]
fn test_balances() {
    let mut tokenizer = Tokenizer::from(
        "2021-01-15 * Flights
    Expenses:Travel            200 EUR
    Assets:Checking account   -200 EUR
2021-01-16 * More flights 1
    Expenses:Travel            300 EUR
    Assets:Checking account           = -500 EUR
2021-01-16 * More flights 2
    Expenses:Travel            300 EUR
    Assets:Checking account   -300 EUR = -800 EUR
2021-01-16 * More flights 3
    Expenses:Travel            300 EUR
    Assets:Checking account           = -1100 EUR
"
        .to_string(),
    );
    let options = CommonOpts::from_iter(["", "-f", ""].iter());
    let parsed = tokenizer.tokenize(&options);
    let ledger = parsed.to_ledger(&options);
    assert!(ledger.is_ok(), "This should balance");
}
