use lazy_static::lazy_static;
use regex::Regex;

use crate::models::Tag;
use crate::parser::Tokenizer;
use crate::ParserError;

pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Result<Tag, ParserError> {
    unimplemented!("tag");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HasName;

    #[test]
    fn test_spaces_in_tag_names() {
        let mut tokenizer = Tokenizer::from("tag   A tag name with spaces   ".to_string());
        let tag = parse(&mut tokenizer).unwrap();
        assert_eq!(tag.get_name(), "A tag name with spaces");
    }
}
