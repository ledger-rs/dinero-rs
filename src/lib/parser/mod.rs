//! Parser module

mod account;
mod chars;
mod comment;
mod commodity;
mod include;
mod payee;
mod price;
mod tag;
pub(crate) mod transaction;

use crate::ledger::{Account, Comment, Currency, Transaction};
use crate::parser::chars::LineType;
use crate::{parser, Error, ErrorType};
use chrono::NaiveDate;
use colored::{ColoredString, Colorize};
use num::rational::Rational64;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Item {
    Comment(Comment),
    Transaction(Transaction<parser::transaction::Posting>),
    Directive(Directive),
    Price(ParsedPrice),
}

#[derive(Debug, Clone)]
pub struct ParsedPrice {
    pub(crate) date: NaiveDate,
    pub(crate) commodity: String,
    pub(crate) other_commodity: String,
    pub(crate) other_quantity: Rational64,
}
#[derive(Debug, Clone)]
pub enum Directive {
    Commodity(Currency),
    Payee {
        name: String,
        note: Option<String>,
        alias: HashSet<String>,
    },
    Account(Account),
    Tag {
        name: String,
        check: Vec<String>,
        assert: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    file: Option<&'a PathBuf>,
    content: Vec<char>,
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
            content: content.chars().collect::<Vec<char>>(),
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
            content: content.chars().collect::<Vec<char>>(),
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
                    content: content.chars().collect::<Vec<char>>(),
                    line_index: 0,
                    line_position: 0,
                    line_string: "",
                    line_characters: vec![],
                    position: 0,
                    seen_files,
                }
            }
            Err(err) => {
                panic!(ErrorType::CannotReadFile(err.to_string()))
            }
        }
    }
}

/// Parses a string into Items
impl<'a> Tokenizer<'a> {
    pub fn parse(&'a mut self) -> Result<Vec<Item>, Error> {
        let mut items: Vec<Item> = Vec::new();
        let len = self.content.iter().count();
        while self.position < len {
            match chars::consume_whitespaces_and_lines(self) {
                LineType::Blank => match self.get_char() {
                    Some(c) => match c {
                        ';' | '!' | '*' | '%' | '#' => {
                            items.push(Item::Comment(comment::parse(self)))
                        }
                        c if c.is_numeric() => {
                            items.push(Item::Transaction(transaction::parse(self)?))
                        }
                        'i' => {
                            // This is the special case
                            let mut new_items = include::parse(self)?;
                            items.append(&mut new_items);
                        }
                        'c' => {
                            items.push(Item::Directive(commodity::parse(self)?));
                        }
                        'p' => {
                            items.push(Item::Directive(payee::parse(self)?));
                        }
                        't' => {
                            items.push(Item::Directive(tag::parse(self)?));
                        }
                        'a' => {
                            items.push(Item::Directive(account::parse(self)?));
                        }
                        'P' => {
                            items.push(Item::Price(price::parse(self)?));
                        }
                        _ => {
                            return Err(self.error(ErrorType::UnexpectedInput));
                        }
                    },
                    None => continue,
                },
                LineType::Indented => {
                    return Err(self.error(ErrorType::ParserError));
                }
            };
        }
        Ok(items)
    }
    fn get_char(&self) -> Option<char> {
        match self.content.get(self.position) {
            Some(c) => Some(*c),
            None => None,
        }
    }
    fn error(&self, err: ErrorType) -> Error {
        let mut message = vec![ColoredString::from("")];
        if let Some(file) = self.file {
            message.push(ColoredString::from("while parsing "));
            message.push(format!("{:?} ", file).bold());
            message.push(ColoredString::from(
                format!(
                    "at position {}:{}\n",
                    self.line_index + 1,
                    self.line_position + 1
                )
                .as_str(),
            ));
        }
        let string = self.content.iter().collect::<String>();
        for (i, line) in string.lines().enumerate() {
            if i < self.line_index - 1 {
                continue;
            };
            if i > self.line_index {
                break;
            };
            message.push(ColoredString::from(line));
        }
        let line = message.pop().unwrap().cyan();
        message.push(ColoredString::from("\n"));
        message.push(line.clone());
        message.push(ColoredString::from("\n"));
        for i in 0..line.len().to_owned() {
            message.push(if i == self.line_position {
                "^".bold()
            } else {
                "-".bold()
            });
        }
        Error {
            message,
            error_type: err,
        }
    }

    pub fn next(&mut self) -> char {
        let c: char = self.content[self.position];
        match c {
            '\n' => {
                self.line_position = 0;
                self.line_index += 1;
            }
            _ => {
                self.line_position += 1;
            }
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
