use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;

use crate::models::{Comment, Origin, Payee};
use crate::parser::tokenizers::comment;
use crate::parser::Tokenizer;
use crate::ParserError;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<Payee, ParserError> {
    unimplemented!("payye");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HasName;
    #[test]
    fn parse_ko() {
        let input = "payee ACME  ; From the Looney Tunes\n\tWrong Acme, Inc.\n".to_string();
        let mut tokenizer = Tokenizer::from(input);
        let payee_raw = parse(&mut tokenizer);
        assert!(payee_raw.is_err());
    }

    #[test]
    fn parse_ok() {
        let input = "payee ACME\n\talias Acme, Inc.\n".to_string();
        let mut tokenizer = Tokenizer::from(input);
        let payee_raw = parse(&mut tokenizer);
        assert!(payee_raw.is_ok());
        let payee = payee_raw.unwrap();
        assert_eq!(payee.get_name(), "ACME");
    }
}
