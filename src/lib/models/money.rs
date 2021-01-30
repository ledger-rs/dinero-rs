use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul, Neg};
use std::rc::Rc;

use num;
use num::rational::Rational64;
use num::{Signed, Zero};

use crate::models::balance::Balance;
use crate::models::{Currency, HasName};

/// Money representation: an amount and a currency
///
/// It is important that calculations are not done with floats but with Rational numbers so that
/// everything adds up correctly
///
/// Money can be added, in which case it returns a balance, as it can have several currencies
/// # Examples
/// ```rust
/// # use dinero::models::{Money, Balance, Currency};
/// # use num::rational::Rational64;
/// # use std::rc::Rc;
/// #
/// let usd = Rc::new(Currency::from("usd"));
/// let eur = Rc::new(Currency::from("eur"));
///
/// let zero = Money::new();
/// let m1 = Money::from((eur.clone(), Rational64::new(100,1)));
/// let m2 = Money::from((eur.clone(), Rational64::new(200,1)));
/// # let m3 = Money::from((eur.clone(), Rational64::new(300,1)));
/// let b1 = m1.clone() + m2; // 300 euros
/// # assert_eq!(*b1.balance.get(&Some(eur.clone())).unwrap(), m3);
///
/// // Multicurrency works as well
/// let d1 = Money::from((usd.clone(), Rational64::new(50,1)));
/// let b2 = d1.clone() + m1.clone(); // 100 euros and 50 usd
/// # assert_eq!(b2.balance.len(), 2);
/// # assert_eq!(*b2.balance.get(&Some(eur.clone())).unwrap(), m1);
/// # assert_eq!(*b2.balance.get(&Some(usd.clone())).unwrap(), d1);
/// ```
#[derive(Clone, Debug)]
pub enum Money {
    Zero,
    Money {
        amount: num::rational::Rational64,
        currency: Rc<Currency>,
    },
}

impl Money {
    pub fn new() -> Self {
        Money::Zero
    }
    pub fn is_zero(&self) -> bool {
        match self {
            Money::Zero => true,
            Money::Money { amount, .. } => amount.is_zero(),
        }
    }
    pub fn is_positive(&self) -> bool {
        match self {
            Money::Zero => true,
            Money::Money { amount, .. } => amount.is_positive(),
        }
    }
    pub fn is_negative(&self) -> bool {
        match self {
            Money::Zero => true,
            Money::Money { amount, .. } => amount.is_negative(),
        }
    }
    pub fn get_commodity(&self) -> Option<Rc<Currency>> {
        match self {
            Money::Zero => None,
            Money::Money { currency, .. } => Some(currency.clone()),
        }
    }
    pub fn get_amount(&self) -> Rational64 {
        match self {
            Money::Zero => Rational64::new(0, 1),
            Money::Money { amount, .. } => amount.clone(),
        }
    }
    pub fn abs(&self) -> Money {
        match self.is_negative() {
            true => -self.clone(),
            false => self.clone(),
        }
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Money::Zero => write!(f, "{}", "0"),
            Money::Money { amount, currency } => {
                let value = amount.numer().clone() as f64 / amount.denom().clone() as f64;
                write!(f, "{} {}", value, currency.get_name())
            }
        }
    }
}

impl Eq for Money {}

impl PartialEq for Money {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Money::Zero => match other {
                Money::Zero => true,
                Money::Money { amount, .. } => amount.is_zero(),
            },
            Money::Money {
                amount: a1,
                currency: c1,
            } => match other {
                Money::Zero => a1.is_zero(),
                Money::Money {
                    amount: a2,
                    currency: c2,
                } => (a1 == a2) & (c1 == c2),
            },
        }
    }
}

impl Mul<Rational64> for Money {
    type Output = Money;

    fn mul(self, rhs: Rational64) -> Self::Output {
        match self {
            Money::Zero => Money::new(),
            Money::Money { amount, currency } => Money::from((currency, amount * rhs)),
        }
    }
}

impl From<(Rc<Currency>, Rational64)> for Money {
    fn from(cur_amount: (Rc<Currency>, Rational64)) -> Self {
        let (currency, amount) = cur_amount;
        Money::Money { amount, currency }
    }
}

impl From<(Currency, Rational64)> for Money {
    fn from(cur_amount: (Currency, Rational64)) -> Self {
        let (currency, amount) = cur_amount;
        Money::Money {
            amount,
            currency: Rc::new(currency),
        }
    }
}

impl Add for Money {
    type Output = Balance;

    fn add(self, rhs: Self) -> Self::Output {
        let b1 = Balance::from(self);
        let b2 = Balance::from(rhs);
        b1 + b2
    }
}

impl<'a> Neg for Money {
    type Output = Money;

    fn neg(self) -> Self::Output {
        match self {
            Money::Zero => Money::Zero,
            Money::Money { currency, amount } => Money::Money {
                currency,
                amount: -amount,
            },
        }
    }
}
