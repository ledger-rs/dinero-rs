use crate::models::{Account, Origin};
use crate::parser::chars::LineType;
use crate::parser::{chars, Tokenizer};
use crate::ParserError;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<Account, ParserError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(format!("{}{}",
        r"(account) +"        , // directive commodity
        r"(.*)"                 , // description
        ).as_str()).unwrap();
    }
    let mystr = chars::get_line(tokenizer);
    let caps = RE.captures(mystr.as_str()).unwrap();

    let mut name = String::new();
    let mut detected: bool = false;
    let mut default = false;
    let mut aliases = HashSet::new();
    let mut check = Vec::new();
    let mut assert = Vec::new();
    let mut payee: Vec<Regex> = Vec::new();
    let mut note = None;
    let mut isin = None;

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
                        name = m.as_str().trim().to_string()
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }

    if !detected {
        return Err(ParserError::UnexpectedInput(None));
    }
    while let LineType::Indented = chars::consume_whitespaces_and_lines(tokenizer) {
        match tokenizer.get_char().unwrap() {
            ';' => {
                chars::get_line(tokenizer);
            } // ignore comments
            _ => match chars::get_string(tokenizer).as_str() {
                "note" => note = Some(chars::get_line(tokenizer).trim().to_string()),
                "isin" => isin = Some(chars::get_line(tokenizer).trim().to_string()),
                "alias" => {
                    aliases.insert(chars::get_line(tokenizer).trim().to_string());
                }
                "check" => {
                    check.push(chars::get_line(tokenizer).trim().to_string());
                }
                "assert" => {
                    assert.push(chars::get_line(tokenizer).trim().to_string());
                }
                "payee" => {
                    payee.push(Regex::new(chars::get_line(tokenizer).trim()).unwrap());
                }
                "default" => default = true,
                found => {
                    eprintln!("Error while parsing posting.");
                    return Err(ParserError::UnexpectedInput(Some(format!(
                        "Found {}. Expected one of alias, note, isin, check, assert, payee.",
                        found
                    ))));
                }
            },
        }
    }
    let account = Account::new(
        name,
        Origin::FromDirective,
        note,
        isin,
        aliases,
        check,
        assert,
        payee,
        default,
    );
    Ok(account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HasName;
    use crate::List;

    #[test]
    fn test_spaces_in_account_names() {
        let mut tokenizer = Tokenizer::from("account An account name with spaces   ".to_string());
        let account = parse(&mut tokenizer).unwrap();
        assert_eq!(account.get_name(), "An account name with spaces");
    }

    #[test]
    fn test_parse_account() {
        let mut tokenizer = Tokenizer::from(
            "account Assets:Checking account
    alias checking
    note An account for everyday expenses
    ; this line will be ignored
    alias checking account
    isin 123456789
    payee Employer
    "
            .to_string(),
        );
        let account = parse(&mut tokenizer).unwrap();
        assert!(!account.is_default(), "Not a default account");
        assert_eq!(account.get_name(), "Assets:Checking account");
    }

    #[test]
    fn get_account_from_alias() {
        let mut tokenizer = Tokenizer::from(
            "account Assets:MyAccount
    alias myAccount
    "
            .to_string(),
        );
        let account = parse(&mut tokenizer).unwrap();
        assert!(!account.is_default(), "Not a default account");
        assert_eq!(account.get_name(), "Assets:MyAccount");

        let mut accounts = List::<Account>::new();
        accounts.insert(account);

        assert!(accounts.get("myAccount").is_ok())
    }
}
