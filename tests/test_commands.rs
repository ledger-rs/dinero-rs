use assert_cmd::Command;
use common::test_args;
mod common;
#[test]
fn date_filters() {
    let args1 = &["bal", "-f", "examples/demo.ledger"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args1).assert();
    let mut output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 18);
    test_args(args1);
    let args2 = &[
        "bal",
        "-f",
        "examples/demo.ledger",
        "-e",
        "2021-01-17",
        "-b",
        "2021-01-15",
        "--force-color",
    ];
    let assert_2 = Command::cargo_bin("dinero").unwrap().args(args2).assert();
    output = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 13);

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
            "examples/demo.ledger",
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
        let args = &["bal", "-f", "examples/demo.ledger", "-X", alias];
        let assert = Command::cargo_bin("dinero").unwrap().args(args).assert();
        outputs.push(String::from_utf8(assert.get_output().to_owned().stdout).unwrap());
        test_args(args);
    }
    assert_eq!(outputs[0], outputs[1], "output mismatch");
}

#[test]
/// Check that the register report is showing virtual postings
fn virtual_postings() {
    let args = &["reg", "-f", "examples/virtual_postings.ledger"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 7);
    test_args(args);
}

#[test]
/// Check that the virtual postings are being filtered out
fn real_filter() {
    let args = &["reg", "-f", "examples/virtual_postings.ledger", "--real"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 4);

    test_args(args);
}

#[test]
/// Check that the tag filter works
fn tag_filter() {
    let args = &["bal", "-f", "examples/demo.ledger", "--flat", "%fruit"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 4);

    test_args(args);
}


#[test]
/// Check that the tag filter works
fn account_filter() {
    let args = &["bal", "-f", "examples/demo.ledger", "--flat", "travel"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 1);

    test_args(args);
}


#[test]
/// Check the accounts command
fn accounts_command() {
    let args = &["accounts", "-f", "examples/demo.ledger"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args).assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 6);

    test_args(args);
}
