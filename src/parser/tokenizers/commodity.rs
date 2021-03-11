use super::super::Rule;
use std::collections::HashSet;

use crate::models::{Comment, Currency, Origin};
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

        while let Some(part) = parsed.next() {
            match part.as_rule() {
                Rule::comment => comments.push(Comment {
                    comment: parse_string(part),
                }),
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
                x => panic!("{:?} not expected", x),
            }
        }

        let currency = Currency {
            name: name.trim().to_string(),
            origin: Origin::FromDirective,
            note,
            aliases,
            format,
            default,
            precision: None,
        };
        currency
    }
}
