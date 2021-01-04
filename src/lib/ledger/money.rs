use num;
use crate::ledger::Currency;
use std::collections::HashMap;
use std::ops::{Add, Mul, Neg};
use num::rational::Rational64;
use num::{Zero, Signed};

/// Money representation: an amount and a currency
///
/// It is important that calculations are not done with floats but with Rational numbers so that
/// everything adds up correctly
///
/// Money can be added, in which case it returns a balance, as it can have several currencies
/// # Examples
/// ```rust
/// # use dinero::ledger::{Money, Balance, Currency};
/// # use num::rational::Rational64;
/// #
/// let usd = Currency::from("usd");
/// let eur = Currency::from("eur");
///
/// let zero = Money::new();
/// let m1 = Money::from((&eur, Rational64::new(100,1)));
/// let m2 = Money::from((&eur, Rational64::new(200,1)));
/// # let m3 = Money::from((&eur, Rational64::new(300,1)));
/// let b1 = m1 + m2; // 300 euros
/// # assert_eq!(*b1.balance.get(&Some(&eur)).unwrap(), m3);
///
/// // Multicurrency works as well
/// let d1 = Money::from((&usd, Rational64::new(50,1)));
/// let b2 = d1 + m1; // 100 euros and 50 usd
/// # assert_eq!(b2.balance.len(), 2);
/// # assert_eq!(*b2.balance.get(&Some(&eur)).unwrap(), m1);
/// # assert_eq!(*b2.balance.get(&Some(&usd)).unwrap(), d1);
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
                match total.to_owned().get(cur) {
                    None => total.insert(*cur, money.clone()),
                    Some(total_money) => match total_money {
                        Money::Zero => total.insert(*cur, money.clone()),
                        Money::Money { amount: already, .. } => match money {
                            Money::Zero => None,
                            Money::Money { amount, .. } => total.insert(
                                *cur, Money::from((
                                    cur.unwrap(),
                                    amount + already.to_owned()
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

impl<'a> Balance<'a> {
    pub fn new() -> Balance<'a> {
        Balance { balance: HashMap::new() }
    }
    pub fn is_zero(&self) -> bool {
        match self.balance.is_empty() {
            true => true,
            false => {
                for (_, money) in self.balance.iter() {
                    if !money.is_zero() { return false; }
                }
                true
            }
        }
    }
    pub fn can_be_zero(&self) -> bool {
        if self.is_zero() { return true; }
        let mut positive = false;
        let mut negative = false;
        for (_, m) in self.balance.iter() {
            if positive & negative { return true; }
            positive = m.is_positive() | positive;
            negative = m.is_negative() | negative;
        }
        false
    }
}

impl<'a> Money<'a> {
    pub fn is_zero(&self) -> bool {
        match self {
            Money::Zero => true,
            Money::Money { amount, .. } => amount.is_zero()
        }
    }
    pub fn is_positive(&self) -> bool {
        match self {
            Money::Zero => true,
            Money::Money { amount, .. } => amount.is_positive()
        }
    }
    pub fn is_negative(&self) -> bool {
        match self {
            Money::Zero => true,
            Money::Money { amount, .. } => amount.is_negative()
        }
    }
}

impl<'a> Neg for Money<'a> {
    type Output = Money<'a>;

    fn neg(self) -> Self::Output {
        match self {
            Money::Zero => Money::Zero,
            Money::Money { currency, amount } =>
                Money::Money {
                    currency,
                    amount: -amount,
                }
        }
    }
}

impl<'a> Neg for Balance<'a> {
    type Output = Balance<'a>;

    fn neg(self) -> Self::Output {
        let mut balance: HashMap<Option<&Currency>, Money> = HashMap::new();
        for (k, v) in self.balance.iter() {
            balance.insert(*k, -*v);
        }
        Balance { balance }
    }
}

/// A price relates two commodities
pub struct Price<'a> {
    pub date: &'a str,
    pub commodity: Money<'a>,
    pub price: Money<'a>,
}