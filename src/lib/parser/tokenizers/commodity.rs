use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;

use crate::models::{Comment, Currency, Origin};
use crate::parser::chars::LineType;
use crate::parser::tokenizers::comment;
use crate::parser::{chars, Tokenizer};
use crate::ParserError;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<Currency, ParserError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(format!("{}{}{}",
        r"(commodity) +"        , // directive commodity
        r"(.*)"                 , // description
        r"(  ;.*)?"             , // note
        ).as_str()).unwrap();
    }
    let mystr = chars::get_line(tokenizer);
    let caps = RE.captures(mystr.as_str()).unwrap();

    let mut name = String::new();
    let mut detected: bool = false;
    let mut note: Option<String> = None;
    let mut format: Option<String> = None;
    let mut comments: Vec<Comment> = vec![];
    let mut default = false;
    let mut aliases = HashSet::new();

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
        return Err(ParserError::UnexpectedInput(Some(
            "Commodity expected. Not found.".to_string(),
        )));
    }
    while let LineType::Indented = chars::consume_whitespaces_and_lines(tokenizer) {
        match tokenizer.get_char().unwrap() {
            ';' => comments.push(comment::parse(tokenizer)),
            _ => match chars::get_string(tokenizer).as_str() {
                "note" => note = Some(chars::get_line(tokenizer).trim().to_string()),
                "alias" => {
                    aliases.insert(chars::get_line(tokenizer).trim().to_string());
                }
                "format" => format = Some(chars::get_line(tokenizer).trim().to_string()),
                "default" => default = true,
                other => {
                    eprintln!("Error while parsing posting.");
                    return Err(ParserError::UnexpectedInput(Some(format!(
                        "Found {}. Should be one of 'note', 'alias', 'format', 'default'",
                        other
                    ))));
                }
            },
        }
    }

    let currency = Currency {
        name,
        origin: Origin::FromDirective,
        note,
        aliases,
        format,
        default,
    };
    Ok(currency)
}
