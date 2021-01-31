use dinero::parser::Tokenizer;

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
        let ledger = parsed.to_ledger(false).unwrap();
        let num_accounts = ledger.accounts.len();
        println!("Test case #{}", i);
        assert_eq!(num_accounts, 2, "There should be two accounts");
    }
}
