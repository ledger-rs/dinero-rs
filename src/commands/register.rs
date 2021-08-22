use crate::models::{conversion, HasName, Ledger, Posting, PostingType};
use crate::models::{Balance, Money};
use crate::parser::value_expr::build_root_node_from_expression;
use crate::Error;
use crate::{filter, CommonOpts};
use colored::Colorize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use terminal_size::{terminal_size, Width};

/// Register report
pub fn execute(options: &CommonOpts, maybe_ledger: Option<Ledger>) -> Result<(), Error> {
    // Get options from options
    let _no_balance_check: bool = options.no_balance_check;
    // Now work
    let ledger = match maybe_ledger {
        Some(ledger) => ledger,
        None => Ledger::try_from(options)?,
    };

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
        let mut postings_vec = t
            .postings
            .borrow()
            .iter()
            .filter(|p| {
                filter::filter(&options, &node, t, p.to_owned(), &ledger.commodities).unwrap()
            })
            .map(|p| p.clone())
            .collect::<Vec<Posting>>();

        // If the exchange option is active, change the amount of every posting to the desired currency. The balance will follow.
        if let Some(currency_string) = &options.exchange {
            if let Ok(currency) = ledger.commodities.get(currency_string) {
                for index in 0..postings_vec.len() {
                    let p = &postings_vec[index];
                    let multipliers = conversion(currency.clone(), p.date, &ledger.prices);
                    if let Some(mult) = multipliers
                        .get(p.amount.as_ref().unwrap().get_commodity().unwrap().as_ref())
                    {
                        let new_amount = Money::Money {
                            amount: p.amount.as_ref().unwrap().get_amount() * mult.clone(),
                            currency: Rc::new(currency.as_ref().clone()),
                        };
                        let mut new_posting = p.clone();
                        new_posting.set_amount(new_amount);
                        postings_vec[index] = new_posting;
                    }
                }
            }
        }

        if options.collapse && (postings_vec.len() > 0) {
            // Sort ...
            postings_vec.sort_by(|a, b| {
                (&format!(
                    "{}{}",
                    a.account.get_name(),
                    a.amount
                        .as_ref()
                        .unwrap()
                        .get_commodity()
                        .unwrap()
                        .get_name()
                ))
                    .partial_cmp(&format!(
                        "{}{}",
                        b.account.get_name(),
                        b.amount
                            .as_ref()
                            .unwrap()
                            .get_commodity()
                            .unwrap()
                            .get_name()
                    ))
                    .unwrap()
            });

            // ... and collapse
            let mut collapsed = vec![postings_vec[0].clone()];
            let mut ind = 0;
            for i in 1..postings_vec.len() {
                if (postings_vec[i].account == collapsed[ind].account)
                    & (postings_vec[i].amount.as_ref().unwrap().get_commodity()
                        == collapsed[ind].amount.as_ref().unwrap().get_commodity())
                {
                    let mut new_posting = postings_vec[i].clone();
                    new_posting.set_amount(
                        (new_posting.amount.clone().unwrap()
                            + collapsed[ind].amount.clone().unwrap())
                        .to_money()
                        .unwrap(),
                    );
                    collapsed[ind] = new_posting;
                } else {
                    ind += 1;
                    collapsed.push(postings_vec[i].clone())
                }
            }
            postings_vec = collapsed;
        }
        for p in postings_vec.iter() {
            counter += 1;
            if counter == 1 {
                match t.get_payee(&ledger.payees) {
                    Some(payee) => print!(
                        "{:w1$}{:width$}",
                        format!("{}", t.date.unwrap()).green(),
                        clip(&format!("{} ", payee), w_description),
                        width = w_description,
                        w1 = w_date
                    ),
                    None => print!(
                        "{:w1$}{:width$}",
                        format!("{}", t.date.unwrap()).green(),
                        clip(&format!("{} ", ""), w_description),
                        width = w_description,
                        w1 = w_date
                    ),
                }
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
