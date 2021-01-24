use crate::ledger::{Cleared, Comment, CostType, Transaction};
use crate::parser::chars::LineType;
use crate::parser::{chars, comment, Tokenizer};
use crate::{Error, ErrorType};
use chrono::NaiveDate;
use lazy_static::lazy_static;
use num::rational::Rational64;
use regex::Regex;
use std::str::FromStr;

/// Parses a transaction
pub(super) fn parse<'a>(tokenizer: &'a mut Tokenizer) -> Result<Transaction<Posting>, Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(format!("{}{}{}{}{}{}",
        r"(\d{4}[\-]\d{2}[\-]\d{2})"        , // date
        r"(= ?\d{4}[\-]\d{2}[\-]\d{2})? +"  , // effective_date
        r"([\*!])? +"                        , // cleared
        r"(\(.*\) )?"                       , // code
        r"(.*)"                             , // description
        r"(  ;.*)?"                         , // note
        ).as_str()).unwrap();
    }
    let mystr = chars::get_line(tokenizer);
    let caps = RE.captures(mystr.as_str()).unwrap();

    let mut transaction = Transaction::<Posting>::new();

    for (i, cap) in caps.iter().enumerate() {
        match cap {
            Some(m) => {
                match i {
                    1 =>
                    // date
                    {
                        transaction.date = Some(parse_date(m.as_str()))
                    }
                    2 =>
                    // effective date
                    {
                        transaction.effective_date = Some(parse_date(m.as_str()))
                    }
                    3 =>
                    // cleared
                    {
                        transaction.cleared = match m.as_str() {
                            "*" => Cleared::Cleared,
                            "!" => Cleared::NotCleared,
                            _ => return Err(tokenizer.error(ErrorType::ParserError)),
                        }
                    }
                    4 =>
                    // code
                    {
                        transaction.code = Some(m.as_str().to_string())
                    }
                    5 =>
                    // description
                    {
                        transaction.description = m.as_str().to_string()
                    }
                    6 =>
                    // note
                    {
                        transaction.code = Some(m.as_str().to_string())
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }

    while let LineType::Indented = chars::consume_whitespaces_and_lines(tokenizer) {
        match tokenizer.get_char().unwrap() {
            ';' => transaction.comments.push(comment::parse(tokenizer)),
            c if c.is_numeric() => return Err(tokenizer.error(ErrorType::UnexpectedInput)),
            _ => match parse_posting(tokenizer) {
                Ok(posting) => transaction.postings.push(posting),
                Err(e) => {
                    eprintln!("Error while parsing posting.");
                    return Err(e);
                }
            },
        }
    }

    Ok(transaction)
}

#[derive(Debug, Clone)]
pub struct Posting {
    pub account: String,
    pub money_amount: Option<Rational64>,
    pub money_currency: Option<String>,
    pub cost_amount: Option<Rational64>,
    pub cost_currency: Option<String>,
    pub cost_type: Option<CostType>,
    pub balance_amount: Option<Rational64>,
    pub balance_currency: Option<String>,
    pub comments: Vec<Comment>,
}
/// Parses a posting
///
fn parse_posting(tokenizer: &mut Tokenizer) -> Result<Posting, Error> {
    // let posting_line = chars::get_line(tokenizer);
    //let vec_chars: Vec<char> = posting_line.chars().collect();
    let mut account = String::new();
    loop {
        let c = tokenizer.next();
        if !c.is_whitespace() {
            account.push(c);
        } else if (c == '\t') | (c == '\n') | (c == '\r') {
            break;
        } else {
            let d = tokenizer.next();
            if d.is_whitespace() | (c == '\n') | (c == '\r') {
                break;
            } else {
                account.push(c);
                account.push(d);
            }
        }
    }
    let mut posting = Posting {
        account,
        money_amount: None,
        money_currency: None,
        cost_amount: None,
        cost_currency: None,
        cost_type: None,
        balance_amount: None,
        balance_currency: None,
        comments: Vec::new(),
    };
    chars::consume_whitespaces(tokenizer);
    // Amounts
    loop {
        match tokenizer.get_char() {
            Some('\n') => break,
            None => break,
            Some(';') => {
                posting.comments.push(comment::parse(tokenizer));
                return Ok(posting);
            }
            Some('=') => {
                tokenizer.position += 1;
                tokenizer.line_position += 1;
                let money = parse_money(tokenizer)?;
                posting.balance_amount = Some(money.0);
                posting.balance_currency = Some(money.1);
            }
            Some('@') => {
                if posting.money_amount.is_none() {
                    return Err(tokenizer.error(ErrorType::ParserError));
                }

                tokenizer.position += 1;
                tokenizer.line_position += 1;
                if tokenizer.get_char().unwrap() == '@' {
                    tokenizer.position += 1;
                    tokenizer.line_position += 1;
                    posting.cost_type = Some(CostType::Total);
                } else {
                    posting.cost_type = Some(CostType::PerUnit);
                }
                let money = parse_money(tokenizer)?;
                posting.cost_amount = Some(money.0);
                posting.cost_currency = Some(money.1);
            }
            _ => {
                let money = parse_money(tokenizer)?;
                posting.money_amount = Some(money.0);
                posting.money_currency = Some(money.1);
            }
        }
        chars::consume_whitespaces(tokenizer);
    }
    Ok(posting)
}

fn parse_money(tokenizer: &mut Tokenizer) -> Result<(Rational64, String), Error> {
    let currency: String;
    let amount: Rational64;

    match tokenizer.get_char() {
        Some(c) if c.is_numeric() | (c == '.') | (c == '-') => {
            amount = parse_amount(tokenizer)?;
            currency = chars::get_string(tokenizer);
        }
        Some(_) => {
            currency = chars::get_string(tokenizer);
            amount = parse_amount(tokenizer)?;
        }
        None => return Err(tokenizer.error(ErrorType::ParserError)),
    }
    Ok((amount, currency))
}

fn parse_amount(tokenizer: &mut Tokenizer) -> Result<Rational64, Error> {
    let mut num = String::new();
    let mut den = "1".to_string();
    let mut decimal = false;
    loop {
        match tokenizer.get_char() {
            Some('.') => {
                if decimal {
                    return Err(tokenizer.error(ErrorType::ParserError));
                } else {
                    decimal = true
                }
            }
            Some(c) if (c == '-') | c.is_numeric() => {
                num.push(c);
                if decimal {
                    den.push('0')
                };
            }
            _ => break,
        }
        tokenizer.position += 1;
        tokenizer.line_position += 1;
    }
    Ok(Rational64::new(
        match i64::from_str(num.as_str()) {
            Ok(n) => n,
            Err(_) => return Err(tokenizer.error(ErrorType::ParserError)),
        },
        match i64::from_str(den.as_str()) {
            Ok(d) => d,
            Err(_) => return Err(tokenizer.error(ErrorType::ParserError)),
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
