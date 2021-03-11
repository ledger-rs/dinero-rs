use dinero::parser::Tokenizer;

use assert_cmd::Command;
use std::path::PathBuf;

#[test]
fn test_include() {
    let p1 = PathBuf::from("tests/example_files/include.ledger".to_string());
    let mut tokenizer: Tokenizer = Tokenizer::from(&p1);
    let res = tokenizer.tokenize();
    // simply that it does not panick
    // todo change for something meaningful
    assert!(true);
}

#[test]
fn test_build_ledger_from_demo() {
    let p1 = PathBuf::from("tests/example_files/demo.ledger".to_string());
    let mut tokenizer: Tokenizer = Tokenizer::from(&p1);
    let items = tokenizer.tokenize();
    let ledger = items.to_ledger(false);
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
    let parsed = tokenizer.tokenize();
    // It parses
    assert!(true);

    // But to a wrong ledger
    let ledger = parsed.to_ledger(false);
    assert!(ledger.is_err());
}

#[test]
/// Test whether glob expressions in include are working
fn include_glob() {
    let assert_1 = Command::cargo_bin("dinero")
        .unwrap()
        .args(&["prices", "-f", "tests/example_files/include.ledger"])
        .assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert!(output.lines().into_iter().count() > 100);
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
    let items = tokenizer.tokenize();
    let ledger = items.to_ledger(false);
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
    let items = tokenizer.tokenize();
    let ledger = items.to_ledger(false);
    assert!(ledger.is_ok());
}
