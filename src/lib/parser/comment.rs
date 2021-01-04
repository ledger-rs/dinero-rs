use crate::parser::{Tokenizer, Item, chars};
use crate::ledger::JournalComment;

pub(super) fn parse(tokenizer :&mut Tokenizer) -> Item {
    tokenizer.position += 1;
    tokenizer.line_position += 1;
    chars::consume_whitespaces(tokenizer);
    return Item::Comment(JournalComment{comment : chars::consume_line(tokenizer)});
}