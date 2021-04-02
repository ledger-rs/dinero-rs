use assert_cmd::Command;
use common::{test_args, test_err};
mod common;

#[test]
/// A test for value expressions with dates and comparisons
fn compare_dates() {
    let args_1 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "date > to_date('2021/01/16')",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_1.lines().into_iter().count(), 2);
    test_args(args_1);

    let args_2 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "date >= to_date('2021/01/16')",
    ];
    let assert_2 = Command::cargo_bin("dinero").unwrap().args(args_2).assert();
    let output_2 = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_2.lines().into_iter().count(), 15);
    test_args(args_2);

    let args_3 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "date < to_date('2021/01/16')",
    ];
    let assert_3 = Command::cargo_bin("dinero").unwrap().args(args_3).assert();
    let output_3 = String::from_utf8(assert_3.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_3.lines().into_iter().count(), 7);
    test_args(args_3);

    let args_4 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "date <= to_date('2021/01/01')",
    ];
    let assert_4 = Command::cargo_bin("dinero").unwrap().args(args_4).assert();
    let output_4 = String::from_utf8(assert_4.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_4.lines().into_iter().count(), 2);
    test_args(args_4);

    let args_4 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "date == to_date('2021/01/01')",
    ];
    let assert_4 = Command::cargo_bin("dinero").unwrap().args(args_4).assert();
    let output_4 = String::from_utf8(assert_4.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_4.lines().into_iter().count(), 2);
    test_args(args_4);
}

#[test]
/// A test for value expressions with dates and comparisons
fn function_any() {
    let args_1 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "any(abs(amount) > 1000)",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_1.lines().into_iter().count(), 3);
    test_args(args_1);
    let args_2 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "any((500 + 500) < abs(amount))",
    ];
    let assert_2 = Command::cargo_bin("dinero").unwrap().args(args_2).assert();
    let output_2 = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_2.lines().into_iter().count(), 3);
    test_args(args_2);
}

#[test]
/// A test for value expressions with dates and comparisons
fn test_equality() {
    let args_1 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "any(abs(amount) == (2 * 1))",
    ];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_1.lines().into_iter().count(), 9);
    test_args(args_1);
    let args_2 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "any(abs(amount) == 100 $)",
    ];
    let assert_2 = Command::cargo_bin("dinero").unwrap().args(args_2).assert();
    let output_2 = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output_2.lines().into_iter().count(), 4);
    test_args(args_2);
}

#[test]
#[should_panic(expected = "Can't compare different currencies. € and USD.")]
/// Bad comparison
fn bad_comparison() {
    let args_1 = &[
        "reg",
        "-f",
        "tests/example_files/demo.ledger",
        "expr",
        "(2 * (5 eur)) < ((3 usd) / 5))",
    ];
    test_err(args_1);
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args_1).assert();
    let output_1 = String::from_utf8(assert_1.get_output().to_owned().stderr).unwrap();
    assert!(output_1.lines().into_iter().count() >= 1);
}
