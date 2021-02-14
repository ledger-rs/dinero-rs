//! Parser module
//!
//! The parser takes an input string (or file) and translates it into tokens with a tokenizer
//! These tokens are one of:
//! - Directive account
//! - Directive payee
//! - Directive commodity

use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;

use colored::{ColoredString, Colorize};

use chars::LineType;

use crate::models::{Account, Comment, Currency, Payee, Transaction};
use crate::{models, Error, List, ParserError};

mod chars;
mod include;
pub mod tokenizers;

use tokenizers::{account, comment, commodity, payee, price, tag, transaction};

#[derive(Debug, Clone)]
pub struct ParsedLedger {
    pub accounts: List<Account>,
    pub payees: List<Payee>,
    pub commodities: List<Currency>,
    pub transactions: Vec<Transaction<transaction::Posting>>,
    pub prices: Vec<models::ParsedPrice>,
    pub comments: Vec<Comment>,
    pub tags: Vec<models::Tag>,
}

impl ParsedLedger {
    pub fn new() -> Self {
        ParsedLedger {
            accounts: List::<Account>::new(),
            payees: List::<models::Payee>::new(),
            commodities: List::<Currency>::new(),
            transactions: vec![],
            prices: vec![],
            comments: vec![],
            tags: vec![],
        }
    }
    pub fn append(&mut self, other: &mut ParsedLedger) {
        self.accounts.append(&other.accounts);
        self.payees.append(&other.payees);
        self.commodities.append(&other.commodities);
        self.transactions.append(&mut other.transactions);
        self.comments.append(&mut other.comments);
        self.transactions.append(&mut other.transactions);
        self.prices.append(&mut other.prices);
    }

    pub fn len(&self) -> usize {
        self.accounts.len()
            + self.payees.len()
            + self.commodities.len()
            + self.transactions.len()
            + self.prices.len()
            + self.comments.len()
            + self.tags.len()
    }
}

/// A struct for holding data about the string being parsed
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
                panic!(ParserError::CannotReadFile(err.to_string()))
            }
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

impl<'a> Tokenizer<'a> {
    /// Parses a string into a parsed ledger. It allows for recursivity,
    /// i.e. the include keyword is properly handled
    pub fn tokenize(&'a mut self) -> Result<ParsedLedger, Error> {
        let mut ledger: ParsedLedger = ParsedLedger::new();
        let len = self.content.iter().count();
        while self.position < len {
            match chars::consume_whitespaces_and_lines(self) {
                LineType::Blank => match self.get_char() {
                    Some(c) => match c {
                        ';' | '!' | '*' | '%' | '#' => ledger.comments.push(comment::parse(self)),
                        c if c.is_numeric() => ledger.transactions.push(transaction::parse(self)?),
                        '=' => ledger
                            .transactions
                            .push(transaction::parse_automated_transaction(self)?),
                        'i' => {
                            // This is the special case
                            let mut new_ledger = include::parse(self)?;
                            ledger.append(&mut new_ledger);
                        }
                        'c' => {
                            ledger.commodities.insert(match commodity::parse(self) {
                                Ok(x) => x,
                                Err(e) => return Err(self.error(e)),
                            });
                        }
                        'p' => {
                            ledger.payees.insert(match payee::parse(self) {
                                Ok(x) => x,
                                Err(e) => return Err(self.error(e)),
                            });
                        }
                        't' => {
                            ledger.tags.push(match tag::parse(self) {
                                Ok(x) => x,
                                Err(e) => return Err(self.error(e)),
                            });
                        }
                        'a' => {
                            ledger.accounts.insert(match account::parse(self) {
                                Ok(x) => x,
                                Err(e) => return Err(self.error(e)),
                            });
                        }
                        'P' => {
                            ledger.prices.push(match price::parse(self) {
                                Ok(x) => x,
                                Err(e) => return Err(self.error(e)),
                            });
                        }
                        _ => {
                            return Err(self.error(ParserError::UnexpectedInput(None)));
                        }
                    },
                    None => continue,
                },
                LineType::Indented => {
                    return Err(self.error(ParserError::UnexpectedInput(Some(
                        "Unexpected indentation".to_string(),
                    ))));
                }
            };
        }
        Ok(ledger)
    }
    fn get_char(&self) -> Option<char> {
        match self.content.get(self.position) {
            Some(c) => Some(*c),
            None => None,
        }
    }
    pub fn error(&self, err: ParserError) -> Error {
        let number_length = format!("{}", self.line_index + 1).len();
        let mut message = Vec::new();
        message.push(format!("Error: ").bold().bright_red());
        message.push(format!("{:?}", err).bold());
        if let Some(file) = self.file {
            // message.push(ColoredString::from(" while parsing "));
            message.push(
                format!(
                    "\n{:width$}{} {:?}",
                    "",
                    "-->".blue(),
                    file,
                    width = number_length
                )
                .bold(),
            );
            message.push(ColoredString::from(
                format!(":{}:{}\n", self.line_index + 1, self.line_position + 1).as_str(),
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
            message.push(format!("{:>width$} |{:6}", i + 1, "", width = number_length).blue());
            // message.push(ColoredString::from(line));
            message.push(format!("{}\n", line).as_str().into());
        }
        let line = message.pop().unwrap().cyan();
        message.push(line.clone());
        message.push(
            format!(
                " {:width$}{}{:6}",
                "",
                "|".blue(),
                "",
                width = number_length
            )
            .as_str()
            .into(),
        );
        for i in 0..line.len().to_owned() {
            message.push(if i == self.line_position {
                "^".bold()
            } else {
                "-".bold()
            });
        }
        Error { message }
    }

    pub fn next(&mut self) -> char {
        if self.position >= self.content.len() {
            eprintln!(
                "{}",
                self.error(ParserError::UnexpectedInput(Some(
                    "end of file".to_string()
                )))
            );
            panic!();
        }
        let c: char = self.content[self.position];
        match c {
            '\n' => {
                self.line_position = 0;
                self.line_index += 1;
            }
            _ => self.line_position += 1,
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
        let items = tokenizer.tokenize().unwrap();
        assert_eq!(items.len(), 0, "Should be empty");
    }

    #[test]
    fn test_only_spaces() {
        let content = "\n\n\n\n\n".to_string();
        let mut tokenizer = Tokenizer::from(content);
        let items = tokenizer.tokenize().unwrap();
        assert_eq!(items.len(), 0, "Should be empty")
    }

    #[test]
    #[should_panic]
    fn next_panic_gracefully() {
        let content = "this is my content".to_string();
        let mut tokenizer = Tokenizer::from(content);
        tokenizer.position = 100; // deliberately a large number
        tokenizer.next(); // This should panic
    }
}
