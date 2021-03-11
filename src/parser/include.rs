use crate::parser::{ParsedLedger, Rule, Tokenizer};
use glob::glob;
use pest::iterators::Pair;

use std::path::PathBuf;

impl<'a> Tokenizer<'a> {
    /// Handles include directive
    ///
    /// Add the found file of files it it has wildcards in the pattern to the queue of files to process and process them.
    /// TODO this is a good place to include parallelism
    pub fn include(&self, element: Pair<Rule>) -> ParsedLedger {
        unimplemented!("include");
    }
}
