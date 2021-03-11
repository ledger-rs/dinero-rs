use super::super::Rule;
use crate::models::{Account, Origin};
use crate::parser::Tokenizer;
use crate::ParserError;
use lazy_static::lazy_static;
use num::bigint::ToBigInt;
use pest::iterators::Pair;
use regex::Regex;
use std::collections::HashSet;

impl<'a> Tokenizer<'a> {
    pub(crate) fn parse(&self, element: Pair<Rule>) -> Account {
        unimplemented!("account");
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
*/
