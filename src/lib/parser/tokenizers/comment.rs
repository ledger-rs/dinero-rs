use crate::models::Comment;
use crate::parser::{chars, Tokenizer};

/// Parses a comment
///
/// # Examples
/// ```rust
/// # use dinero::parser::{Tokenizer};
/// # use dinero::models::Comment;
/// # let content = "; This is a comment\n".to_string();
/// let mut tokenizer = Tokenizer::from(content);
/// let parsed_ledger = tokenizer.tokenize().unwrap();
/// assert_eq!(parsed_ledger.len(), 1, "Should have parsed one item");
/// let comment =  parsed_ledger.comments.get(0).unwrap();
/// assert_eq!(comment.comment, "This is a comment".to_string());
/// ```
/// ```rust
/// # use dinero::parser::{Tokenizer};
/// # use dinero::models::Comment;
/// # let content = "; This is a comment\n".to_string();
/// let content = "; This is a comment\n\n\n; This is another comment\n\n\n".to_string();
/// let mut tokenizer = Tokenizer::from(content);
/// let items = tokenizer.tokenize().unwrap();
/// assert_eq!(items.len(), 2, "Should have parsed two comments")
/// ```
pub(crate) fn parse(tokenizer: &mut Tokenizer) -> Comment {
    tokenizer.position += 1;
    tokenizer.line_position += 1;
    chars::consume_whitespaces_and_lines(tokenizer);
    return Comment {
        comment: chars::get_line(tokenizer),
    };
}
