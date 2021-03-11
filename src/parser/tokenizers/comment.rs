use crate::models::Comment;
use crate::parser::Tokenizer;

/// Parses a comment
///
/// # Examples
/// ```rust
/// # use dinero::parser::{Tokenizer};
/// # use dinero::models::Comment;
/// # let content = "; This is a comment\n; This is another comment\n".to_string();
/// let mut tokenizer = Tokenizer::from(content);
/// let parsed_ledger = tokenizer.tokenize().unwrap();
/// assert_eq!(parsed_ledger.len(), 2, "Should have parsed two items");
/// let comment_1 = parsed_ledger.comments.get(0).unwrap();
/// assert_eq!(comment_1.comment, "This is a comment".to_string());
/// let comment_2 = parsed_ledger.comments.get(1).unwrap();
/// assert_eq!(comment_2.comment, "This is another comment".to_string());
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
    return Comment {
        comment: unimplemented!("comment"),
    };
}
