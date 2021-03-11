use dinero::parser::Tokenizer;

use assert_cmd::Command;
use std::path::PathBuf;

#[test]
#[should_panic(expected = "Should be money.")]
/// The expression in an automated account should evaluate to money
fn not_money() {
    let mut tokenizer: Tokenizer = Tokenizer::from(
        "
= travel
    (failure)     (account)
2021-01-15 * Flights
    Expenses:Travel      200 EUR
    Assets:Checking account   -200 EUR
"
        .to_string(),
    );
    let parsed = tokenizer.tokenize();

    // It parses -- it has not panicked
    assert!(true);

    // But to a wrong ledger -- panics!
    let ledger = parsed.to_ledger(false);
}
