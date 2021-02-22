use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;

use crate::models::{Comment, Origin, Payee};
use crate::parser::chars::LineType;
use crate::parser::tokenizers::comment;
use crate::parser::{chars, Tokenizer};
use crate::ParserError;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<Payee, ParserError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(format!("{}{}{}",
        r"(payee) +"            , // payee directive
        r"(.*)"                 , // description
        r"(  ;.*)?"             , // note
        ).as_str()).unwrap();
    }
    let mystr = chars::get_line(tokenizer);
    let caps = RE.captures(mystr.as_str()).unwrap();

    let mut name = String::new();
    let mut detected: bool = false;
    let mut note: Option<String> = None;
    let mut comments: Vec<Comment> = vec![];
    let mut alias = HashSet::new();

    for (i, cap) in caps.iter().enumerate() {
        match cap {
            Some(m) => {
                match i {
                    1 =>
                    // payee
                    {
                        detected = true;
                    }
                    2 =>
                    // description
                    {
                        name = m.as_str().to_string()
                    }
                    3 =>
                    // note
                    {
                        note = Some(m.as_str().to_string())
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }

    if !detected {
        return Err(ParserError::UnexpectedInput(None));
    }
    while let LineType::Indented = chars::consume_whitespaces_and_lines(tokenizer) {
        match tokenizer.get_char().unwrap() {
            ';' => comments.push(comment::parse(tokenizer)),
            _ => match chars::get_string(tokenizer).as_str() {
                "alias" => {
                    alias.insert(chars::get_line(tokenizer).trim().to_string());
                }
                _ => {
                    eprintln!("Error while parsing posting.");
                    return Err(ParserError::UnexpectedInput(None));
                }
            },
        }
    }

    Ok(Payee {
        name,
        note,
        alias,
        origin: Origin::FromDirective,
    })
}

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
        let payee_raw = parse(& mut tokenizer);
        assert!(payee_raw.is_ok());
        let payee = payee_raw.unwrap();
        assert_eq!(payee.get_name(), "ACME");
    }
}
