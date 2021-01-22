use crate::parser::chars::LineType;
use crate::parser::{chars, Tokenizer, Directive};
use crate::{Error, ErrorType};
use lazy_static::lazy_static;
use regex::Regex;

pub(super) fn parse(tokenizer: &mut Tokenizer) -> Result<Directive, Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(format!("{}{}",
        r"(tag) +"        , // directive commodity
        r"(.*)"                 , // description
        ).as_str()).unwrap();
    }
    let mystr = chars::get_line(tokenizer);
    let caps = RE.captures(mystr.as_str()).unwrap();

    let mut name = String::new();
    let mut detected: bool = false;
    let mut check = vec![];
    let mut assert = vec![];

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
            ';' => {  }, // Skip comments
            _ => match chars::get_string(tokenizer).as_str() {
                "check" => {check.push(chars::get_line(tokenizer).trim().to_string());},
                "assert" => {assert.push(chars::get_line(tokenizer).trim().to_string());},
                _=> {
                    eprintln!("Error while parsing posting.");
                    return Err(tokenizer.error(ErrorType::UnexpectedInput));
                }
            },
        }
    }

    Ok(Directive::Tag {
        name,
        check,
        assert,
    })
}
