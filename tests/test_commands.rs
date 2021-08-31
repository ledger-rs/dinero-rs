use assert_cmd::Command;
use common::{test_args, test_err};
mod common;
#[test]
fn date_filters() {
    let args1 = &[
        "bal",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/demo.ledger",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args1).assert();
    let mut output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 17);
    test_args(args1);
    let args2 = &[
        "bal",
        "-f",
        "tests/example_files/demo.ledger",
        "-e",
        "2021-01-17",
        "-b",
        "2021-01-15",
        "--force-color",
    ];
    let assert_2 = Command::cargo_bin("dinero").unwrap().args(args2).assert();
    output = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 12);

    test_args(args2);
}

/// A test for [issue 18](https://github.com/frosklis/dinero-rs/issues/18)
#[test]
fn exchange() {
    let mut outputs = Vec::new();
    for _ in 0..100 {
        let args = &[
            "bal",
            "-f",
            "tests/example_files/demo.ledger",
            "-X",
            "EUR",
            "--force-color",
        ];
        let assert = Command::cargo_bin("dinero").unwrap().args(args).assert();
        outputs.push(String::from_utf8(assert.get_output().to_owned().stdout).unwrap());
        test_args(args);
    }
    for i in 1..100 {
        assert_eq!(outputs[i], outputs[0], "output mismatch");
    }
}

/// A test for [issue 17](https://github.com/frosklis/dinero-rs/issues/17)
/// the aliases should not care about uppercase / lowercase
#[test]
fn commodity_alias() {
    let mut outputs = Vec::new();
    let aliases = vec!["EUR", "eur"];
    for alias in aliases {
        let args = &[
            "bal",
            "--init-file",
            "tests/example_files/empty_ledgerrc",
            "-f",
            "tests/example_files/demo.ledger",
            "-X",
            alias,
        ];
        let assert = Command::cargo_bin("dinero").unwrap().args(args).assert();
        outputs.push(String::from_utf8(assert.get_output().to_owned().stdout).unwrap());
        test_args(args);
    }
    assert_eq!(outputs[0], outputs[1], "output mismatch");
}

#[test]
/// Check that the register report is showing virtual postings
fn virtual_postings() {
    let args = &[
        "reg",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/virtual_postings.ledger",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    test_args(args);
    assert_eq!(output.lines().into_iter().count(), 7);
}

#[test]
/// Check that the virtual postings are being filtered out
fn real_filter() {
    let args = &[
        "reg",
        "-f",
        "tests/example_files/virtual_postings.ledger",
        "--real",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 4);

    test_args(args);
}

#[test]
/// Check that the tag filter works
fn tag_filter() {
    let args = &[
        "bal",
        "-f",
        "tests/example_files/demo.ledger",
        "--flat",
        "%fruit",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 4);

    test_args(args);
}

#[test]
/// Check that the tag filter works
fn depth_tree() {
    let args_1 = &[
        "bal",
        "-f",
        "tests/example_files/demo.ledger",
        "--depth",
        "1",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_1.lines().into_iter().count(), 11);
    test_args(args_1);

    let args_2 = &[
        "bal",
        "-f",
        "tests/example_files/demo.ledger",
        "--flat",
        "--depth",
        "1",
    ];
    let assert_2 = Command::cargo_bin("dinero").unwrap().args(args_2).assert();
    let output_2 = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_2.lines().into_iter().count(), 11);

    test_args(args_2);
}

#[test]
/// Check that the tag filter works
fn account_filter() {
    let args = &[
        "bal",
        "-f",
        "tests/example_files/demo.ledger",
        "--flat",
        "travel",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 1);

    test_args(args);
}

