use super::super::{GrammarParser, Rule};
use crate::models::ParsedPrice;
use crate::parser::utils::{parse_date, parse_rational, parse_string};
use crate::parser::{chars, Tokenizer};
use crate::ParserError;
use pest::Parser;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<ParsedPrice, ParserError> {
    let mystr = chars::get_line(tokenizer);
    let mut parsed = GrammarParser::parse(Rule::price, mystr.as_str())
        .expect("Could not parse price!") // unwrap the parse result
        .next()
        .unwrap()
        .into_inner();
    let date = parse_date(parsed.next().unwrap());
    let commodity = parse_string(parsed.next().unwrap());
    let amount = parse_rational(parsed.next().unwrap());
    let other_commodity = parse_string(parsed.next().unwrap());

    Ok(ParsedPrice {
        date,
        commodity,
        other_commodity,
        other_quantity: amount,
    })
}
