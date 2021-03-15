use crate::models::PostingType;
use crate::models::{Balance, Money};
use crate::parser::value_expr::build_root_node_from_expression;
use crate::parser::Tokenizer;
use crate::Error;
use crate::{filter, CommonOpts};
use colored::Colorize;
use std::collections::HashMap;
use terminal_size::{terminal_size, Width};

/// Register report
pub fn execute(options: &CommonOpts) -> Result<(), Error> {
    // Get options from options
    let path = options.input_file.clone();
    let no_balance_check: bool = options.no_balance_check;
    // Now work
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize();
    let mut ledger = items.to_ledger(options)?;

    let mut balance = Balance::new();

    let size = terminal_size();
    let mut width: usize = 80;
    if let Some((Width(w), _)) = size {
        width = w as usize;
    }
    let w_date: usize = 11;
    let mut w_amount: usize = 21;
    let mut w_balance: usize = 21;
    let w_description: usize = 42;
    let w_account: usize = if w_date + w_description + w_amount + w_balance >= width {
        w_amount = 17;
        w_balance = 17;
        34
    } else {
        width - w_date - w_description - w_amount - w_balance
    };

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
        let mut counter = 0;
        for p in t.postings.borrow().iter() {
            if !filter::filter(&options, &node, t, p, &mut ledger.commodities)? {
                continue;
            }
            counter += 1;
            if counter == 1 {
                print!(
                    "{:w1$}{:width$}",
                    format!("{}", t.date.unwrap()),
                    clip(
                        &format!("{} ", t.get_payee(&mut ledger.payees)),
                        w_description
                    ),
                    width = w_description,
                    w1 = w_date
                );
            }
            if counter > 1 {
                print!("{:width$}", "", width = w_description + 11);
            }
            balance = balance + Balance::from(p.amount.as_ref().unwrap().clone());
            if balance.is_zero() {
                balance = Balance::from(Money::Zero);
            }
            match p.kind {
                PostingType::Real => print!(
                    "{:width$}",
                    format!("{}", p.account).blue(),
                    width = w_account
                ),
                PostingType::Virtual => print!(
                    "{:width$}",
                    format!("({})", p.account).blue(),
                    width = w_account
                ),
                PostingType::VirtualMustBalance => print!(
                    "{:width$}",
                    format!("[{}]", p.account).blue(),
                    width = w_account
                ),
            }

            match p.amount.as_ref().unwrap().is_negative() {
                false => print!(
                    "{:>width$}",
                    format!("{}", p.amount.as_ref().unwrap()),
                    width = w_amount
                ),
                true => print!(
                    "{:>width$}",
                    format!("{}", p.amount.as_ref().unwrap()).red(),
                    width = w_amount
                ),
            }
            let mut more_than_one_line: bool = false;
            for (_, money) in balance.iter() {
                if more_than_one_line {
                    print!(
                        "{:width$}",
                        "",
                        width = w_date + w_description + w_account + w_amount
                    );
                }
                more_than_one_line = true;
                match money.is_positive() {
                    true => println!("{:>width$}", format!("{}", money), width = w_balance),
                    false => println!("{:>width$}", format!("{}", money).red(), width = w_balance),
                }
            }
        }
    }

    // We're done :)
    Ok(())
}

fn clip(string: &String, width: usize) -> String {
    if string.len() < width - 3 {
        string.to_string()
    } else {
        let mut ret = String::new();
        for (i, c) in string.chars().enumerate() {
            if i >= width - 3 {
                break;
            }
            ret.push(c);
        }

        format!("{}..", ret)
    }
}
