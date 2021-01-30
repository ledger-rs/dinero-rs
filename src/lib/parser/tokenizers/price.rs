use std::str::FromStr;

use chrono::NaiveDate;
use lazy_static::lazy_static;
use num::rational::Rational64;
use regex::Regex;

use crate::models::ParsedPrice;
use crate::parser::{chars, Tokenizer};
use crate::ParserError;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<ParsedPrice, ParserError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(format!("{}{}{}{}{}",
        r"P +",
        r"(\d{4}[\-/]\d{2}[\-/]\d{2}) +"        , // date
        r"(.*) +"                        , // commodity
        r"([\d\.,]+) +"                       , // quantity
        r"(.*)"                             , // other_commodity
        ).as_str()).unwrap();
    }
    let mystr = chars::get_line(tokenizer);
    let caps = match RE.captures(mystr.as_str()) {
        Some(m) => m,
        None => {
            return Err(ParserError::UnexpectedInput(Some(
                "Expected price.".to_string(),
            )));
        }
    };

    let mut date: NaiveDate = NaiveDate::from_num_days_from_ce(0);
    let mut commodity: String = String::new();
    let mut other_commodity: String = String::new();
    let mut other_quantity: Rational64 = Rational64::from_integer(1);

    for (i, cap) in caps.iter().enumerate() {
        match cap {
            Some(m) => {
                match i {
                    1 =>
                    // date
                    {
                        date = parse_date(m.as_str());
                    }
                    2 =>
                    // code
                    {
                        commodity = m.as_str().to_string();
                    }
                    3 =>
                    // code
                    {
                        other_quantity = parse_amount(m.as_str())?;
                    }
                    4 =>
                    // code
                    {
                        other_commodity = m.as_str().to_string();
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }

    Ok(ParsedPrice {
        date,
        commodity,
        other_commodity,
        other_quantity,
    })
}

fn parse_amount(input: &str) -> Result<Rational64, ParserError> {
    let mut num = String::new();
    let mut den = "1".to_string();
    let mut decimal = false;
    for c in input.chars() {
        match c {
            '.' => {
                if decimal {
                    return Err(ParserError::UnexpectedInput(Some(
                        "Too many decimal separators".to_string(),
                    )));
                } else {
                    decimal = true
                }
            }
            c if (c == '-') | c.is_numeric() => {
                num.push(c);
                if decimal {
                    den.push('0')
                };
            }
            _ => break,
        }
    }
    Ok(Rational64::new(
        match i64::from_str(num.as_str()) {
            Ok(n) => n,
            Err(_) => {
                return Err(ParserError::UnexpectedInput(Some(
                    "Wrong number format".to_string(),
                )))
            }
        },
        match i64::from_str(den.as_str()) {
            Ok(d) => d,
            Err(_) => return Err(ParserError::UnexpectedInput(None)),
        },
    ))
}

fn parse_date(date_str: &str) -> NaiveDate {
    // yyyy-mm-dd is 10 characters
    assert!(date_str.len() == 10);
    assert_eq!(
        date_str.chars().nth(4),
        date_str.chars().nth(7),
        "Separators mismatch"
    );
    let sep = date_str.chars().nth(4).unwrap();
    let mut parts = date_str.split(sep);
    let year = i32::from_str(parts.next().unwrap()).unwrap();
    let month = u32::from_str(parts.next().unwrap()).unwrap();
    let day = u32::from_str(parts.next().unwrap()).unwrap();

    NaiveDate::from_ymd(year, month, day)
}
