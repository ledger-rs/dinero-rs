use crate::parser::{chars, Item, Tokenizer};

use crate::{Error, ErrorType};
use glob::glob;
use std::path::PathBuf;

/// Handles include directive
pub(super) fn parse<'a>(tokenizer: &'a mut Tokenizer) -> Result<Vec<Item>, Error> {
    chars::consume_str(tokenizer, &"include ".to_string())?;
    let pattern = chars::get_line(tokenizer);
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                files.push(path.clone());
                match tokenizer.seen_files.get(&path) {
                    Some(_) => return Err(tokenizer.error(ErrorType::IncludeLoop)),
                    None => (),
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    let mut items: Vec<Item> = Vec::new();
    for file in files {
        let mut inner_tokenizer: Tokenizer = Tokenizer::from(&file);
        for p in tokenizer.seen_files.iter() {
            inner_tokenizer.seen_files.insert(*p);
        }
        let mut new_items: Vec<Item> = inner_tokenizer.parse()?;
        items.append(&mut new_items);
    }
    Ok(items)
}
