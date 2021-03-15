use dinero::{CommonOpts, parser::Tokenizer};
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
    let parsed = tokenizer.tokenize();
    let ledger = parsed.to_ledger(&CommonOpts::new());
    assert!(ledger.is_ok(), "This should balance");
}
