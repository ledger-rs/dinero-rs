//! Parser module

mod chars;
mod comment;

use crate::Error;
use std::path::Path;
use std::fs::read_to_string;
use crate::parser::chars::LineType;
use crate::ledger::JournalComment;

pub enum Item {
    Comment(JournalComment),
    Transaction,
    Directive,
}

pub struct Tokenizer<'a> {
    file: Option<&'a Path>,
    content: String,
    line_index: usize,
    line_position: usize,
    line_string: &'a str,
    line_characters: Vec<char>,
    position: usize,
}

impl From<String> for Tokenizer<'_> {
    fn from(content: String) -> Self {
        Tokenizer {
            file: None,
            content,
            line_index: 0,
            line_position: 0,
            line_string: "",
            line_characters: Vec::new(),
            position: 0,
        }
    }
}

impl<'a> From<&'a Path> for Tokenizer<'a> {
    fn from(file: &'a Path) -> Self {
        match read_to_string(file) {
            Ok(content) => {
                Tokenizer {
                    file: Some(file),
                    content,
                    line_index: 0,
                    line_position: 0,
                    line_string: "",
                    line_characters: vec![],
                    position: 0,
                }
            }
            Err(err) => { panic!(Error::CannotReadFile { message: err.to_string() }) }
        }
    }
}

/// Parses a string into Items
impl<'a> Tokenizer<'a> {
    pub fn parse(&'a mut self) -> Result<Vec<Item>, Error> {
        let mut items: Vec<Item> = Vec::new();
        let len = self.content.len();
        while self.position < len {
            let item = match chars::consume_whitespaces(self) {
                LineType::Blank => match self.get_char() {
                    Some(c) => match c {
                        ';' | '!' | '*' | '%' | '#' => comment::parse(self),
                        _ => panic!("Unexpected char"),
                    }
                    None => continue,
                },
                LineType::Indented => { return Err(Error::ParserError); }
            };
            items.push(item);
        }
        Ok(items)
    }
    fn get_char(&self) -> Option<char> {
        match self.content.chars().collect::<Vec<char>>().get(self.position) {
            Some(c) => Some(*c),
            None => None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let content = "".to_string();
        let mut tokenizer = Tokenizer::from(content);
        let items = tokenizer.parse().unwrap();
        assert_eq!(items.len(), 0, "Should be empty");
    }

    #[test]
    fn test_only_spaces() {
        let content = "\n\n\n\n\n".to_string();
        let mut tokenizer = Tokenizer::from(content);
        let items = tokenizer.parse().unwrap();
        assert_eq!(items.len(), 0, "Should be empty")
    }
}