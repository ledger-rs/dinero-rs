use dinero::parser::Tokenizer;
use dinero::CommonOpts;

use std::convert::TryFrom;
use std::path::PathBuf;
use structopt::StructOpt;

#[test]
fn test_include() {
    let p1 = PathBuf::from("tests/example_files/include.ledger".to_string());
    let mut tokenizer: Tokenizer = Tokenizer::try_from(&p1).unwrap();
    let _res = tokenizer.tokenize(&CommonOpts::from_iter(["", "-f", ""].iter()));
    // simply that it does not panic
    // todo change for something meaningful
    assert!(true);
}

#[test]
fn test_build_ledger_from_demo() {
    let p1 = PathBuf::from("tests/example_files/demo.ledger".to_string());
    let mut tokenizer: Tokenizer = Tokenizer::try_from(&p1).unwrap();
    let options = CommonOpts::from_iter(["", "-f", ""].iter());
    let items = tokenizer.tokenize(&options);
    let ledger = items.to_ledger(&options);
    assert!(ledger.is_ok());
}

#[test]
fn test_fail() {
    let mut tokenizer: Tokenizer = Tokenizer::from(
        "
2021-01-15 * Flights
    Expenses:Travel      200 EUR
    Assets:Checking account   -180 EUR
"
        .to_string(),
    );
    let parsed = tokenizer.tokenize(&CommonOpts::from_iter(["", "-f", ""].iter()));
    // It parses
    assert!(true);

    // But to a wrong ledger
    let ledger = parsed.to_ledger(&CommonOpts::from_iter(["", "-f", ""].iter()));
    assert!(ledger.is_err());
}

#[test]
fn comment_no_spaces() {
    let mut tokenizer: Tokenizer = Tokenizer::from(
        "
2000-01-01 * Sell shares
    Assets:Shares      -3.25 ACME @@ 326 USD;@ 100 USD
    Assets:Checking     326 USD
        "
        .to_string(),
    );
    let options = CommonOpts::from_iter(["", "-f", ""].iter());
    let items = tokenizer.tokenize(&options);
    let ledger = items.to_ledger(&options);
    assert!(ledger.is_ok());
}
#[test]
fn comment_spaces() {
    let mut tokenizer: Tokenizer = Tokenizer::from(
        "
2000-01-01 * Sell shares
    Assets:Shares      -3.25 ACME @@ 326 USD  ;@ 100 USD
    Assets:Checking     326 USD
        "
        .to_string(),
    );
    let options = CommonOpts::from_iter(["", "-f", ""].iter());
    let items = tokenizer.tokenize(&options);
    let ledger = items.to_ledger(&options);
    assert!(ledger.is_ok());
}
