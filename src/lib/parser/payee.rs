use crate::ledger::Comment;
use crate::parser::chars::LineType;
use crate::parser::{chars, comment, Directive, Tokenizer};
use crate::{Error, ErrorType};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

pub(super) fn parse(tokenizer: &mut Tokenizer) -> Result<Directive, Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(format!("{}{}{}",
        r"(payee) +"        , // directive commodity
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
                    // commodity
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
        return Err(tokenizer.error(ErrorType::UnexpectedInput));
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
                    return Err(tokenizer.error(ErrorType::UnexpectedInput));
                }
            },
        }
    }

    Ok(Directive::Payee { name, note, alias })
}
