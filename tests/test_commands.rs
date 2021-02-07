use assert_cmd::Command;
use dinero::run_app;
#[test]
fn date_filters() {
    let assert_1 = Command::cargo_bin("dinero")
        .unwrap()
        .args(&["bal", "-f", "examples/demo.ledger"])
        .assert();
    let mut output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 18);
    let assert_2 = Command::cargo_bin("dinero")
        .unwrap()
        .args(&[
            "bal",
            "-f",
            "examples/demo.ledger",
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

/// A test for [issue 18](https://github.com/frosklis/dinero-rs/issues/18)
#[test]
fn exchange() {
    let mut outputs = Vec::new();
    for _ in 0..100 {
        let assert = Command::cargo_bin("dinero")
            .unwrap()
            .args(&["bal", "-f", "examples/demo.ledger", "-X", "EUR"])
            .assert();
        outputs.push(String::from_utf8(assert.get_output().to_owned().stdout).unwrap());
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
        let assert = Command::cargo_bin("dinero")
            .unwrap()
            .args(&["bal", "-f", "examples/demo.ledger", "-X", alias])
            .assert();
        outputs.push(String::from_utf8(assert.get_output().to_owned().stdout).unwrap());
    }
    assert_eq!(outputs[0], outputs[1], "output mismatch");
}

#[test]
/// Check that the register report is showing virtual postings
fn virtual_postings() {
    let assert_1 = Command::cargo_bin("dinero")
        .unwrap()
        .args(&["reg", "-f", "examples/virtual_postings.ledger"])
        .assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 7);
}

#[test]
/// Check that the virtual postings are being filtered out
fn real_filter() {
    let assert_1 = Command::cargo_bin("dinero")
        .unwrap()
        .args(&["reg", "-f", "examples/virtual_postings.ledger", "--real"])
        .assert();
    let output = String::from_utf8(assert_1.get_output().to_owned().stdout).unwrap();
    assert_eq!(output.lines().into_iter().count(), 4);

    let args: Vec<String> = vec![
        "testing",
        "reg",
        "-f",
        "examples/virtual_postings.ledger",
        "--real",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();
    let res = run_app(args);
    assert!(res.is_ok());
}
