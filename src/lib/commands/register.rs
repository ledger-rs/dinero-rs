use std::path::PathBuf;

use colored::Colorize;

use crate::filter;
use crate::models::Balance;
use crate::parser::Tokenizer;
use crate::Error;

/// Register report
pub fn execute(
    path: PathBuf,
    query: Vec<String>,
    real: bool,
    no_balance_check: bool,
) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize()?;
    let ledger = items.to_ledger(no_balance_check)?;

    let mut balance = Balance::new();

    for t in ledger.transactions.iter() {
        let mut counter = 0;
        for p in t.postings.iter() {
            if !filter::filter(&query, t, p, real) {
                continue;
            }
            counter += 1;
            if counter == 1 {
                print!("{:10} {:20}", t.date.unwrap(), t.description);
            }
            if counter > 1 {
                print!("{:31}", "");
            }
            balance = balance + Balance::from(p.amount.as_ref().unwrap().clone());
            print!("{:30}", format!("{}", p.account).blue());

            match p.amount.as_ref().unwrap().is_negative() {
                false => print!("{:>16}", format!("{}", p.amount.as_ref().unwrap())),
                true => print!("{:>16}", format!("{}", p.amount.as_ref().unwrap()).red()),
            }
            for (i, (_, money)) in balance.iter().enumerate() {
                if i > 0 {
                    print!("{:77}", "")
                }
                match money.is_negative() {
                    false => println!("{:>20}", format!("{}", money)),
                    true => println!("{:>20}", format!("{}", money).red()),
                }
            }
        }
    }

    // We're done :)
    Ok(())
}
