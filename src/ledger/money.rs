use num;
use crate::ledger::Currency;

/// Money representation: an amount and a currency
/// It is important that calculations are not done with floats but with Rational numbers so that
/// everything adds up correctly
///
/// Money can be added, in which case it returns a balance, as it can have several currencies

#[derive(Copy, Clone)]
enum Money<'a> {
    Zero,
    Money {
        amount: num::rational::Rational64,
        currency: &'a Currency<'a>
    }
}
struct Balance<'a> {
    balance: Vec<Money<'a>>
}

/// A price relates two commodities
struct Price<'a> {
    date: &'a str,
    commodity: Money<'a>,
    price: Money<'a>
}