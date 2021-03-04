use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use colored::Colorize;

use crate::models::{conversion, Account, Balance, Currency, HasName, Money};
use crate::parser::value_expr::build_root_node_from_expression;
use crate::parser::Tokenizer;
use crate::Error;
use crate::{filter, CommonOpts};
use chrono::Utc;
use num::rational::BigRational;
use std::ops::Deref;
use std::rc::Rc;

/// Balance report
pub fn execute(options: &CommonOpts, flat: bool, show_total: bool) -> Result<(), Error> {
    // Get the options
    let path: PathBuf = options.input_file.clone();
    let depth = options.depth;
    let no_balance_check = options.no_balance_check;
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize()?;
    let mut ledger = items.to_ledger(no_balance_check)?;

    let mut balances: HashMap<Rc<Account>, Balance> = HashMap::new();

    // Build a cache of abstract value trees, it takes time to parse expressions, so better do it only once
    let mut regexes = HashMap::new();
    let query = filter::preprocess_query(&options.query);
    let node = if query.len() > 2 {
        Some(build_root_node_from_expression(
            query.as_str(),
            &mut regexes,
        ))
    } else {
        None
    };

    for t in ledger.transactions.iter() {
        for p in t.postings.borrow().iter() {
            if !filter::filter(&options, &node, t, p, &mut ledger.commodities)? {
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

    // For printing this out, take into account whether it is a flat report or not
    // if it is not, the parent balances have to be updated
    let mut vec_balances: Vec<(&str, Balance)> = vec![];
    let mut temp: Vec<(String, Balance)>;
    let mut accounts = HashSet::new();
    let mut new_balances = HashMap::new();
    let mut vec: Vec<String>;
    if !flat {
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
            let mut prefix = account.clone();
            prefix.push(':'); // It is important to add this see [issue #8](https://github.com/frosklis/dinero-rs/issues/8)
            let balance = new_balances
                .iter()
                .filter(|x| (x.0 == account) | x.0.starts_with(&prefix))
                .fold(Balance::new(), |acc, new| acc + new.1.clone());
            new_balances.insert(account.as_str().clone(), balance);
        }
        vec_balances = new_balances
            .iter()
            .filter(|x| !x.1.is_zero())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect()
    } else {
        match depth {
            Some(depth) => {
                temp = balances
                    .iter()
                    .filter(|x| !x.1.is_zero())
                    .map(|x| {
                        (
                            x.0.get_name()
                                .split(":")
                                .collect::<Vec<&str>>()
                                .iter()
                                .map(|x| x.to_string())
                                .take(depth)
                                .collect::<Vec<String>>()
                                .join(":"),
                            x.1.clone(),
                        )
                    })
                    .collect::<Vec<(String, Balance)>>();
                temp.sort_by(|a, b| a.0.cmp(&b.0));
                let mut account = String::new();
                for (acc, value) in temp.iter() {
                    if acc.to_string() != account {
                        vec_balances.push((acc.as_str(), value.clone()));
                    } else {
                        let n = vec_balances.len();
                        vec_balances[n - 1] =
                            (&acc.as_str(), vec_balances[n - 1].clone().1 + value.clone());
                    }

                    account = acc.to_string();
                }
            }
            None => {
                vec_balances = balances
                    .iter()
                    .filter(|x| !x.1.is_zero())
                    .map(|x| (x.0.get_name(), x.1.clone()))
                    .collect()
            }
        }
    }

    // Print the balances by account
    let mut multipliers = HashMap::new();
    if let Some(currency_string) = &options.exchange {
        if let Ok(currency) = ledger.commodities.get(currency_string) {
            multipliers = conversion(
                currency.clone(),
                Utc::now().naive_local().date(),
                &ledger.prices,
            );
            let mut updated_balances = Vec::new();
            for (acc, balance) in vec_balances.iter() {
                updated_balances.push((
                    acc.clone(),
                    convert_balance(balance, &multipliers, currency),
                ));
            }
            vec_balances = updated_balances;
        }
    }

    vec_balances.sort_by(|a, b| a.0.cmp(b.0));
    let num_bal = vec_balances.len();
    let mut index = 0;
    let mut showed_balances = 0;
    while index < num_bal {
        let (account, bal) = &vec_balances[index];
        if let Some(depth) = depth {
            if account.split(":").count() > depth {
                index += 1;
                continue;
            }
        }
        if bal.is_zero() {
            index += 1;
            continue;
        }
        showed_balances += 1;

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
        if first {
            // This means the balance was empty
            print!("{:>20}", "0");
        }
        if flat {
            println!("  {}", account.blue());
        } else {
            let mut n = account.split(":").count();
            for _ in 0..n {
                print!("  ");
            }
            // start by getting the account name
            let mut text = account.split(":").last().unwrap().to_string();
            // This is where it gets tricky, we need to collapse while we can
            let mut collapse = true;
            loop {
                if (index + 1) >= num_bal {
                    break;
                }
                if vec_balances[index + 1].0.split(":").count() != (n + 1) {
                    break;
                }
                for j in (index + 2)..num_bal {
                    let name = vec_balances[j].0.clone();
                    if !name.starts_with(account) {
                        break;
                    }
                    let this_depth = name.split(":").count();
                    if this_depth == n + 1 {
                        collapse = false;
                        break;
                    }
                }
                if collapse {
                    text.push(':');
                    text.push_str(&vec_balances[index + 1].0.split(":").last().unwrap());
                    n = n + 1;
                    index = index + 1;
                } else {
                    break;
                }
            }
            println!("{}", text.blue());
        }
        index += 1;
    }

    // Print the total
    if show_total & (showed_balances > 1) {
        // Calculate it
        let mut total_balance = balances
            .iter()
            .fold(Balance::new(), |acc, x| acc + x.1.to_owned());
        print!("{}", "--------------------");
        if multipliers.len() > 0 {
            total_balance = convert_balance(
                &total_balance,
                &multipliers,
                ledger
                    .commodities
                    .get(options.exchange.as_ref().unwrap().as_str())
                    .unwrap(),
            );
        }
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

fn convert_balance(
    balance: &Balance,
    multipliers: &HashMap<Rc<Currency>, BigRational>,
    currency: &Currency,
) -> Balance {
    let mut new_balance = Balance::new();
    for (curr, money) in balance.iter() {
        if let Some(mult) = multipliers.get(curr.clone().unwrap().as_ref()) {
            new_balance = new_balance
                + Money::Money {
                    amount: money.get_amount() * mult.clone(),
                    currency: Rc::new(currency.clone()),
                }
                .into()
        } else {
            new_balance = new_balance + money.clone().into();
        }
    }
    new_balance
}
