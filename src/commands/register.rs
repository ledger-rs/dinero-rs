use crate::models::{Balance, Money};
use crate::parser::expressions::build_root_node_from_expression;
use crate::parser::expressions::format_expression_to_string;
use crate::parser::Tokenizer;
use crate::Error;
use crate::{filter, CommonOpts};
use std::collections::HashMap;

/// Register report
pub fn execute(options: &CommonOpts) -> Result<(), Error> {
    // Get options from options
    let path = options.input_file.clone();
    // Now work
    let mut tokenizer: Tokenizer = Tokenizer::from(&path);
    let items = tokenizer.tokenize();
    let mut ledger = items.to_ledger(options)?;

    let mut balance = Balance::new();

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
            let first = counter == 1;

            balance = balance + Balance::from(p.amount.as_ref().unwrap().clone());
            if balance.is_zero() {
                balance = Balance::from(Money::Zero);
            }

            print!(
                "{}",
                format_expression_to_string(
                    options.register_format.as_str(),
                    p,
                    t,
                    options,
                    first,
                    &mut ledger.commodities,
                    &mut regexes,
                )
            );
        }
    }

    // We're done :)
    Ok(())
}
