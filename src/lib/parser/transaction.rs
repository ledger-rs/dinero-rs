use crate::parser::{Tokenizer, Item};
use crate::{ErrorType, Error};

pub(super) fn parse<'a>(tokenizer :&'a mut Tokenizer) -> Result<Item, Error> {
    // todo!("Transaction parsing not implemented");
    Err(tokenizer.error(ErrorType::UnexpectedInput))
}