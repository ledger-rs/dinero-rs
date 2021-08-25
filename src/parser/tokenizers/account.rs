use super::super::Rule;
use crate::parser::Tokenizer;
use crate::{models::Account, parser::utils::parse_string};

use pest::iterators::Pair;
use regex::Regex;

impl<'a> Tokenizer<'a> {
    pub(crate) fn parse_account(&self, element: Pair<Rule>) -> Account {
        let mut parsed = element.into_inner();
        let name = parse_string(parsed.next().unwrap());

        let mut account = Account::from_directive(name);

        for part in parsed {
            match part.as_rule() {
                Rule::account_property => {
                    let mut property = part.into_inner();
                    match property.next().unwrap().as_rule() {
                        Rule::alias => {
                            account
                                .aliases
                                .insert(parse_string(property.next().unwrap()));
                        }
                        Rule::note => account.note = Some(parse_string(property.next().unwrap())),
                        Rule::iban => account.iban = Some(parse_string(property.next().unwrap())),
                        Rule::country => {
                            account.country = Some(parse_string(property.next().unwrap()))
                        }
                        Rule::assert => account.assert.push(parse_string(property.next().unwrap())),
                        Rule::check => account.check.push(parse_string(property.next().unwrap())),
                        Rule::payee_subdirective => account.payee.push(
                            Regex::new(parse_string(property.next().unwrap()).trim()).unwrap(),
                        ),
                        _ => {}
                    }
                }
                Rule::flag => account.default = true,
                _ => {}
            }
        }
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
