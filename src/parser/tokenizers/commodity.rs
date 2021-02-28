use super::super::{GrammarParser, Rule};
use std::collections::HashSet;

use crate::models::{Comment, Currency, Origin};
use crate::parser::chars::LineType;
use crate::parser::tokenizers::comment;
use crate::parser::utils::parse_string;
use crate::parser::{chars, Tokenizer};
use crate::ParserError;
use pest::Parser;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<Currency, ParserError> {
    let mystr = chars::get_line(tokenizer);
    let mut parsed = GrammarParser::parse(Rule::commodity, mystr.as_str())
        .expect("Could not parse commodity!") // unwrap the parse result
        .next()
        .unwrap()
        .into_inner();

    let name = parse_string(parsed.next().unwrap());
    let mut note: Option<String> = None;
    let mut format: Option<String> = None;
    let mut comments: Vec<Comment> = vec![];
    let mut default = false;
    let mut aliases = HashSet::new();

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
        name: name.trim().to_string(),
        origin: Origin::FromDirective,
        note,
        aliases,
        format,
        default,
        precision: None,
    };
    Ok(currency)
}
