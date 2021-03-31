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
use crate::{models, List};
use pest::Parser;

mod include;
// pub mod tokenizers;
pub mod tokenizers;
pub(crate) mod utils;
pub mod value_expr;

use tokenizers::transaction;

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
    pub files: Vec<PathBuf>,
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
            files: vec![],
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
        self.files.append(&mut other.files);
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
        if let Some(file) = self.file {
            ledger.files.push(file.clone());
        }
        match GrammarParser::parse(Rule::journal, self.content.as_str()) {
            Ok(mut parsed) => {
                let mut elements = parsed.next().unwrap().into_inner();
                while let Some(element) = elements.next() {
                    match element.as_rule() {
                        Rule::directive => {
                            let inner = element.into_inner().next().unwrap();
                            match inner.as_rule() {
                                Rule::include => {
                                    // This is the special case
                                    let mut new_ledger = self.include(inner);
                                    ledger.append(&mut new_ledger);
                                }
                                Rule::price => {
                                    ledger.prices.push(self.parse_price(inner));
                                }
                                Rule::tag_dir => {
                                    ledger.tags.push(self.parse_tag(inner));
                                }
                                Rule::commodity => {
                                    let commodity = self.parse_commodity(inner);
                                    ledger.commodities.remove(&commodity);
                                    ledger.commodities.insert(commodity);
                                }
                                Rule::account_dir => {
                                    ledger.accounts.insert(self.parse_account(inner));
                                }
                                Rule::payee_dir => {
                                    ledger.payees.insert(self.parse_payee(inner));
                                }
                                _ => {}
                            }
                        }
                        Rule::transaction | Rule::automated_transaction => {
                            let transaction = self.parse_transaction(element);
                            for posting in transaction.postings.borrow().iter() {
                                let currencies = &[
                                    (&posting.money_currency, &posting.money_format),
                                    (&posting.cost_currency, &posting.cost_format),
                                    (&posting.balance_currency, &posting.balance_format),
                                ];
                                for (currency, format) in currencies {
                                    if let Some(c) = currency {
                                        let found = ledger.commodities.get(c);
                                        if found.is_err() {
                                            let mut commodity = Currency::from(c.as_str());
                                            commodity
                                                .set_format(format.as_ref().unwrap().to_owned());
                                            ledger.commodities.insert(commodity);
                                        }
                                    }
                                }
                            }
                            ledger.transactions.push(transaction);
                        }
                        _x => {
                            // eprintln!("{:?}", x);
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(file) = &self.file {
                    eprintln!("Can't parse {:?} {}", file, e);
                }
                eprintln!("Error found in line {}", e)
            }
        }
        // dbg!(&ledger);
        ledger
    }
}
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
