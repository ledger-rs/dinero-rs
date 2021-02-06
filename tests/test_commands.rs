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
            "--force-color",
        ])
        .assert();
    output = String::from_utf8(assert_2.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 13);
}

/// A test for issue 18
#[test]
fn exchange() {
    let mut outputs = Vec::new();
    for _ in 0..100 {
        let assert = Command::cargo_bin("dinero")
            .unwrap()
            .args(&["bal", "-f", "tests/demo.ledger", "-X", "EUR"])
            .assert();
        outputs.push(String::from_utf8(assert.get_output().to_owned().stdout).unwrap());
    }
    for i in 1..100 {
        assert_eq!(outputs[i], outputs[0], "output mismatch");
    }
}
