#[test]
fn test_balance() {
    let args: Vec<String> = vec![
        "executable",
        "bal",
        "-f",
        "tests/demo.ledger",
        "--init-file",
        "tests/example_ledgerrc",
        "--real",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();
    let res = dinero::run_app(args);
    assert!(res.is_ok());
}

#[test]
#[should_panic(expected = "Bad config file \"tests/example_bad_ledgerrc\"\nThis line should be a comment but isn\'t, it is bad on purpose.")]
fn test_bad_ledgerrc() {
    let args: Vec<String> = vec![
        "executable",
        "bal",
        "--init-file",
        "tests/example_bad_ledgerrc"
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();
    let res = dinero::run_app(args);
}
