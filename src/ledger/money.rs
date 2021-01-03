use num;
use crate::ledger::Currency;
use std::collections::HashMap;
use std::ops::{Add, Mul};
use num::rational::Rational64;
use num::Zero;

/// Money representation: an amount and a currency
/// It is important that calculations are not done with floats but with Rational numbers so that
/// everything adds up correctly
///
/// Money can be added, in which case it returns a balance, as it can have several currencies

/// ```rust
/// use dinero::ledger::{Money, Balance};
/// use dinero::ledger::Currency;
/// use num::rational::Rational64;
///
/// let usd = Currency::from("usd");
/// let eur = Currency::from("eur");
///
/// let zero = Money::new();
/// let m1 = Money::from((&eur, Rational64::new(100,1)));
/// let m2 = Money::from((&eur, Rational64::new(200,1)));
/// let m3 = Money::from((&eur, Rational64::new(300,1)));
/// let b1 = m1 + m2;
/// assert_eq!(*b1.balance.get(&Some(&eur)).unwrap(), m3);
///
///
/// ```
#[derive(Copy, Clone, Debug)]
pub enum Money<'a> {
    Zero,
    Money {
        amount: num::rational::Rational64,
        currency: &'a Currency<'a>,
    },
}

impl Eq for Money<'_> {}

impl<'a> PartialEq for Money<'a> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Money::Zero => match other {
                Money::Zero => true,
                Money::Money { amount, .. } => amount.is_zero(),
            },
            Money::Money { amount: a1, currency: c1 } => match other {
                Money::Zero => a1.is_zero(),
                Money::Money { amount: a2, currency: c2 } => (a1 == a2) & (c1 == c2),
            }
        }
    }
}

impl Money<'_> {
    pub fn new() -> Self {
        Money::Zero
    }
}

impl<'a> Mul<Rational64> for Money<'a> {
    type Output = Money<'a>;

    fn mul(self, rhs: Rational64) -> Self::Output {
        match self {
            Money::Zero => Money::new(),
            Money::Money { amount, currency } => Money::from((currency, amount * rhs)),
        }
    }
}

impl<'a> From<(&'a Currency<'a>, Rational64)> for Money<'a> {
    fn from(cur_amount: (&'a Currency<'a>, Rational64)) -> Self {
        let (currency, amount) = cur_amount;
        Money::Money {
            amount,
            currency,
        }
    }
}

pub struct Balance<'a> {
    pub balance: HashMap<Option<&'a Currency<'a>>, Money<'a>>,
}

impl<'a> From<Money<'a>> for Balance<'a> {
    fn from(money: Money<'a>) -> Self {
        let mut balance: HashMap<Option<&Currency>, Money> = HashMap::new();
        match money {
            Money::Zero => balance.insert(None, Money::Zero),
            Money::Money { currency, .. } => {
                balance.insert(Some(currency), money.clone())
            }
        };
        Balance { balance }
    }
}

impl<'a> Add for Money<'a> {
    type Output = Balance<'a>;

    fn add(self, rhs: Self) -> Self::Output {
        let b1 = Balance::from(self);
        let b2 = Balance::from(rhs);
        b1 + b2
    }
}

impl<'a> Add for Balance<'a> {
    type Output = Balance<'a>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut total: HashMap<Option<&Currency>, Money> = HashMap::new();
        let balances = vec![self, rhs];
        for bal in balances.iter() {
            for (cur, money) in bal.balance.iter() {
                match total.get(cur) {
                    None => total.insert(*cur, money.clone()),
                    Some(total_money) => match total_money {
                        Money::Zero => total.insert(*cur, money.clone()),
                        Money::Money { amount: already, .. } => match money {
                            Money::Zero => None,
                            Money::Money { amount, .. } => total.insert(
                                *cur, Money::from((
                                    cur.unwrap(),
                                    amount + already
                                )),
                            )
                        }
                    }
                };
            }
        }
        Balance { balance: total }
    }
}

/// A price relates two commodities
pub struct Price<'a> {
    pub date: &'a str,
    pub commodity: Money<'a>,
    pub price: Money<'a>,
}