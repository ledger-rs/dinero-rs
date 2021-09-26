use assert_cmd::Command;
use dinero::{parser::Tokenizer, CommonOpts};
use structopt::StructOpt;
mod common;
use common::test_args;

#[test]
fn bitcoin_balances() {
    let first_args = &[
        "bal",
        "crypto",
        "--args-only",
        "-f",
        "tests/example_files/prices.ledger",
        "--exchange",
        "USD",
    ];

    let expectations = &[
        ("2021-09", "48864"),
        ("2021-08-15", "45244"),
        ("2021-08-12", "46365"),
    ];

    for (date, amount) in expectations {
        let mut args: Vec<&str> = first_args.to_vec();
        args.append(&mut vec!["--end", date]);

        let assert = Command::cargo_bin("dinero").unwrap().args(&args).assert();
        let output = String::from_utf8(assert.get_output().to_owned().stdout).unwrap();
        for (i, line) in output.lines().into_iter().enumerate() {
            match i {
                0 => assert!(String::from(line).contains(amount), "Should be {}", amount),
                _ => unreachable!(),
            }
        }
        test_args(&args);
    }
}
#[test]
fn bitcoin_balances_convert() {
    let first_args = &[
        "bal",
        "crypto",
        "--args-only",
        "-f",
        "tests/example_files/prices.ledger",
        "--convert",
        "USD",
    ];

    let expectations = &[
        ("2021-09", "48864"),
        ("2021-08-15", "45244"),
        ("2021-08-12", "46365"),
    ];

    for (date, amount) in expectations {
        let mut args: Vec<&str> = first_args.to_vec();
        args.append(&mut vec!["--end", date]);

        let assert = Command::cargo_bin("dinero").unwrap().args(&args).assert();
        let output = String::from_utf8(assert.get_output().to_owned().stdout).unwrap();
        for (i, line) in output.lines().into_iter().enumerate() {
            match i {
                0 => {
                    assert!(String::from(line).contains(amount), "Should be {}", amount);
                    assert!(
                        String::from(line).contains("BTC"),
                        "Should contain 1 BTC"
                    );
                }
                _ => unreachable!(),
            }
        }
        test_args(&args);
    }
}

#[test]
/// Check the exchange option in the register report
fn reg_exchange() {
    let args = &[
        "reg",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/reg_exchange.ledger",
        "--exchange",
        "EUR",
        "travel",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();

    for (i, line) in output.lines().into_iter().enumerate() {
        match i {
            0 => assert!(String::from(line).contains("100")),
            1 => assert!(String::from(line).contains("133")),
            _ => unreachable!(),
        }
    }

    test_args(args);
}
