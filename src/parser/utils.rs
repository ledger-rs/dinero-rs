//! This module contains auxiliary parsers

use super::{GrammarParser, Rule};
use chrono::NaiveDate;
use num::{BigInt, BigRational};
use pest::iterators::Pair;

use pest::Parser;
use std::str::FromStr;

pub(crate) fn parse_str_as_date(date: &str) -> NaiveDate {
    parse_date(
        GrammarParser::parse(Rule::date, date)
            .unwrap()
            .next()
            .unwrap(),
    )
}

/// Parses a date
pub(crate) fn parse_date(date: Pair<Rule>) -> NaiveDate {
    // Assume date is a Rule::date
    let mut parsed = date.into_inner();
    let year = i32::from_str(parsed.next().unwrap().as_str()).unwrap();
    let sep1 = parsed.next().unwrap().as_str();
    let month = u32::from_str(parsed.next().unwrap().as_str()).unwrap();
    let sep2 = parsed.next().unwrap().as_str();
    let day = u32::from_str(parsed.next().unwrap().as_str()).unwrap();

    assert_eq!(sep1, sep2, "wrong date separator");
    NaiveDate::from_ymd(year, month, day)
}

pub(crate) fn parse_rational(number: Pair<Rule>) -> BigRational {
    let mut num = String::new();
    let mut den = "1".to_string();
    let mut decimal = false;
    for c in number.as_str().chars() {
        if c == '.' {
            decimal = true
        } else {
            num.push(c);
            if decimal {
                den.push('0')
            };
        }
    }
    BigRational::new(
        BigInt::from_str(num.as_str()).unwrap(),
        BigInt::from_str(den.as_str()).unwrap(),
    )
}

pub(crate) fn parse_string(string: Pair<Rule>) -> String {
    match string.as_rule() {
        Rule::string => {
            let quoted = string.as_str();
            let len = quoted.len();
            quoted[1..len - 1].to_string()
        }
        Rule::unquoted => string.as_str().to_string(),
        Rule::commodity_spec => {
            let as_str = string.as_str();
            match string.into_inner().next() {
                Some(x) => parse_string(x),
                None => as_str.trim().to_string(),
            }
        }
        Rule::currency | Rule::commodity_in_directive => {
            parse_string(string.into_inner().next().unwrap())
        }
        _ => string.as_str().trim().to_string(),
    }
}
