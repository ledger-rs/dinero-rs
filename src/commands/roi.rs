use num::ToPrimitive;
use prettytable::format;
use prettytable::Table;

use crate::app::PeriodGroup;
use crate::commands::balance::convert_balance;
use crate::models::{conversion, Balance, Ledger, Money};
use crate::parser::value_expr::build_root_node_from_expression;
use crate::Error;
use crate::{filter, CommonOpts};
use chrono::{Datelike, Duration, NaiveDate};
use num::{BigInt, BigRational, Zero};
use std::collections::HashMap;
use std::convert::TryFrom;

/// ROI (return on investment) report
pub fn execute(
    options: &CommonOpts,
    maybe_ledger: Option<Ledger>,
    cash_flows_query: Vec<String>,
    assets_value_query: Vec<String>,
    frequency: Frequency,
) -> Result<(), Error> {
    let mut ledger = match maybe_ledger {
        Some(ledger) => ledger,
        None => Ledger::try_from(options)?,
    };

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
    let mut currency = None;

    let mut first_date = None;
    let mut last_date = None;

    let mut periods: Vec<Period> = vec![];

    for t in ledger.transactions.iter() {
        // cash_flows
        for p in t.postings.borrow().iter() {
            if !filter::filter(&options, &cash_flows_node, t, p, &mut ledger.commodities)? {
                continue;
            }
            let index = get_period_index(p.date, &mut periods, frequency);
            let period = &mut periods[index];

            if first_date.is_none() {
                first_date = Some(p.date.clone());
            }
            last_date = Some(p.date.clone());

            match currency.as_ref() {
                Some(_) => (),
                None => {
                    currency = Some(p.amount.as_ref().unwrap().get_commodity().unwrap().clone())
                }
            }
            if p.amount.as_ref().unwrap().get_commodity().unwrap()
                == currency.as_ref().unwrap().clone()
            {
                period.add_cash(p.amount.as_ref().unwrap().to_owned());
            } else {
                let multipliers =
                    conversion(currency.as_ref().unwrap().clone(), p.date, &ledger.prices);
                let mult = multipliers
                    .get(p.amount.as_ref().unwrap().get_commodity().unwrap().as_ref())
                    .unwrap();
                let new_amount = Money::Money {
                    amount: p.amount.as_ref().unwrap().get_amount() * mult.clone(),
                    currency: currency.as_ref().unwrap().clone(),
                };
                period.add_cash(new_amount);
            }
        } // cash flows

        // balances
        for p in t.postings.borrow().iter() {
            if !filter::filter(&options, &assets_value_node, t, p, &mut ledger.commodities)? {
                continue;
            }
            let index = get_period_index(p.date, &mut periods, frequency);
            let mut period = &mut periods[index];

            period.final_balance =
                period.final_balance.clone() + Balance::from(p.amount.as_ref().unwrap().to_owned());
        } // balances
    }

    let mut insertions: Vec<Period> = vec![];
    periods.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    let mut last_period_date = None;
    for p in periods.iter_mut() {
        if last_period_date.is_none() {
            last_period_date = Some(p.end.clone());
            continue;
        }
        // Because the gap may be more than one month, we need a loop
        'inner: loop {
            let expected = last_period_date.unwrap() + Duration::days(1);
            last_period_date = Some(period_ending(expected.clone(), frequency));
            if expected == p.start {
                break 'inner;
            }
            let new_period = Period::from_date(expected, frequency);
            insertions.push(new_period);
        }
    }

    periods.append(&mut insertions);
    periods.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    let mut prev_final_balance = Balance::new();
    let mut prev_final_money = None;

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(
        row![r->"Begin", r->"End", r->"Value (begin)", r->"Cash flow", r->"Value (end)", r->"TWR", r->"TWR_y"],
    );
    for (i, p) in periods.iter_mut().enumerate() {
        if i > 0 {
            p.initial_balance = prev_final_balance;
            p.initial_money = prev_final_money;
        }
        p.final_balance = p.final_balance.clone() + p.initial_balance.clone();

        if p.final_money.is_none() {
            let multipliers = conversion(currency.as_ref().unwrap().clone(), p.end, &ledger.prices);
            p.final_money = Some(
                convert_balance(&p.final_balance, &multipliers, currency.as_ref().unwrap())
                    .to_money()
                    .unwrap(),
            );
        }
        if p.initial_money.is_none() {
            let multipliers =
                conversion(currency.as_ref().unwrap().clone(), p.start, &ledger.prices);
            p.initial_money = Some(
                convert_balance(&p.initial_balance, &multipliers, currency.as_ref().unwrap())
                    .to_money()
                    .unwrap(),
            );
        }

        table.add_row(row![
            format!("{}", p.start),
            format!("{}", p.end),
            r->format!("{}", p.initial_money.as_ref().unwrap()),
            r->format!("{}", p.cash_flow),
            r->format!("{}", p.final_money.as_ref().unwrap()),
            r->format!("{:.2}%", (&p.twr() * BigInt::from(100)).to_f64().unwrap()),
            r->format!("{:.2}%", &p.twr_annualized() * 100 as f64),
        ]);

        prev_final_balance = p.final_balance.clone();
        prev_final_money = p.final_money.clone();
    }
    // Print the table to stdout
    table.printstd();

    // Add a summary
    // Final unit price: 22720.00/134.93=168.38 U.
    // Total TWR: 68.38%.
    // Period: 5.41 years.
    // Annualized TWR: 10.12%
    let mut total_twr = 1.0;
    for p in periods.iter() {
        total_twr *= 1.0 + p.twr().to_f64().unwrap();
        // dbg!(&total_twr);
    }
    total_twr = (total_twr - 1.0);
    let num_days = ((last_date.unwrap() - first_date.unwrap()).num_days() + 1) as f64;
    let twr_annualized = (1.0 + total_twr).powf(365.25 / num_days) - 1.0;
    println!("Total TWR: {:.2}%", total_twr * 100.0);
    println!("Period: {:.2} years", num_days / 365.25);
    println!("Annualized TWR: {:.2}%", twr_annualized * 100.0);
    Ok(())
}

