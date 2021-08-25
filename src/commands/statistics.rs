use std::convert::TryFrom;

use crate::models::Ledger;
use crate::{ CommonOpts};

/// Statistics command
///
/// Prints summary statistics from the ledger
pub fn execute(
    options: &CommonOpts,
    maybe_ledger: Option<Ledger>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ledger = match maybe_ledger {
        Some(ledger) => ledger,
        None => Ledger::try_from(options)?,
    };

    // Number of transactions
    let mut num_postings = 0;
    for t in ledger.transactions.iter() {
        num_postings += t.postings.borrow().iter().count();
    }

    let num_files = ledger.files.len();
    if num_files > 0 {
        println!("Number of files processed: {}", num_files);
        for file in ledger.files.iter() {
            let path_str = file.clone().into_os_string().into_string().unwrap();

            println!("\t{}", &path_str);
        }
    }

    let first_transaction_date = &ledger.transactions.get(0).unwrap().date.unwrap();
    let last_transaction_date = &ledger
        .transactions
        .iter()
        .rev()
        .next()
        .unwrap()
        .date
        .unwrap();
    let num_days = 1 + last_transaction_date
        .signed_duration_since(*first_transaction_date)
        .num_days();
    // Print the stats
    println!("{} postings", num_postings);
    println!("{} transactions", &ledger.transactions.len());

    println!(
        "First transaction: {}",
        first_transaction_date.format(&options.date_format)
    );
    println!(
        "Last transaction: {}",
        last_transaction_date.format(&options.date_format)
    );
    println!("{} days between first and last transaction", num_days);
    println!(
        "{:.2} transactions per day (average)",
        (ledger.transactions.len() as f64) / (num_days as f64)
    );
    println!(
        "{:.2} postings per day (average)",
        (num_postings as f64) / (num_days as f64)
    );

    println!("{} price entries", &ledger.prices.len());
    println!("{} different accounts", &ledger.accounts.len());
    println!("{} different payees", &ledger.payees.len());
    println!("{} different commodities", &ledger.commodities.len());

    Ok(())
}
