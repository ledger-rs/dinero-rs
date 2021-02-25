use crate::models::{Cleared, Comment, PostingType, PriceType, Transaction, TransactionType};
use crate::parser::chars::LineType;
use crate::parser::tokenizers::comment;
use crate::parser::{chars, Tokenizer};
use crate::{Error, ParserError};
use chrono::NaiveDate;
use lazy_static::lazy_static;
use num::rational::BigRational;
use num::BigInt;
use regex::Regex;
use std::str::FromStr;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<Transaction<RawPosting>, Error> {
    parse_generic(tokenizer, true)
}

pub(crate) fn parse_automated_transaction(
    tokenizer: &mut Tokenizer,
) -> Result<Transaction<RawPosting>, Error> {
    parse_generic(tokenizer, false)
}

/// Parses a transaction
fn parse_generic(tokenizer: &mut Tokenizer, real: bool) -> Result<Transaction<RawPosting>, Error> {
    lazy_static! {
        static ref RE_REAL: Regex = Regex::new(format!("{}{}{}{}{}{}{}",
        r"(\d{4}[/-]\d{2}[/-]\d{2})"        , // date
        r"(= ?\d{4}[/-]\d{2}[/-]\d{2})? +"  , // effective_date
        r"([\*!])? +"                       , // cleared
        r"(\(.*\) )?"                       , // code
        r"(.*)"                             , // description
        r"( |.*)"                           , // payee
        r"( ;.*)?"                          , // note
        ).as_str()).unwrap();
        static ref RE_AUTOMATED: Regex = Regex::new(format!("{}",r"=(.*)" ).as_str()).unwrap();
    }
    let mystr = chars::get_line(tokenizer);
    let caps = match real {
        true => match RE_REAL.captures(mystr.as_str()) {
            Some(c) => c,
            None => return Err(tokenizer.error(ParserError::UnexpectedInput(None))),
        },
        false => RE_AUTOMATED.captures(mystr.as_str()).unwrap(),
    };

    let mut transaction = Transaction::<RawPosting>::new(match real {
        true => TransactionType::Real,
        false => TransactionType::Automated,
    });

    for (i, cap) in caps.iter().enumerate() {
        match cap {
            Some(m) => {
                match i {
                    1 =>
                    // date
                    {
                        match real {
                            true => transaction.date = Some(parse_date(m.as_str())),
                            false => {
                                transaction.description = m.as_str().to_string();
                            }
                        }
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
                            _ => Cleared::Unknown,
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
                        transaction.description = m.as_str().to_string();
                    }
                    6 =>
                    // payee
                    {
                        if real {
                            match m.as_str() {
                                "" => (),
                                x => transaction.payee = Some(x.to_string()),
                            }
                        }
                    }
                    7 =>
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
    if real & transaction.payee.is_none() {
        transaction.payee = Some(transaction.description.clone());
    }
    // Have a flag so that it can be known whether a comment belongs to the transaction or to the
    // posting
    let mut parsed_posting = false;
    while let LineType::Indented = chars::consume_whitespaces_and_lines(tokenizer) {
        match tokenizer.get_char().unwrap() {
            ';' => {
                let comment = comment::parse(tokenizer);
                match parsed_posting {
                    true => {
                        let len = transaction.postings.len();

                        transaction.postings[len - 1].comments.push(comment);
                    }
                    false => {
                        transaction.comments.push(comment);
                    }
                }
            }
            c if c.is_numeric() => {
                return Err(tokenizer.error(ParserError::UnexpectedInput(Some(
                    "Expecting account name".to_string(),
                ))));
            }
            _ => {
                match parse_posting(tokenizer, transaction.transaction_type, &transaction.payee) {
                    // Although here we already know the kind of the posting (virtual, real),
                    // we deal with that in the next phase of parsing
                    Ok(posting) => transaction.postings.push(posting),
                    Err(e) => {
                        eprintln!("Error while parsing posting.");
                        return Err(tokenizer.error(e));
                    }
                }
                parsed_posting = true;
            }
        }
    }

    Ok(transaction)
}

#[derive(Debug, Clone)]
pub struct RawPosting {
    pub account: String,
    pub money_amount: Option<BigRational>,
    pub money_currency: Option<String>,
    pub cost_amount: Option<BigRational>,
    pub cost_currency: Option<String>,
    pub cost_type: Option<PriceType>,
    pub balance_amount: Option<BigRational>,
    pub balance_currency: Option<String>,
    pub comments: Vec<Comment>,
    pub amount_expr: Option<String>,
    pub kind: PostingType,
    pub payee: Option<String>,
}

/// Parses a posting
///
fn parse_posting(
    tokenizer: &mut Tokenizer,
    transaction_type: TransactionType,
    default_payee: &Option<String>,
) -> Result<RawPosting, ParserError> {
    let mut account = String::new();
    let mut posting_type = PostingType::Real;
    let mut finished = false;
    // Get the account name
    loop {
        let c = tokenizer.next();
        if !c.is_whitespace() {
            account.push(c);
        } else if (c == '\t') | (c == '\n') {
            if c == '\n' {
                // println!("1 {}-", account);
                tokenizer.line_index -= 1;
                tokenizer.position -= 1;
                finished = true;
            }
            break;
        } else {
            let d = tokenizer.next();
            if d.is_whitespace() | (c == '\n') {
                break;
            } else {
                account.push(c);
                account.push(d);
            }
        }
    }
    // See if it is a virtual account
    match &account[0..1] {
        "(" => {
            posting_type = PostingType::Virtual;
            if account.pop().unwrap() != ')' {
                return Err(ParserError::UnexpectedInput(Some(
                    "Expected ')'".to_string(),
                )));
            }
        }
        "[" => {
            posting_type = PostingType::VirtualMustBalance;
            if account.pop().unwrap() != ']' {
                return Err(ParserError::UnexpectedInput(Some(
                    "Expected ']'".to_string(),
                )));
            }
        }
        _ => {}
    }

    // If it is not real, get the actual account name
    match posting_type {
        PostingType::Real => {}
        PostingType::Virtual | PostingType::VirtualMustBalance => {
            account = account[1..].trim().to_string();
            // println!("{} is a virtual account {:?}", account, posting_type)
        }
    }

    let mut posting = RawPosting {
        account,
        money_amount: None,
        money_currency: None,
        cost_amount: None,
        cost_currency: None,
        cost_type: None,
        balance_amount: None,
        balance_currency: None,
        comments: Vec::new(),
        amount_expr: None,
        kind: posting_type,
        payee: default_payee.clone(),
    };
    if finished {
        return Ok(posting);
    }
    chars::consume_whitespaces(tokenizer);
    // Amounts
    loop {
        match tokenizer.get_char() {
            Some('(') => {
                // This is a value expression
                posting.amount_expr = Some(chars::get_value_expression(tokenizer));
            }
            Some('\n') => break,
            None => break,
            Some(';') => {
                posting.comments.push(comment::parse(tokenizer));
                return Ok(posting);
            }
            Some('=') => {
                tokenizer.position += 1;
                tokenizer.line_position += 1;
                chars::consume_whitespaces(tokenizer);
                let money = match parse_money(tokenizer) {
                    Ok(money) => money,
                    Err(e) => {
                        // eprintln!("I fail here 218.");
                        return Err(e);
                    }
                };
                posting.balance_amount = Some(money.0);
                posting.balance_currency = Some(money.1);
            }
            Some('@') => {
                if posting.money_amount.is_none() {
                    return Err(ParserError::UnexpectedInput(Some(
                        "Amount not found".to_string(),
                    )));
                }

                tokenizer.position += 1;
                tokenizer.line_position += 1;
                if tokenizer.get_char().unwrap() == '@' {
                    tokenizer.position += 1;
                    tokenizer.line_position += 1;
                    posting.cost_type = Some(PriceType::Total);
                } else {
                    posting.cost_type = Some(PriceType::PerUnit);
                }
                chars::consume_whitespaces(tokenizer);
                let money = match parse_money(tokenizer) {
                    Ok(money) => money,
                    Err(e) => {
                        // eprintln!("I fail here 249.");
                        return Err(e);
                    }
                };
                posting.cost_amount = Some(money.0);
                posting.cost_currency = Some(money.1);
            }
            _ => match parse_money(tokenizer) {
                Ok(money) => {
                    posting.money_amount = Some(money.0);
                    posting.money_currency = Some(money.1);
                }
                Err(e) => match transaction_type {
                    TransactionType::Real | TransactionType::Periodic => return Err(e),
                    TransactionType::Automated => {
                        posting.amount_expr = Some(chars::get_line(tokenizer));

                        tokenizer.line_index -= 1;
                        tokenizer.position -= 1;
                    }
                },
            },
        }
        chars::consume_whitespaces(tokenizer);
    }
    Ok(posting)
}

/// Parses money
fn parse_money(tokenizer: &mut Tokenizer) -> Result<(BigRational, String), ParserError> {
    let mut currency: String;
    let amount: BigRational;

    match tokenizer.get_char() {
        Some(c) if c.is_numeric() | (c == '.') | (c == '-') => {
            amount = match parse_amount(tokenizer) {
                Ok(amount) => amount,
                Err(e) => {
                    // eprintln!("I fail here 286.");
                    return Err(e);
                }
            };
            currency = chars::get_string(tokenizer);
            if currency.starts_with("\"") {
                let n = currency.len();
                currency = currency[1..n - 1].to_string();
            }
        }
        Some(_) => {
            currency = chars::get_string(tokenizer);
            if currency.starts_with("\"") {
                let n = currency.len();
                currency = currency[1..n - 1].to_string();
            }
            amount = match parse_amount(tokenizer) {
                Ok(amount) => amount,
                Err(e) => {
                    // eprintln!("I fail here 297.");
                    return Err(e);
                }
            };
        }
        None => {
            return Err(ParserError::UnexpectedInput(Some(
                "Expected ammount missing".to_string(),
            )));
        }
    }
    Ok((amount, currency))
}

fn parse_amount(tokenizer: &mut Tokenizer) -> Result<BigRational, ParserError> {
    let mut num = String::new();
    let mut den = "1".to_string();
    let mut decimal = false;
    loop {
        match tokenizer.get_char() {
            Some('.') => {
                if decimal {
                    return Err(ParserError::UnexpectedInput(Some(
                        "Too many decimal separators".to_string(),
                    )));
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
    Ok(BigRational::new(
        match BigInt::from_str(num.as_str()) {
            Ok(n) => n,
            Err(_) => {
                return Err(ParserError::UnexpectedInput(Some(
                    "Wrong number format".to_string(),
                )));
            }
        },
        match BigInt::from_str(den.as_str()) {
            Ok(d) => d,
            Err(_) => return Err(ParserError::UnexpectedInput(None)),
        },
    ))
}

fn parse_date(input_str: &str) -> NaiveDate {
    // yyyy-mm-dd is 10 characters -- guaranted by the regexp, but it comes with maybe stuff in the front
    let len = input_str.len();
    let date_str = &input_str[len - 10..len];
    assert!(date_str.len() == 10, date_str.to_string());
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
