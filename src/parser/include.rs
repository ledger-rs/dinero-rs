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
        let mut pattern = String::new();
        let mut files: Vec<PathBuf> = Vec::new();
        if let Some(current_path) = self.file {
            let mut parent = current_path.parent().unwrap().to_str().unwrap().to_string();
            if parent.len() == 0 {
                parent.push('.')
            }
            parent.push('/');
            pattern.push_str(parent.as_str());
        }
        let parsed_glob = element.into_inner().next().unwrap().as_str();
        pattern.push_str(parsed_glob);
        for entry in glob(&pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    files.push(path.clone());
                    match self.seen_files.get(&path) {
                        Some(_) => {
                            panic!("Cycle detected. {:?}", &path);
                        }
                        None => (),
                    }
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
        let mut items: ParsedLedger = ParsedLedger::new();
        for file in files {
            let mut inner_tokenizer: Tokenizer = Tokenizer::from(&file);
            for p in self.seen_files.iter() {
                inner_tokenizer.seen_files.insert(*p);
            }
            let mut new_items: ParsedLedger = inner_tokenizer.tokenize();
            items.append(&mut new_items);
        }
        items
    }
}
