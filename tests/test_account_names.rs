use dinero::{CommonOpts, parser::Tokenizer};

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
        println!("Test case #{}", i);
        let parsed = tokenizer.tokenize();
        let mut num_accounts = parsed.accounts.len();
        assert_eq!(num_accounts, 0, "There should be no accounts when parsed");
        let mut options = CommonOpts::new();
        options.no_balance_check = true;
        num_accounts = parsed.to_ledger(&options).unwrap().accounts.len();
        assert_eq!(num_accounts, 2, "There should be two accounts");
    }
}
