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
                        _x => {}
                    }
                }
                Rule::flag => account.default = true,
                _x => {},
            }
        }
        account
    }
}

#[cfg(test)]
mod tests {
    use structopt::StructOpt;

    use super::*;
    use crate::models::HasName;
    use crate::CommonOpts;

    #[test]
    fn test_spaces_in_account_names() {
        let mut tokenizer = Tokenizer::from("account An account name with spaces   ".to_string());
        let options = CommonOpts::from_iter(["", "-f", ""].iter());
        let items = tokenizer.tokenize(&options);
        let account = items
            .accounts
            .get("An account name with spaces")
            .unwrap()
            .as_ref();
        assert_eq!(account.get_name(), "An account name with spaces");
        assert_eq!(items.accounts.len(), 1);
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
        let options = CommonOpts::from_iter(["", "-f", ""].iter());
        let items = tokenizer.tokenize(&options);
        let account = items
            .accounts
            .get("Assets:Checking account")
            .unwrap()
            .as_ref();
        assert!(!account.is_default(), "Not a default account");
        assert_eq!(account.get_name(), "Assets:Checking account");
    }

    #[test]
    fn get_account_from_alias() {
        let mut tokenizer = Tokenizer::from(
            "account Assets:MyAccount
    alias myAccount
    check commodity == \"$\"
    assert commodity == \"$\"
    default
    "
            .to_string(),
        );
        let options = CommonOpts::from_iter(["", "-f", ""].iter());
        let items = tokenizer.tokenize(&options);
        let account = items.accounts.get("myAccount").unwrap().as_ref();
        assert!(account.is_default(), "A default account");
        assert_eq!(account.get_name(), "Assets:MyAccount");
        assert!(!account.check.is_empty(), "It has a check");
        assert!(!account.assert.is_empty(), "It has an assert");
    }
}
