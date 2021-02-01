use dinero::models::{Account, HasName};
use dinero::parser::Tokenizer;
use std::ops::Deref;

#[test]
fn test_account_names() {
    let tokenizers: Vec<Tokenizer> = vec![
        Tokenizer::from(
            "2021-01-15 * Flights
    Expenses:Travel            200 EUR
    Assets:Checking account   -200 EUR
2021-01-16 * More flights
    Expenses:Travel            300 EUR
    Assets:Checking account 
"
            .to_string(),
        ),
        Tokenizer::from(
            "2021-01-15 * Flights
    Expenses:Travel            200 EUR
    Assets:Checking account   -200 EUR
2021-01-16 * More flights
    Expenses:Travel            300 EUR
    Assets:Checking account  =-500 EUR
"
            .to_string(),
        ),
        Tokenizer::from(
            "2021-01-15 * Flights
    Expenses:Travel            200 EUR
    Assets:Checking account   -200 EUR
2021-01-16 * More flights
    Expenses:Travel            300 EUR
    Assets:Checking account   -300 EUR
"
            .to_string(),
        ),
        Tokenizer::from(
            "2021-01-15 * Flights
    Expenses:Travel            200 EUR
    Assets:Checking account   -200 EUR
2021-01-16 * More flights
    Expenses:Travel            300 EUR
    Assets:Checking account   -300 EUR = -500 EUR
"
            .to_string(),
        ),
    ];
    for (i, mut tokenizer) in tokenizers.into_iter().enumerate() {
        let parsed = tokenizer.tokenize().unwrap();
        let mut num_accounts = parsed.accounts.len();
        println!("Test case #{}", i);
        assert_eq!(num_accounts, 0, "There should be no accounts when parsed");
        num_accounts = parsed.to_ledger(true).unwrap().accounts.len();
        assert_eq!(num_accounts, 2, "There should be two accounts");
    }
}

#[test]
fn test_spaces_in_account_names() {
    let mut tokenizer = Tokenizer::from("account An account name with spaces   ".to_string());
    let parsed = tokenizer.tokenize().unwrap();
    let account = parsed
        .accounts
        .iter()
        .map(|x| x.1.deref())
        .collect::<Vec<&Account>>()[0];
    assert_eq!(account.get_name(), "An account name with spaces");
}
