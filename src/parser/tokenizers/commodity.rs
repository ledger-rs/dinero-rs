use super::super::Rule;
use std::collections::HashSet;

use crate::models::{Comment, Currency, CurrencyDisplayFormat};
use crate::parser::utils::parse_string;
use crate::parser::Tokenizer;

use pest::iterators::Pair;

impl<'a> Tokenizer<'a> {
    pub(crate) fn parse_commodity(&self, element: Pair<Rule>) -> Currency {
        let mut parsed = element.into_inner();
        let name = parse_string(parsed.next().unwrap());
        let mut note: Option<String> = None;
        let mut format: Option<String> = None;
        let mut comments: Vec<Comment> = vec![];
        let mut default = false;
        let mut aliases = HashSet::new();

        for part in parsed {
            match part.as_rule() {
                Rule::comment => comments.push(Comment::from(parse_string(
                    part.into_inner().next().unwrap(),
                ))),
                Rule::commodity_property => {
                    let mut property = part.into_inner();
                    match property.next().unwrap().as_rule() {
                        Rule::alias => {
                            aliases.insert(parse_string(property.next().unwrap()));
                        }
                        Rule::note => note = Some(parse_string(property.next().unwrap())),
                        Rule::format => format = Some(parse_string(property.next().unwrap())),
                        _ => {}
                    }
                }
                Rule::flag => default = true,
                Rule::end => {}
                Rule::blank_line => {}
                x => panic!("{:?} not expected", x),
            }
        }

        let mut currency = Currency::from_directive(name.trim().to_string());
        currency.set_aliases(aliases);
        if default {
            currency.set_default();
        }
        if let Some(n) = note {
            currency.set_note(n);
        }
        if let Some(f) = format {
            currency.format = Some(f.clone());
            currency.set_format(&CurrencyDisplayFormat::from(f.as_str()));
        }

        currency
    }
}
