use assert_cmd::Command;
use common::test_args;
mod common;
#[test]
/// Check the search by tag command
fn tags() {
    let args1 = &["reg", "-f", "tests/example_files/tags.ledger", "%healthy"];
    let assert_1 = Command::cargo_bin("dinero").unwrap().args(args1).assert();
    let output1 = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output1.lines().into_iter().count(), 1);
    test_args(args1);

    let args2 = &["reg", "-f", "tests/example_files/tags.ledger", "%shopping"];
    let assert_2 = Command::cargo_bin("dinero").unwrap().args(args2).assert();
    let output2 = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output2.lines().into_iter().count(), 2);

    test_args(args2);
}
