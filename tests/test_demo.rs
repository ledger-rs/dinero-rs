use dinero::models::Ledger;
use dinero::parser::Tokenizer;
use std::convert::TryFrom;
use std::path::PathBuf;

#[test]
fn test_include() {
    let p1 = PathBuf::from("tests/include1.ledger".to_string());
    let mut tokenizer: Tokenizer = Tokenizer::from(&p1);
    let res = tokenizer.tokenize();
    assert!(res.is_ok());
}

#[test]
fn test_build_ledger_from_demo() {
    let p1 = PathBuf::from("tests/demo.ledger".to_string());
    let mut tokenizer: Tokenizer = Tokenizer::from(&p1);
    let items = tokenizer.tokenize().unwrap();
    let ledger = Ledger::try_from(items);
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
    let parsed = tokenizer.tokenize().unwrap();
    println!("{:?}", Ledger::try_from(parsed));
}
