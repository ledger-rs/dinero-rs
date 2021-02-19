use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul, Neg, Sub};
use std::rc::Rc;

use num;
use num::rational::BigRational;
use num::{BigInt, Signed, Zero};

use crate::models::balance::Balance;
use crate::models::{Currency, HasName};
use std::str::FromStr;

/// Money representation: an amount and a currency
///
/// It is important that calculations are not done with floats but with Rational numbers so that
/// everything adds up correctly
///
/// Money can be added, in which case it returns a balance, as it can have several currencies
/// # Examples
/// ```rust
/// # use dinero::models::{Money, Balance, Currency};
/// # use num::rational::BigRational;
/// # use std::rc::Rc;
/// use num::BigInt;
/// #
/// let usd = Rc::new(Currency::from("usd"));
/// let eur = Rc::new(Currency::from("eur"));
///
/// let zero = Money::new();
/// let m1 = Money::from((eur.clone(), BigRational::from(BigInt::from(100))));
/// let m2 = Money::from((eur.clone(), BigRational::from(BigInt::from(200))));
/// # let m3 = Money::from((eur.clone(), BigRational::from(BigInt::from(300))));
/// let b1 = m1.clone() + m2; // 300 euros
/// # assert_eq!(*b1.balance.get(&Some(eur.clone())).unwrap(), m3);
///
/// // Multicurrency works as well
/// let d1 = Money::from((usd.clone(), BigRational::from(BigInt::from(50))));
/// let b2 = d1.clone() + m1.clone(); // 100 euros and 50 usd
/// # assert_eq!(b2.balance.len(), 2);
/// # assert_eq!(*b2.balance.get(&Some(eur.clone())).unwrap(), m1);
/// # assert_eq!(*b2.balance.get(&Some(usd.clone())).unwrap(), d1);
/// ```
#[derive(Clone, Debug)]
pub enum Money {
    Zero,
    Money {
        amount: num::rational::BigRational,
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
    pub fn get_amount(&self) -> BigRational {
        match self {
            Money::Zero => BigRational::new(BigInt::from(0), BigInt::from(1)),
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
                // num = trunc + fract
                let decimals: usize = currency.get_precision();
                let base: i32 = 10;
                let mut integer_part = amount.trunc();
                let decimal_part = (amount.fract() * BigInt::from(base.pow(decimals as u32 + 2)))
                    .abs()
                    .trunc();
                let mut decimal_str =
                    format!("{:0width$}", decimal_part.numer(), width = decimals + 2);
                decimal_str.truncate(decimals + 1);

                let mut decimal = if u64::from_str(&decimal_str).unwrap() % 10 >= 5 {
                    u64::from_str(&decimal_str).unwrap() / 10 + 1
                } else {
                    u64::from_str(&decimal_str).unwrap() / 10
                };
                let len = format!("{}", decimal).len();
                if len == decimals + 1 {
                    decimal = 0;
                    if integer_part.is_positive() {
                        integer_part += BigInt::from(1);
                    } else {
                        integer_part -= BigInt::from(1);
                    }
                }
                decimal_str = format!("{:0width$}", decimal, width = decimals);
                write!(
                    f,
                    "{}.{} {}",
                    integer_part.numer(),
                    decimal_str,
                    currency.get_name()
                )
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

impl Mul<BigRational> for Money {
    type Output = Money;

    fn mul(self, rhs: BigRational) -> Self::Output {
        match self {
            Money::Zero => Money::new(),
            Money::Money { amount, currency } => Money::from((currency, amount * rhs)),
        }
    }
}

impl From<(Rc<Currency>, BigRational)> for Money {
    fn from(cur_amount: (Rc<Currency>, BigRational)) -> Self {
        let (currency, amount) = cur_amount;
        Money::Money { amount, currency }
    }
}

impl From<(Currency, BigRational)> for Money {
    fn from(cur_amount: (Currency, BigRational)) -> Self {
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
impl Sub for Money {
    type Output = Balance;

    fn sub(self, rhs: Self) -> Self::Output {
        let b1 = Balance::from(self);
        let b2 = Balance::from(rhs);
        b1 - b2
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
