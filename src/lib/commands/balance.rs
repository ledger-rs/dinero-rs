use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use colored::Colorize;

use crate::ledger;
use crate::ledger::{Account, Balance, HasName, Money, Price};
use crate::parser::Tokenizer;
use crate::Error;

/// Balance report
pub fn execute(
    path: PathBuf,
    flat: bool,
    show_total: bool,
    depth: Option<usize>,
) -> Result<(), Error> {
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.parse()?;
    let mut ledgerelements = ledger::build_ledger(&items)?;
    let (mut transactions, mut balances, mut prices) = ledger::populate_transactions(&items, &mut ledgerelements)?;
    // Balance the transactions
    for t in transactions.iter_mut() {
        let date = t.date.unwrap().clone();
        let balance = t.balance(&mut balances).unwrap();
        if balance.len() == 2 {
            let vec = balance.iter()
                .map(|(_, x)| x.abs())
                .collect::<Vec<Money>>();
            prices.push(Price {
                date: date,
                commodity: vec[0],
                price: vec[1],
            });
        }
    }
    let mut balances: HashMap<&Account, Balance> = HashMap::new();

    for t in transactions.iter() {
        for p in t.postings.iter() {
            let mut cur_bal = balances
                .get(p.account)
                .unwrap_or(&Balance::new())
                .to_owned();
            cur_bal = cur_bal + p.amount.unwrap().into();
            balances.insert(p.account, cur_bal.to_owned());
        }
    }

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
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect()
    } else {
        vec_balances = balances
            .iter()
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
            print!("{:>20}", format!("{}", money));
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
                print!("\n{:>20}", format!("{}", money));
            }
        }
        println!();
    }

    // We're done :)
    Ok(())
}
