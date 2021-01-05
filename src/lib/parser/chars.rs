use super::Tokenizer;
use crate::Error;

pub(super) enum LineType {
    Blank,
    Indented,
}

pub(super) fn consume_whitespaces(tokenizer: &mut Tokenizer) -> LineType {
    let chars = tokenizer.content.chars().collect::<Vec<char>>();
    while let Some(c) = chars.get(tokenizer.position) {
        if *c == '\n' {
            tokenizer.line_index += 1;
            tokenizer.line_position = 0;
            tokenizer.position += 1;
        } else if c.is_whitespace() | (*c == '\r') {
            tokenizer.position += 1;
            tokenizer.line_position += 1;
        } else {
            match tokenizer.line_position {
                0 => return LineType::Blank,
                _ => return LineType::Indented,
            }
        }
    }
    LineType::Blank
}

pub(super) fn consume_str(tokenizer: &mut Tokenizer, string: &String) -> Result<(), Error> {
    let chars = string.chars().collect::<Vec<char>>();
    let content = tokenizer.content.chars().collect::<Vec<char>>();
    for input in chars.iter() {
        let found = content.get(tokenizer.position);
        match found {
            Some(c) if c == input => {
                tokenizer.line_position += 1;
                tokenizer.position += 1;
                if *input == '\n' {
                    tokenizer.line_index += 1;
                    tokenizer.line_position = 0;
                }
            }
            None => return Err(Error::ParserError),
            Some(c) => return Err(Error::UnexpectedInput),
        }
    }
    Ok(())
}

pub(super) fn consume_line(tokenizer: &mut Tokenizer) -> String {
    let chars = tokenizer.content.chars().collect::<Vec<char>>();
    let mut retval: Vec<char> = Vec::new();
    while let Some(c) = chars.get(tokenizer.position) {
        tokenizer.position += 1;
        if *c == '\n' {
            tokenizer.line_index += 1;
            tokenizer.line_position = 0;
            break;
        }
        retval.push(*c);
    }
    retval.iter().collect()
}
