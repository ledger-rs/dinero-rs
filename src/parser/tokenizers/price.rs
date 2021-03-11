use super::super::Rule;
use crate::models::ParsedPrice;
use crate::parser::utils::{parse_date, parse_rational, parse_string};
use crate::parser::Tokenizer;

use pest::iterators::Pair;

impl<'a> Tokenizer<'a> {
    pub(crate) fn parse_price(&self, element: Pair<Rule>) -> ParsedPrice {
        let mut parsed = element.into_inner();
        let date = parse_date(parsed.next().unwrap());
        let commodity = {
            let time_or_commodity = parsed.next().unwrap();
            match time_or_commodity.as_rule() {
                Rule::time => parse_string(parsed.next().unwrap()),
                _ => parse_string(time_or_commodity),
            }
        };
        let amount = parse_rational(parsed.next().unwrap());
        let other_commodity = parse_string(parsed.next().unwrap());

        ParsedPrice {
            date,
            commodity,
            other_commodity,
            other_quantity: amount,
        }
    }
}
