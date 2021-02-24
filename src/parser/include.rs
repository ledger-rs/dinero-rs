use crate::parser::{chars, ParsedLedger, Tokenizer};

use crate::{Error, ParserError};
use glob::glob;
use std::path::PathBuf;

/// Handles include directive
pub(super) fn parse(tokenizer: &mut Tokenizer) -> Result<ParsedLedger, Error> {
    chars::consume_str(tokenizer, &"include ".to_string())?;
    let mut pattern = String::new();
    let mut files: Vec<PathBuf> = Vec::new();
    if let Some(current_path) = tokenizer.file {
        // let mut parent = format!("{:?}", current_path.parent().unwrap()).replace('"', "");
        let mut parent = current_path.parent().unwrap().to_str().unwrap().to_string();
        if parent.len() == 0 {
            parent.push('.')
        }
        parent.push('/');
        pattern.push_str(parent.as_str());
    }
    pattern.push_str(chars::get_line(tokenizer).as_str());
    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                files.push(path.clone());
                match tokenizer.seen_files.get(&path) {
                    Some(_) => {
                        return Err(tokenizer.error(ParserError::IncludeLoop(format!(
                            "Cycle detected. {:?}",
                            &path
                        ))))
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
        for p in tokenizer.seen_files.iter() {
            inner_tokenizer.seen_files.insert(*p);
        }
        let mut new_items: ParsedLedger = inner_tokenizer.tokenize()?;
        items.append(&mut new_items);
    }
    Ok(items)
}
