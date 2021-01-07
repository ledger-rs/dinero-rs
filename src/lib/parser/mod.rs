//! Parser module

mod chars;
mod comment;
mod include;
mod transaction;

use crate::{ErrorType, Error, parser};
use std::path::{Path, PathBuf};
use std::fs::read_to_string;
use crate::parser::chars::LineType;
use crate::ledger::{Comment, Transaction};
use std::collections::{HashSet, HashMap};
use colored::{ColoredString, Colorize};
use chrono::NaiveDate;
use std::str::FromStr;

pub enum Item {
    Comment(Comment),
    Transaction(Transaction<parser::transaction::Posting>),
    Directive,
}

#[derive(Debug, Clone)]
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

impl<'a> From<&'a str> for Tokenizer<'a> {
    fn from(content: &'a str) -> Self {
        Tokenizer {
            file: None,
            content: String::from(content),
            line_index: 0,
            line_position: 0,
            line_string: "",
            line_characters: Vec::new(),
            position: 0,
            seen_files: HashSet::new(),
        }
    }
}

impl<'a> From<String> for Tokenizer<'a> {
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
                    content: content,
                    line_index: 0,
                    line_position: 0,
                    line_string: "",
                    line_characters: vec![],
                    position: 0,
                    seen_files,
                }
            }
            Err(err) => { panic!(ErrorType::CannotReadFile(err.to_string())) }
        }
    }
}

/// Parses a string into Items
impl<'a> Tokenizer<'a> {
    pub fn parse(&'a mut self) -> Result<Vec<Item>, Error> {
        let mut items: Vec<Item> = Vec::new();
        let len = self.content.len();
        while self.position < len {
            match chars::consume_whitespaces_and_lines(self) {
                LineType::Blank => match self.get_char() {
                    Some(c) => match c {
                        ';' | '!' | '*' | '%' | '#' => items.push(Item::Comment(comment::parse(self))),
                        c if c.is_numeric() => items.push(Item::Transaction(transaction::parse(self)?)),
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
                LineType::Indented => { return Err(self.error(ErrorType::ParserError)); }
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
    fn error(&self, err: ErrorType) -> Error {
        let mut message = vec![ColoredString::from("\n")];
        // TODO not the fastest
        for (i, line) in self.content.lines().enumerate() {
            if i < self.line_index - 1 { continue; };
            if i > self.line_index { break; };
            message.push(ColoredString::from(line));
        }
        let mut line = message.pop().unwrap().cyan();
        message.push(ColoredString::from("\n"));
        message.push(line.clone());
        message.push(ColoredString::from("\n"));
        for i in 0..line.len().to_owned() {
            message.push(if i == self.line_position { "^".bold() } else { "-".bold() });
        }
        //message.push(ColoredString::from("\n"));
        Error {
            message,
            error_type: err,
        }
    }

    pub fn next(&mut self) -> char {
        let c: char = self.content.chars().collect::<Vec<char>>()[self.position];
        match c {
            '\n' => {
                self.line_position = 0;
                self.line_index += 1;
            }
            _ => { self.line_position += 1; }
        }
        self.position += 1;
        c
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