#[test]
/// Check the accounts command
fn accounts_command() {
    let args = &[
        "accounts",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/demo.ledger",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 7);

    test_args(args);
}

#[test]
/// Check the prices command
fn prices_command() {
    let args = &[
        "prices",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/demo.ledger",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 8);

    test_args(args);
}

#[test]
/// Check the payees command
fn payees_command() {
    let args = &[
        "payees",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/demo.ledger",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(
        output.lines().into_iter().count(),
        6,
        "Because of the aliases, there should be only 6 payees, not 7."
    );

    test_args(args);
}

#[test]
/// Check the commodities command
fn commodities_command() {
    let args = &[
        "commodities",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/demo.ledger",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();

    assert_eq!(output.lines().into_iter().count(), 8);
    test_args(args);
}

#[test]
/// If this fails it means that it created an extra posting
fn automated_fail() {
    let args = &[
        "reg",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/automated_fail.ledger",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output_err = String::from_utf8(assert_1.get_output().to_owned().stderr).unwrap();
    assert_eq!(output_err.lines().into_iter().count(), 5);

    test_err(args);
}

#[test]
fn automated_value_expression() {
    let args = &[
        "reg",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/automated.ledger",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 11);

    test_args(args);
}

#[test]
fn automated_add_tag() {
    let args = &[
        "reg",
        "-f",
        "tests/example_files/automated.ledger",
        "%yummy",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 2);

    test_args(args);
}

#[test]
fn payee_from_comments() {
    let args = &[
        "reg",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/demo.ledger",
        "@shop",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 1);

    test_args(args);
}

#[test]
/// Check that strict works
fn strict() {
    let args_1 = &[
        "bal",
        "-f",
        "tests/example_files/demo.ledger",
        "--strict",
        "--args-only",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stderr).unwrap();
    assert!(output_1.lines().into_iter().count() > 3);
    test_args(args_1);
}

#[test]
/// It should fail with no config file
fn no_config_file() {
    let args_1 = &["bal", "--init-file", "a file that does not exist"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stderr).unwrap();
    assert_eq!(output_1.lines().into_iter().count(), 1);
    test_err(args_1);
}

#[test]
#[should_panic()]
/// Check that pedantic works
fn pedantic() {
    let args_1 = &["bal", "-f", "tests/example_files/demo.ledger", "--pedantic"];

    test_args(args_1);
}

#[test]
/// Check the stats command
fn stats() {
    let args_1 = &["stats", "-f", "tests/example_files/demo.ledger"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert!(output_1.lines().into_iter().count() > 3);
    test_args(args_1);
}

#[test]
/// Check the collapse option
fn collapse() {
    let args = &[
        "reg",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/collapse_demo.ledger",
        "--collapse",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 2);

    test_args(args);
}

#[test]
/// Check the exchange option in the register report
fn roi() {
    let args = &[
        "roi",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/hledger_roi.ledger",
        "--cash-flows",
        "cash",
        "--assets-value",
        "snake",
        "-Q",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();

    for (i, line) in output.lines().into_iter().enumerate() {
        match i {
            3 => assert!(String::from(line).contains("2.50%")),
            5 => assert!(String::from(line).contains("2.38%")),
            _ => (),
        }
    }

    test_args(args);
}
#[test]
fn roi_calendar() {
    let args = &[
        "roi",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/hledger_roi.ledger",
        "--cash-flows",
        "cash",
        "--assets-value",
        "snake",
        "-Q",
        "--calendar",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();

    for (i, line) in output.lines().into_iter().enumerate() {
        if let 3 = i {
            assert!(String::from(line).contains("2.50%"));
            assert!(String::from(line).contains("2.38%"));
        }
    }

    test_args(args);
}

/// It should be equivalent to pass the args-only to passing an empty ledgerrc
#[test]
fn args_only() {
    let args_1 = &[
        "bal",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/hledger_roi.ledger",
    ];
    let args_2 = &[
        "bal",
        "--args-only",
        "-f",
        "tests/example_files/hledger_roi.ledger",
    ];

    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    let assert_2 = Command::cargo_bin("dinero").unwrap().args(args_2).assert();
    let output_2 = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();

    println!("{}", &output_1);
    println!("{}", &output_2);
    assert_eq!(output_1, output_2);

    test_args(args_1);
    test_args(args_2);
}

#[test]
/// Check the collapse option
fn related() {
    let args = &[
        "reg",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/collapse_demo.ledger",
        "--related",
        "travel",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();

    for (i, line) in output.lines().into_iter().enumerate() {
        match i {
            0 => assert!(String::from(line).contains("Checking account")),
            _ => unreachable!(),
        }
    }

    test_args(args);
}

#[test]
fn empty_file() {
    let args = &[
        "reg",
        "--init-file",
        "tests/example_files/empty_ledgerrc",
        "-f",
        "tests/example_files/empty_ledgerrc",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stderr).unwrap();
    for (i, line) in output.lines().enumerate() {
        match i {
            0 => assert_eq!(line, "The journal file does not have any information"),
            _ => unreachable!("The output should have only one line"),
        }
    }

    test_err(args);
}
