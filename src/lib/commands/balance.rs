use crate::parser::{Tokenizer};
use crate::Error;
use std::path::PathBuf;
use colored::Colorize;
use crate::ledger;
use crate::ledger::{LedgerElements, Account, Balance, HasName};
use std::collections::HashMap;

pub fn execute(file: &str) -> Result<(), Error> {
    let path = PathBuf::from(file);
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let mut items = tokenizer.parse()?;
    let mut ledgerelements = ledger::build_ledger(&items)?;
    let (mut transactions, mut balances) = ledger::populate_transactions(&items, &ledgerelements)?;
    transactions.iter_mut().for_each(|t| t.balance(&mut balances).unwrap());
    let mut balances: HashMap<&Account, Balance> = HashMap::new();

    for t in transactions.iter() {
        for p in t.postings.iter() {
            let mut cur_bal = balances.get(p.account).unwrap_or(&Balance::new()).to_owned();
            cur_bal = cur_bal + p.amount.unwrap().into();
            balances.insert(p.account, cur_bal.to_owned());
        }
    }

    let mut total_balance = Balance::new();
    for (account, bal) in balances.iter() {
        for (_, money) in bal.balance.iter() {
            print!("\n{:>20}", format!("{}", money));
        }
        total_balance = total_balance + bal.to_owned();
        print!("  {}", account.get_name().blue());
    }
    print!("\n--------------------");

    
    for (_, money) in total_balance.balance.iter() {
        print!("\n{:>20}", format!("{}", money));
    }
    println!("");
    Ok(())
}