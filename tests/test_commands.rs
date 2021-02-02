use assert_cmd::Command;

#[test]
fn date_filters() {
    let assert_1 = Command::cargo_bin("dinero")
        .unwrap()
        .args(&["bal", "-f", "tests/demo.ledger"])
        .assert();
    let mut output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 18);
    let assert_2 = Command::cargo_bin("dinero")
        .unwrap()
        .args(&[
            "bal",
            "-f",
            "tests/demo.ledger",
            "-e",
            "2021-01-17",
            "-b",
            "2021-01-15",
        ])
        .assert();
    output = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 13);
}
