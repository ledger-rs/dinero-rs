use rexpect::spawn;

const CARGO: &'static str = env!("CARGO");

#[test]
fn repl_open_and_close() {
    let options =
        " run -- --init-file tests/example_files/empty_ledgerrc -f tests/example_files/demo.ledger";
    let command = format!("{}{}", CARGO, options);
    let mut p = spawn(command.as_str(), Some(10_000)).unwrap();
    p.exp_regex(">> ").unwrap();
    p.send_line("exit").unwrap();
    p.exp_eof().unwrap();
    assert!(true);
}
