use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use colored::Colorize;

use crate::filter;
use crate::models::{Account, Balance, HasName};
use crate::parser::Tokenizer;
use crate::Error;
use std::ops::Deref;
use std::rc::Rc;

/// Balance report
pub fn execute(
    path: PathBuf,
    flat: bool,
    show_total: bool,
    depth: Option<usize>,
    query: Vec<String>,
    real: bool,
    no_balance_check: bool,
) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize()?;
    let ledger = items.to_ledger(no_balance_check)?;

    let mut balances: HashMap<Rc<Account>, Balance> = HashMap::new();

    for t in ledger.transactions.iter() {
        for p in t.postings_iter() {
            if !filter::filter(&query, t, p, real) {
                continue;
            }
            let mut cur_bal = balances
                .get(p.account.deref())
                .unwrap_or(&Balance::new())
                .to_owned();
            cur_bal = cur_bal + Balance::from(p.amount.as_ref().unwrap().clone());
            balances.insert(p.account.clone(), cur_bal.to_owned());
        }
    }

    // Remove the ones with balance zero

    // For printing this out, take into account whether it is a flat report or not
    // if it is not, the parent balances have to be updated
    let mut vec_balances: Vec<(&str, Balance)>;
    let mut accounts = HashSet::new();
    let mut new_balances = HashMap::new();
    let mut vec: Vec<String>;
    if !flat {
        // vec_balances = complete_balance(&balances);
        for (acc, bal) in balances.iter() {
            let mut pattern = "".to_string();
            for part in acc.get_name().split(":") {
                if pattern.len() > 0 {
                    pattern.push_str(":");
                }
                pattern.push_str(part);
                accounts.insert(pattern.clone());
            }
            new_balances.insert(acc.get_name(), bal.clone());
        }

        // Sort by depth
        vec = accounts.iter().map(|x| x.clone()).collect();
        vec.sort_by(|a, b| a.matches(":").count().cmp(&b.matches(":").count()));

        for account in vec.iter() {
            let balance = new_balances
                .iter()
                .filter(|x| x.0.starts_with(account.as_str()))
                .fold(Balance::new(), |acc, new| acc + new.1.clone());
            new_balances.insert(account.as_str().clone(), balance);
        }
        vec_balances = new_balances
            .iter()
            .filter(|x| !x.1.is_zero())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect()
    } else {
        vec_balances = balances
            .iter()
            .filter(|x| !x.1.is_zero())
            .map(|x| (x.0.get_name(), x.1.clone()))
            .collect();
    }

    // Print the balances by account

    vec_balances.sort_by(|a, b| a.0.cmp(b.0));
    for (account, bal) in vec_balances.iter() {
        if let Some(depth) = depth {
            if account.split(":").count() > depth {
                continue;
            }
        }

        let mut first = true;

        for (_, money) in bal.balance.iter() {
            if !first {
                println!();
            }
            first = false;
            match money.is_negative() {
                true => print!("{:>20}", format!("{}", money).red()),
                false => print!("{:>20}", format!("{}", money)),
            }
        }
        if flat {
            println!("  {}", account.blue());
        } else {
            let n = account.split(":").count();
            let text = account.split(":").last().unwrap();
            for _ in 0..n {
                print!("  ");
            }
            println!("{}", text.blue());
        }
    }

    // Print the total
    if show_total {
        // Calculate it
        let total_balance = balances
            .iter()
            .fold(Balance::new(), |acc, x| acc + x.1.to_owned());
        print!("{}", "--------------------");

        if total_balance.is_zero() {
            print!("\n{:>20}", "0");
        } else {
            for (_, money) in total_balance.balance.iter() {
                match money.is_negative() {
                    true => print!("\n{:>20}", format!("{}", money).red()),
                    false => print!("\n{:>20}", format!("{}", money)),
                }
            }
        }
        println!();
    }

    // We're done :)
    Ok(())
}
