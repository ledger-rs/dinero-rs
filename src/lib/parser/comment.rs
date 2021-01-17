use crate::parser::{Tokenizer, chars};
use crate::ledger::Comment;


/// Parses a comment
///
/// # Examples
/// ```rust
/// # use dinero::parser::{Tokenizer,Item};
/// # use dinero::ledger::Comment;
/// # let content = "; This is a comment\n".to_string();
/// let mut tokenizer = Tokenizer::from(content);
/// let items = tokenizer.parse().unwrap();
/// assert_eq!(items.len(), 1, "Should have parsed one item");
/// let comment = match items.get(0).unwrap() {
///     Item::Comment(Comment{comment}) => comment,
///     _ => panic!("It should be a comment")
/// };
/// assert_eq!(*comment, "This is a comment".to_string());
/// ```
/// ```rust
/// # use dinero::parser::{Tokenizer,Item};
/// # use dinero::ledger::Comment;
/// # let content = "; This is a comment\n".to_string();
/// let content = "; This is a comment\n\n\n; This is another comment\n\n\n".to_string();
/// let mut tokenizer = Tokenizer::from(content);
/// let items = tokenizer.parse().unwrap();
/// assert_eq!(items.len(), 2, "Should have parsed two comments")
/// ```
pub(super) fn parse(tokenizer :&mut Tokenizer) -> Comment {
    tokenizer.position += 1;
    tokenizer.line_position += 1;
    chars::consume_whitespaces_and_lines(tokenizer);
    return Comment {comment : chars::get_line(tokenizer)};
}