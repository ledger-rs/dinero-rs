use std::path::PathBuf;

use crate::ledger;
use crate::ledger::{Account, HasName};
use crate::parser::Tokenizer;
use crate::Error;
use std::ops::Deref;

pub fn execute(path: PathBuf) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.parse()?;
    let mut ledgerelements = ledger::build_ledger(&items)?;
    let (_, _, prices) = ledger::populate_transactions(&items, &mut ledgerelements)?;

    for price in prices {
        println!("{}", price);
    }
    Ok(())
}
