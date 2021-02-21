use crate::models::{Currency, HasName, Posting, PostingType, Transaction};
use crate::parser::value_expr::{eval_expression, EvalResult};
use crate::{CommonOpts, Error, List};
use colored::Colorize;

pub fn filter(
    options: &CommonOpts,
    transaction: &Transaction<Posting>,
    posting: &Posting,
    commodities: &mut List<Currency>,
) -> Result<bool, Error> {
    // Get what's needed
    let predicate = preprocess_query(&options.query);
    let real = options.real;

    // Check for real postings
    if real {
        if let PostingType::Real = posting.kind {
        } else {
            return Ok(false);
        }
    }

    // Check for dates at the transaction level
    // todo should do this at the posting level
    if let Some(date) = options.end {
        if transaction.date.unwrap() >= date {
            return Ok(false);
        }
    }
    if let Some(date) = options.begin {
        if transaction.date.unwrap() < date {
            return Ok(false);
        }
    }

    filter_predicate(predicate.as_str(), posting, transaction, commodities)
}

pub fn filter_predicate_old(predicate: &Vec<String>, posting: &Posting) -> bool {
    let name = posting.account.get_name().to_lowercase();
    if predicate.len() == 0 {
        return true;
    }
    for pred in predicate {
        let p = pred.trim();
        if p.starts_with("%") {
            // look in the posting tags
            for tag in posting.tags.iter() {
                match tag.name.to_lowercase().find(&p.to_lowercase()[1..]) {
                    None => continue,
                    Some(_) => return true,
                }
            }
        } else {
            match name.find(&p.to_lowercase()) {
                None => continue,
                Some(_) => return true,
            }
        }
    }
    false
}

pub fn filter_predicate(
    predicate: &str,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
) -> Result<bool, Error> {
    if predicate.len() == 0 {
        return Ok(true);
    }
    let result = eval_expression(predicate, posting, transaction, commodities);
    match result {
        EvalResult::Boolean(b) => Ok(b),
        _ => Err(Error {
            message: vec![predicate.red().bold(), "should return a boolean".normal()],
        }),
    }
}

/// Create search expression from Strings
///
/// The command line arguments provide syntactic sugar which save time when querying the journal.
/// This expands it to an actual query
///
/// # Examples
/// ```rust
/// # use dinero::filter::preprocess_query;
/// let params:Vec<String> = vec!["@payee", "savings" , "and", "checking", "and", "expr", "/aeiou/"].iter().map(|x| x.to_string()).collect();
/// let processed = preprocess_query(&params);
/// assert_eq!(processed, "(@payee) or (account =~ /(?i)savings/) and (account =~ /(?i)checking/) or (/aeiou/)")
/// ```
pub fn preprocess_query(query: &Vec<String>) -> String {
    let mut expression = String::new();
    let mut and = false;
    let mut first = true;
    let mut expr = false;
    for raw_term in query.iter() {
        let term = raw_term.trim();
        if term.len() == 0 {
            continue;
        }
        if (term == "and") | (term == "expr") {
            and = term == "and";
            expr = term == "expr";
            continue;
        }
        let join_term = if !first {
            if and {
                " and ("
            } else {
                " or ("
            }
        } else {
            "("
        };
        expression.push_str(join_term);
        if expr {
            expression.push_str(term);
        } else if let Some(c) = term.chars().next() {
            match c {
                '@' => expression.push_str(term),
                '%' => {
                    expression.push_str("has_tag(/(?i)"); // case insensitive
                    expression.push_str(&term.to_string()[1..]);
                    expression.push_str("/)")
                }
                _ => {
                    expression.push_str("account =~ /(?i)"); // case insensitive
                    expression.push_str(term);
                    expression.push_str("/")
                }
            }
        }
        expression.push_str(")");
        and = false;
        expr = false;
        first = false;
    }
    expression
}
