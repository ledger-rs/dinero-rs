use std::collections::HashSet;

use regex::Regex;

use crate::models::{Origin, Payee};
use crate::parser::Tokenizer;

use super::super::Rule;

use crate::parser::utils::parse_string;

use pest::iterators::Pair;

impl<'a> Tokenizer<'a> {
    pub(crate) fn parse_payee(&self, element: Pair<Rule>) -> Payee {
        let mut parsed = element.into_inner();
        let name = parse_string(parsed.next().unwrap());
        let mut note: Option<String> = None;
        let mut alias = HashSet::new();

        while let Some(part) = parsed.next() {
            match part.as_rule() {
                Rule::comment => {}
                Rule::payee_property => {
                    let mut property = part.into_inner();
                    match property.next().unwrap().as_rule() {
                        Rule::alias => {
                            alias.insert(parse_string(property.next().unwrap()));
                        }
                        Rule::note => note = Some(parse_string(property.next().unwrap())),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let alias_regex: Vec<Regex> = alias
            .iter()
            .map(|x| Regex::new(x.clone().as_str()).unwrap())
            .collect();
        let payee = Payee::new(name, note, alias, alias_regex, Origin::FromDirective);
        payee
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::HasName, CommonOpts};

    #[test]
    //#[should_panic(expected = "Can't compare different currencies. € and USD.")]
    fn parse_ko() {
        let input = "payee ACME  ; From the Looney Tunes\n\tWrong Acme, Inc.\n".to_string();
        let mut tokenizer = Tokenizer::from(input);
        let items = tokenizer.tokenize(&CommonOpts::new());
        assert_eq!(items.payees.len(), 0);
    }

    #[test]
    fn parse_ok() {
        let input = "payee ACME\n\talias Acme, Inc.\n".to_string();
        let mut tokenizer = Tokenizer::from(input);

        let items = tokenizer.tokenize(&CommonOpts::new());
        assert_eq!(items.payees.len(), 1);

        assert!(items.payees.get("acme, inc.").is_ok());
        let payee = &items.payees.get("acme").unwrap();
        assert_eq!(payee.get_name(), "ACME");
    }
}
