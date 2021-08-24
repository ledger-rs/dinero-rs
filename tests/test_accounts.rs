use dinero::{parser::Tokenizer, CommonOpts};
use structopt::StructOpt;

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
        let parsed = tokenizer.tokenize(&CommonOpts::from_iter(["", "-f", ""].iter()));
        let mut num_accounts = parsed.accounts.len();
        assert_eq!(num_accounts, 0, "There should be no accounts when parsed");
        let mut options = CommonOpts::from_iter(["", "-f", ""].iter());
        options.no_balance_check = true;
        num_accounts = parsed.to_ledger(&options).unwrap().accounts.len();
        assert_eq!(num_accounts, 2, "There should be two accounts");
    }
}

#[test]
fn test_account_directive() {
    let mut tokenizer = Tokenizer::from(
        "account Assets:Revolut
    country GB
    alias revolut
    payee Revolut "
            .to_string(),
    );

    let parsed = tokenizer.tokenize(&CommonOpts::from_iter(["", "-f", ""].iter()));
    let num_accounts = parsed.accounts.len();
    assert_eq!(num_accounts, 1, "Parse one account")
}
