//! Parser module

mod chars;
mod comment;
mod include;

use crate::Error;
use std::path::{Path, PathBuf};
use std::fs::read_to_string;
use crate::parser::chars::LineType;
use crate::ledger::JournalComment;
use std::collections::{HashSet, HashMap};

pub enum Item {
    Comment(JournalComment),
    Transaction,
    Directive,
}

pub struct Tokenizer<'a> {
    file: Option<&'a PathBuf>,
    content: String,
    line_index: usize,
    line_position: usize,
    line_string: &'a str,
    line_characters: Vec<char>,
    position: usize,
    seen_files: HashSet<&'a PathBuf>,
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
            seen_files: HashSet::new(),
        }
    }
}

impl<'a> From<&'a PathBuf> for Tokenizer<'a> {
    fn from(file: &'a PathBuf) -> Self {
        match read_to_string(file) {
            Ok(content) => {
                let mut seen_files: HashSet<&PathBuf> = HashSet::new();
                seen_files.insert(file);
                Tokenizer {
                    file: Some(file),
                    content,
                    line_index: 0,
                    line_position: 0,
                    line_string: "",
                    line_characters: vec![],
                    position: 0,
                    seen_files,
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
            match chars::consume_whitespaces(self) {
                LineType::Blank => match self.get_char() {
                    Some(c) => match c {
                        ';' | '!' | '*' | '%' | '#' => items.push(comment::parse(self)),
                        'i' => {
                            // This is the special case
                            let mut new_items = include::parse(self)?;
                            items.append(&mut new_items);
                        }
                        _ => // TODO Change by an Error
                            panic!("Unexpected char '{}'", c),
                    }
                    None => continue,
                },
                LineType::Indented => { return Err(Error::ParserError); }
            };
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