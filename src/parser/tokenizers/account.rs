use super::super::Rule;
use crate::parser::Tokenizer;
use crate::{
    models::{Account, Origin},
    parser::utils::parse_string,
};

use pest::iterators::Pair;
use regex::Regex;
use std::collections::HashSet;

impl<'a> Tokenizer<'a> {
    pub(crate) fn parse_account(&self, element: Pair<Rule>) -> Account {
        let mut parsed = element.into_inner();
        let name = parse_string(parsed.next().unwrap());
        let mut default = false;
        let mut aliases = HashSet::new();
        let mut check = Vec::new();
        let mut assert = Vec::new();
        let mut payee: Vec<Regex> = Vec::new();
        let mut note = None;
        let mut iban = None;
        let mut country = None;

        while let Some(part) = parsed.next() {
            match part.as_rule() {
                Rule::account_property => {
                    let mut property = part.into_inner();
                    match property.next().unwrap().as_rule() {
                        Rule::alias => {
                            aliases.insert(parse_string(property.next().unwrap()));
                        }
                        Rule::note => note = Some(parse_string(property.next().unwrap())),
                        Rule::iban => iban = Some(parse_string(property.next().unwrap())),
                        Rule::country => country = Some(parse_string(property.next().unwrap())),
                        Rule::assert => assert.push(parse_string(property.next().unwrap())),
                        Rule::check => check.push(parse_string(property.next().unwrap())),
                        Rule::payee_subdirective => payee.push(
                            Regex::new(parse_string(property.next().unwrap()).trim()).unwrap(),
                        ),
                        _ => {}
                    }
                }
                Rule::flag => default = true,
                _ => {}
            }
        }
        let account = Account::new(
            name,
            Origin::FromDirective,
            note,
            iban,
            country,
            aliases,
            check,
            assert,
            payee,
            default,
        );
        account
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HasName;
    use crate::List;

    #[test]
    fn test_spaces_in_account_names() {
        let mut tokenizer = Tokenizer::from("account An account name with spaces   ".to_string());
        // let account = parse(&mut tokenizer).unwrap();
        // assert_eq!(account.get_name(), "An account name with spaces");
        unimplemented!("test spaces in account names");
    }

    #[test]
    fn test_parse_account() {
        let mut tokenizer = Tokenizer::from(
            "account Assets:Checking account
    alias checking
    note An account for everyday expenses
    ; this line will be ignored
    alias checking account
    iban 123456789
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
*/
