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

use crate::models::{Account, Comment, Currency, Payee, Transaction};
use crate::{models, List, ParserError};
use pest::Parser;

mod include;
// pub mod tokenizers;
pub mod tokenizers;
mod utils;
pub mod value_expr;

use tokenizers::{account, comment, commodity, payee, price, tag, transaction};

#[derive(Parser)]
#[grammar = "grammar/grammar.pest"]
pub struct GrammarParser;

#[derive(Debug, Clone)]
pub struct ParsedLedger {
    pub accounts: List<Account>,
    pub payees: List<Payee>,
    pub commodities: List<Currency>,
    pub transactions: Vec<Transaction<transaction::RawPosting>>,
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
    content: String,
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
                    content,
                    seen_files,
                }
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }
}

impl<'a> From<String> for Tokenizer<'a> {
    fn from(content: String) -> Self {
        Tokenizer {
            file: None,
            content,
            seen_files: HashSet::new(),
        }
    }
}

impl<'a> Tokenizer<'a> {
    /// Parses a string into a parsed ledger. It allows for recursion,
    /// i.e. the include keyword is properly handled
    pub fn tokenize(&'a mut self) -> ParsedLedger {
        let mut ledger: ParsedLedger = ParsedLedger::new();
        match GrammarParser::parse(Rule::journal, self.content.as_str()) {
            Ok(mut parsed) => {
                let mut elements = parsed.next().unwrap().into_inner();
                while let Some(element) = elements.next() {
                    match element.as_rule() {
                        Rule::directive => {
                            let inner = element.into_inner().next().unwrap();
                            match inner.as_rule() {
                                Rule::include => {
                                    self.include(inner);
                                }
                                Rule::price => {
                                    ledger.prices.push(self.parse_price(inner));
                                }
                                _ => {}
                            }
                        }
                        Rule::transaction => {
                            ledger.transactions.push(self.parse_transaction(element));
                        }
                        x => {
                            eprintln!("{:?}", x);
                        }
                    }
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
        // dbg!(&ledger);
        ledger
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let content = "".to_string();
        let mut tokenizer = Tokenizer::from(content);
        let items = tokenizer.tokenize();
        assert_eq!(items.len(), 0, "Should be empty");
    }

    #[test]
    fn test_only_spaces() {
        let content = "\n\n\n\n\n".to_string();
        let mut tokenizer = Tokenizer::from(content);
        let items = tokenizer.tokenize();
        assert_eq!(items.len(), 0, "Should be empty")
    }
}
*/
