use crate::models::Tag;

use crate::parser::Tokenizer;

use super::super::Rule;

use crate::parser::utils::parse_string;

use pest::iterators::Pair;

impl<'a> Tokenizer<'a> {
    pub(crate) fn parse_tag(&self, element: Pair<Rule>) -> Tag {
        let mut parsed = element.into_inner();
        let name = parse_string(parsed.next().unwrap());

        let mut check = vec![];
        let mut assert = vec![];

        while let Some(part) = parsed.next() {
            match part.as_rule() {
                Rule::commodity_property => {
                    let mut property = part.into_inner();
                    match property.next().unwrap().as_rule() {
                        Rule::check => {
                            check.push(parse_string(property.next().unwrap()));
                        }
                        Rule::assert => assert.push(parse_string(property.next().unwrap())),
                        _ => {}
                    }
                }
                _x => {}
            }
        }
        Tag {
            name,
            check,
            assert,
            value: None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommonOpts, models::HasName};

    #[test]
    fn test_spaces_in_tag_names() {
        let mut tokenizer = Tokenizer::from("tag   A tag name with spaces   ".to_string());
        let items = tokenizer.tokenize(&CommonOpts::new());
        let tag = &items.tags[0];
        assert_eq!(tag.get_name(), "A tag name with spaces");
    }
}

