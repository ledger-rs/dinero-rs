use prettytable::format;
use prettytable::Table;

use crate::commands::balance::convert_balance;
use crate::models::{conversion, Balance, HasName, Ledger, Money};
use crate::parser::utils::rational2float;
use crate::parser::value_expr::build_root_node_from_expression;
use crate::Error;
use crate::{filter, CommonOpts};
use chrono::{Datelike, Duration, NaiveDate};
use num::{BigInt, BigRational, Zero};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

/// ROI (return on investment) report
pub fn execute(
    options: &CommonOpts,
    maybe_ledger: Option<Ledger>,
    cash_flows_query: Vec<String>,
    assets_value_query: Vec<String>,
) -> Result<(), Error> {
    let mut ledger = match maybe_ledger {
        Some(ledger) => ledger,
        None => Ledger::try_from(options)?,
    };

    // dbg!(&cash_flows_query);
    // dbg!(&assets_value_query);

    // TODO exit gracefully
    assert!(
        cash_flows_query.len() > 0,
        "cash flows query has to be provided"
    );
    assert!(assets_value_query.len() > 0, "assets value query");

    // Prepare the nodes for filtering
    let mut regexes = HashMap::new();
    let mut query = filter::preprocess_query(&cash_flows_query);
    let cash_flows_node = if query.len() > 2 {
        Some(build_root_node_from_expression(
            query.as_str(),
            &mut regexes,
        ))
    } else {
        None
    };
    query = filter::preprocess_query(&assets_value_query);
    let assets_value_node = if query.len() > 2 {
        Some(build_root_node_from_expression(
            query.as_str(),
            &mut regexes,
        ))
    } else {
        None
    };

    // Get a currency
    let currency = &ledger
        .commodities
        .get(&options.exchange.as_ref().unwrap())?
        .clone();

    let mut first_date = options.begin.clone();
    let mut last_date = options.begin.clone();
    if let Some(date) = first_date {
        first_date = Some(month_beginning(date));
    }

    let mut periods: Vec<Period> = vec![];

    for t in ledger.transactions.iter() {
        // cash_flows
        for p in t.postings.borrow().iter() {
            if !filter::filter(&options, &cash_flows_node, t, p, &mut ledger.commodities)? {
                continue;
            }
            let index = get_period_index(p.date, &mut periods);
            let mut period = &mut periods[index];
            if p.amount
                .as_ref()
                .unwrap()
                .get_commodity()
                .unwrap()
                .get_name()
                == currency.get_name()
            {
                period.add_cash(p.amount.as_ref().unwrap().to_owned());
            } else {
                let multipliers = conversion(currency.clone(), p.date, &ledger.prices);
                let mult = multipliers
                    .get(p.amount.as_ref().unwrap().get_commodity().unwrap().as_ref())
                    .unwrap();
                let new_amount = Money::Money {
                    amount: p.amount.as_ref().unwrap().get_amount() * mult.clone(),
                    currency: Rc::new(currency.as_ref().clone()),
                };
                period.add_cash(new_amount);
            }
        } // cash flows

        // balances
        for p in t.postings.borrow().iter() {
            if !filter::filter(&options, &assets_value_node, t, p, &mut ledger.commodities)? {
                continue;
            }
            let index = get_period_index(p.date, &mut periods);
            let mut period = &mut periods[index];

            period.final_balance =
                period.final_balance.clone() + Balance::from(p.amount.as_ref().unwrap().to_owned());
        } // balances
    }

    let mut last_date = None;
    let mut insertions: Vec<Period> = vec![];
    periods.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    for p in periods.iter_mut() {
        if last_date.is_none() {
            last_date = Some(p.end.clone());
            continue;
        }
        // Because the gap may be more than one month, we need a loop
        'inner: loop {
            let expected = last_date.unwrap() + Duration::days(1);
            last_date = Some(month_ending(expected.clone()));
            if expected == p.start {
                break 'inner;
            }
            let new_period = Period::from_date(expected);
            insertions.push(new_period);
        }
    }

    periods.append(&mut insertions);
    periods.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    let mut prev_final_balance = Balance::new();
    let mut prev_final_money = None;

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row!["Period", r->"Cash flow", r->"Final balance", r->"twr"]);
    for (i, p) in periods.iter_mut().enumerate() {
        if i > 0 {
            p.initial_balance = prev_final_balance;
            p.initial_money = prev_final_money;
        }
        p.final_balance = p.final_balance.clone() + p.initial_balance.clone();

        if p.final_money.is_none() {
            let multipliers = conversion(currency.clone(), p.end, &ledger.prices);
            p.final_money = Some(
                convert_balance(&p.final_balance, &multipliers, currency)
                    .to_money()
                    .unwrap(),
            );
        }
        if p.initial_money.is_none() {
            let multipliers = conversion(currency.clone(), p.start, &ledger.prices);
            p.initial_money = Some(
                convert_balance(&p.initial_balance, &multipliers, currency)
                    .to_money()
                    .unwrap(),
            );
        }

        table.add_row(row![
            format!("{}", p.start)[0..7],
            r->format!("{}", p.cash_flow),
            r->format!("{}", p.final_money.as_ref().unwrap()),
            r->format!("{}%", rational2float(&(&p.twr() * BigInt::from(100)), 2)),
        ]);

        prev_final_balance = p.final_balance.clone();
        prev_final_money = p.final_money.clone();
    }
    // Print the table to stdout
    table.printstd();
    Ok(())
}

fn get_period_index(date: NaiveDate, periods: &mut Vec<Period>) -> usize {
    let begin = month_beginning(date);
    for (i, period) in periods.iter().enumerate() {
        if period.start == begin {
            return i;
        }
    }
    let period = Period::from_date(date);
    periods.insert(0, period);
    0
}
#[derive(Debug)]
struct Period {
    start: NaiveDate,
    end: NaiveDate,
    initial_balance: Balance,
    final_balance: Balance,
    cash_flow: Money,
    initial_money: Option<Money>,
    final_money: Option<Money>,
}
impl Period {
    fn from_date(d: NaiveDate) -> Self {
        Period {
            start: month_beginning(d),
            end: month_ending(d),
            initial_balance: Balance::new(),
            final_balance: Balance::new(),
            cash_flow: Money::Zero,
            initial_money: None,
            final_money: None,
        }
    }
    fn add_cash(&mut self, money: Money) {
        let bal = self.cash_flow.clone() + money;
        self.cash_flow = bal.to_money().unwrap();
    }
    fn twr(&self) -> BigRational {
        let end = self.final_money.as_ref().unwrap().get_amount();
        let initial = self.initial_money.as_ref().unwrap().get_amount();
        if initial.is_zero() {
            return initial;
        }
        let flow = self.cash_flow.get_amount();
        // (end - initial + flow) / initial
        let twr = (end + flow) / initial - BigInt::from(1);
        twr
    }
}

/// Returns the first day of the month
fn month_beginning(d: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd(d.year(), d.month(), 1)
}

/// Returns the last day of the month
fn month_ending(d: NaiveDate) -> NaiveDate {
    let month: u32;
    match d.month() {
        12 => NaiveDate::from_ymd(d.year(), 12, 31),
        other => {
            month = other + 1;
            NaiveDate::from_ymd(d.year(), month, 1) - Duration::days(1)
        }
    }
}
