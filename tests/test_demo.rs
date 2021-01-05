use std::path::{PathBuf};
use dinero::parser::Tokenizer;

#[test]
fn test_include() {
    let p1 = PathBuf::from("tests/include1.ledger".to_string());
    let mut tokenizer: Tokenizer = Tokenizer::from(&p1);
    let res = tokenizer.parse();
    assert!(res.is_ok());
}