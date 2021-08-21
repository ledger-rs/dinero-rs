//! This module contains auxiliary parsers

use super::{GrammarParser, Rule};
use chrono::NaiveDate;
use num::{BigInt, BigRational, Signed};
use pest::iterators::Pair;

use pest::Parser;
use std::str::FromStr;
use std::usize;

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

pub(crate) fn rational2float(number: &BigRational, decimals: usize) -> f64 {
    let base: i32 = 10;
    let mut integer_part = number.trunc();
    let decimal_part = (number.fract() * BigInt::from(base.pow(decimals as u32 + 2)))
        .abs()
        .trunc();

    let mut decimal_str = if decimals == 0 {
        String::new()
    } else {
        format!("{:0width$}", decimal_part.numer(), width = decimals + 2)
    };
    if decimals > 0 {
        decimal_str.truncate(decimals + 1);

        let mut decimal = if u64::from_str(&decimal_str).unwrap() % 10 >= 5 {
            u64::from_str(&decimal_str).unwrap() / 10 + 1
        } else {
            u64::from_str(&decimal_str).unwrap() / 10
        };
        let len = format!("{}", decimal).len();
        if len == decimals + 1 {
            decimal = 0;
            if integer_part.is_positive() {
                integer_part += BigInt::from(1);
            } else {
                integer_part -= BigInt::from(1);
            }
        }
        let decimal_separator = ".";
        decimal_str = format!("{}{:0width$}", decimal_separator, decimal, width = decimals);
    }
    let repr = format!("{}{}", integer_part, decimal_str);
    f64::from_str(repr.as_str()).unwrap()
}
/// Counts the number of decimals in an amount as defined in the grammar
pub(crate) fn count_decimals(amount: &str) -> usize {
    let mut parsed = GrammarParser::parse(Rule::money, amount)
        .unwrap()
        .next()
        .unwrap()
        .into_inner();
    let number = parsed.next().unwrap();

    let number = match number.as_rule() {
        Rule::number => number,
        //Rule::currency is the only other option
        _ => parsed.next().unwrap(),
    };

    assert_eq!(number.as_rule(), Rule::number);

    let text = number.as_str();
    // dbg!(text);
    if text.contains(".") {
        number.as_str().split(".").last().unwrap().len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::utils::count_decimals;

    #[test]
    fn count_decimals_test() {
        assert_eq!(count_decimals("150.4 EUR"), 1);
        assert_eq!(count_decimals("150 EUR"), 0);
        assert_eq!(count_decimals("EUR 150.4"), 1);
    }
}