fn get_period_index(date: NaiveDate, periods: &mut Vec<Period>, frequency: Frequency) -> usize {
    let begin = period_beginning(date, frequency);
    for (i, period) in periods.iter().enumerate() {
        if period.start == begin {
            return i;
        }
    }
    let period = Period::from_date(date, frequency);
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
    fn from_date(d: NaiveDate, frequency: Frequency) -> Self {
        Period {
            start: period_beginning(d, frequency),
            end: period_ending(d, frequency),
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
        let flow = self.cash_flow.get_amount();
        if initial.is_zero() {
            if flow.is_zero() {
                return flow;
            } else {
                return -(end + &flow) / flow;
            };
        }
        // (end - initial + flow) / initial
        let twr = (end - &initial + flow) / initial;
        twr
    }

    fn twr_annualized(&self) -> f64 {
        let twr = self.twr().to_f64().unwrap() + 1.0;
        let num_days = 1 + (self.end - self.start).num_days();
        twr.powf(365.25 / (num_days as f64)) - 1.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Frequency {
    Monthly,
    Quarterly,
    Yearly,
}

/// Period groups
impl From<PeriodGroup> for Frequency {
    fn from(p: PeriodGroup) -> Self {
        if !p.monthly & !p.quarterly & p.yearly {
            Frequency::Yearly
        } else if !p.monthly & p.quarterly & !p.yearly {
            Frequency::Quarterly
        } else if p.monthly & !p.quarterly & !p.yearly {
            Frequency::Monthly
        }
        // default to yearly
        else if !p.monthly & !p.quarterly & !p.yearly {
            Frequency::Yearly
        } else {
            panic!("Incompatible options")
        }
    }
}
/// Returns the first day of the month
fn period_beginning(d: NaiveDate, frequency: Frequency) -> NaiveDate {
    match frequency {
        Frequency::Monthly => NaiveDate::from_ymd(d.year(), d.month(), 1),
        Frequency::Quarterly => NaiveDate::from_ymd(d.year(), ((d.month() - 1) / 3) * 3 + 1, 1),
        Frequency::Yearly => NaiveDate::from_ymd(d.year(), 1, 1),
    }
}

/// Returns the last day of the period
fn period_ending(d: NaiveDate, frequency: Frequency) -> NaiveDate {
    // Find the beginning of the next period and subtract one day
    let month: u32;

    match d.month() {
        12 => NaiveDate::from_ymd(d.year(), 12, 31),
        other => match frequency {
            Frequency::Monthly => {
                month = other + 1;
                NaiveDate::from_ymd(d.year(), month, 1) - Duration::days(1)
            }
            Frequency::Quarterly => {
                if d.month() > 9 {
                    NaiveDate::from_ymd(d.year(), 12, 31)
                } else {
                    NaiveDate::from_ymd(d.year(), ((d.month() - 1) / 3) * 3 + 4, 1)
                        - Duration::days(1)
                }
            }
            Frequency::Yearly => NaiveDate::from_ymd(d.year(), 12, 31),
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_january() {
        let date = NaiveDate::from_ymd(2019, 1, 15);
        assert_eq!(
            period_ending(date, Frequency::Monthly),
            NaiveDate::from_ymd(2019, 1, 31)
        );
        assert_eq!(
            period_ending(date, Frequency::Quarterly),
            NaiveDate::from_ymd(2019, 3, 31)
        );
        assert_eq!(
            period_ending(date, Frequency::Yearly),
            NaiveDate::from_ymd(2019, 12, 31)
        );
        assert_eq!(
            period_beginning(date, Frequency::Monthly),
            NaiveDate::from_ymd(2019, 1, 1)
        );
        assert_eq!(
            period_beginning(date, Frequency::Quarterly),
            NaiveDate::from_ymd(2019, 1, 1)
        );
        assert_eq!(
            period_beginning(date, Frequency::Yearly),
            NaiveDate::from_ymd(2019, 1, 1)
        );
    }
    #[test]
    fn test_march() {
        let date = NaiveDate::from_ymd(2019, 3, 15);
        assert_eq!(
            period_ending(date, Frequency::Monthly),
            NaiveDate::from_ymd(2019, 3, 31)
        );
        assert_eq!(
            period_ending(date, Frequency::Quarterly),
            NaiveDate::from_ymd(2019, 3, 31)
        );
        assert_eq!(
            period_ending(date, Frequency::Yearly),
            NaiveDate::from_ymd(2019, 12, 31)
        );
        assert_eq!(
            period_beginning(date, Frequency::Monthly),
            NaiveDate::from_ymd(2019, 3, 1)
        );
        assert_eq!(
            period_beginning(date, Frequency::Quarterly),
            NaiveDate::from_ymd(2019, 1, 1)
        );
        assert_eq!(
            period_beginning(date, Frequency::Yearly),
            NaiveDate::from_ymd(2019, 1, 1)
        );
    }
}
