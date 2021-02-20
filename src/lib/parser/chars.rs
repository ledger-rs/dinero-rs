use crate::ParserError;

use super::Tokenizer;

pub(super) enum LineType {
    Blank,
    Indented,
}

pub(super) fn consume_whitespaces_and_lines(tokenizer: &mut Tokenizer) -> LineType {
    while let Some(c) = tokenizer.content.get(tokenizer.position) {
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
    while let Some(c) = tokenizer.content.get(tokenizer.position) {
        if !c.is_whitespace() {
            break;
        }
        if *tokenizer.content.get(tokenizer.position).unwrap() == '\n' {
            break;
        }
        tokenizer.position += 1;
        tokenizer.line_position += 1;
    }
}

pub(super) fn consume_str<'a>(
    tokenizer: &'a mut Tokenizer,
    string: &'a String,
) -> Result<(), ParserError> {
    let chars = string.chars().collect::<Vec<char>>();
    for input in chars.iter() {
        let found = tokenizer.content.get(tokenizer.position);
        match found {
            Some(c) if c == input => {
                tokenizer.line_position += 1;
                tokenizer.position += 1;
                if *input == '\n' {
                    tokenizer.line_index += 1;
                    tokenizer.line_position = 0;
                }
            }
            None => return Err(ParserError::UnexpectedInput(None)),
            Some(_) => return Err(ParserError::UnexpectedInput(None)),
        }
    }
    Ok(())
}

pub(super) fn get_value_expression(tokenizer: &mut Tokenizer) -> String {
    let mut retval: Vec<char> = Vec::new();
    let mut open = 0;
    let mut close = 0;

    while let Some(c) = tokenizer.content.get(tokenizer.position) {
        if *c == '(' {
            open += 1;
        } else if *c == ')' {
            close += 1;
        } else if *c == '\n' {
            if open == close {
                break;
            }
            tokenizer.line_index += 1;
            tokenizer.line_position = 0;
        } else if *c == ';' {
            break;
        }
        retval.push(*c);
        tokenizer.position += 1;
    }
    retval.iter().collect()
}

pub(super) fn get_line(tokenizer: &mut Tokenizer) -> String {
    let mut retval: Vec<char> = Vec::new();
    while let Some(c) = tokenizer.content.get(tokenizer.position) {
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

/// Returns a word (whatever is between spaces)
pub(super) fn get_string(tokenizer: &mut Tokenizer) -> String {
    consume_whitespaces(tokenizer);
    let mut retval: Vec<char> = Vec::new();
    let mut quote = false;
    while let Some(c) = tokenizer.content.get(tokenizer.position) {
        if *c == '\n' {
            break;
        } else if *c == '"' {
            if retval.len() == 0 {
                quote = true
            } else if quote {
                tokenizer.position += 1;
                tokenizer.line_position += 1;
                break;
            }
        } else if (c.is_whitespace() | c.is_numeric() | (*c == '.') | (*c == '-') | (*c == ';'))
            & !quote
        {
            break;
        } else {
            retval.push(*c);
        }
        tokenizer.position += 1;
        tokenizer.line_position += 1;
    }
    consume_whitespaces(tokenizer);
    retval.iter().collect()
}
