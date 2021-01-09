use super::Tokenizer;
use crate::{Error, ErrorType};

pub(super) enum LineType {
    Blank,
    Indented,
}

pub(super) fn consume_whitespaces_and_lines(tokenizer: &mut Tokenizer) -> LineType {
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

pub(super) fn consume_whitespaces(tokenizer: &mut Tokenizer) {
    let chars = tokenizer.content.chars().collect::<Vec<char>>();
    while let Some(c) = chars.get(tokenizer.position) {
        if !c.is_whitespace() { break; }
        if *chars.get(tokenizer.position).unwrap() == '\n' { break; }
        tokenizer.position += 1;
        tokenizer.line_position += 1;
    }
}

pub(super) fn consume_str<'a>(
    tokenizer: &'a mut Tokenizer,
    string: &'a String,
) -> Result<(), Error> {
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
            None => return Err(tokenizer.error(ErrorType::ParserError)),
            Some(_) => return Err(tokenizer.error(ErrorType::UnexpectedInput)),
        }
    }
    Ok(())
}

pub(super) fn get_line(tokenizer: &mut Tokenizer) -> String {
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

pub(super) fn get_string(tokenizer: &mut Tokenizer) -> String {
    consume_whitespaces(tokenizer);
    let chars = tokenizer.content.chars().collect::<Vec<char>>();
    let mut retval: Vec<char> = Vec::new();
    let mut quote = false;
    while let Some(c) = chars.get(tokenizer.position) {
        if *c == '\n' {
            break;
        } else if *c == '"' {
            if chars.len() == 0 {
                quote = true
            } else if quote {
                break;
            }
        }
        if (c.is_whitespace() | c.is_numeric() | (*c == '.') | (*c == '-')) & !quote {
            break;
        }
        retval.push(*c);
        tokenizer.position += 1;
        tokenizer.line_position += 1;
    }
    consume_whitespaces(tokenizer);
    retval.iter().collect()
}
