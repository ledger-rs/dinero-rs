use rexpect::spawn;

// const CARGO: &'static str = env!("CARGO");

#[test]
fn test_repl() {
    let command = "cargo run -- --init-file tests/example_files/empty_ledgerrc -f tests/example_files/demo.ledger";
    let mut p = spawn(command, None).unwrap();
    p.exp_regex(">> ").unwrap();
    p.send_line("anonymous").unwrap();
    p.send_line("exit").unwrap();
    p.exp_eof().unwrap();
    assert!(true);
}
