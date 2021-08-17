use assert_cmd::Command;
use dinero::run_app;
use std::thread;
use std::time::Duration;
mod common;
// const CARGO: &'static str = env!("CARGO");

#[test]
fn stdin_repl() {
    let args1 = &[
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/demo.ledger",
    ];
    let command = Command::cargo_bin("dinero")
        .unwrap()
        .args(args1)
        .write_stdin("exit")
        .ok();
    assert!(command.is_ok());
}
