use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::rc::Rc;

use num;
use num::rational::BigRational;
use num::{BigInt, Signed, Zero};

use crate::models::balance::Balance;
use crate::models::currency::{CurrencySymbolPlacement, DigitGrouping, NegativeAmountDisplay};
use crate::models::{Currency, HasName};
use num::traits::Inv;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::str::FromStr;

/// Money representation: an amount and a currency
///
/// It is important that calculations are not done with floats but with Rational numbers so that
/// everything adds up correctly
///
/// Money can be added, in which case it returns a balance, as it can have several currencies
/// # Examples
/// ```rust
/// # use dinero::models::{Money, Balance, Currency, DigitGrouping};
/// # use num::rational::BigRational;
/// # use std::rc::Rc;
/// use num::BigInt;
/// #
/// let usd = Rc::new(Currency::from("usd"));
/// let mut eur = Rc::new(Currency::from("eur"));
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
///
/// // There are various display formats
/// // -4_285_714.28571... EUR
/// let mut euro = Currency::from("eur");
/// // euro.set_decimal_separator(',');
/// // euro.set_thousands_separator('.');
/// // euro.set_digit_grouping(DigitGrouping::Indian);
/// // let rc_euro = Rc::new(euro);
/// // let money = Money::from((rc_euro, BigRational::new(BigInt::from(-30000000), BigInt::from(7))));
/// // assert_eq!(format!("{}", &money), "-42.85.714,29 eur");
/// // assert_ne!(format!("{}", &money), "-4285714,29 eur");
///
///
/// ```
#[derive(Clone, Debug, PartialOrd)]
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

impl Ord for Money {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_amount = self.get_amount();
        let other_amount = other.get_amount();
        match self.get_commodity() {
            None => self_amount.cmp(other_amount.borrow()),
            Some(self_currency) => match other.get_commodity() {
                None => self_amount.cmp(other_amount.borrow()),
                Some(other_currency) => {
                    if self_currency == other_currency {
                        self_amount.cmp(other_amount.borrow())
                    } else {
                        panic!(
                            "Can't compare different currencies. {} and {}.",
                            self_currency, other_currency
                        );
                    }
                }
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

impl Div<BigRational> for Money {
    type Output = Money;

    fn div(self, rhs: BigRational) -> Self::Output {
        self * rhs.inv()
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

impl Display for Money {
    // [This is what Microsoft says about currency formatting](https://docs.microsoft.com/en-us/globalization/locale/currency-formatting)

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Money::Zero => write!(f, "{}", "0"),
            Money::Money { amount, currency } => {
                // Suppose: -1.234.567,000358 EUR

                // Get the format
                let format = currency.display_format;

                // num = trunc + fract
                let base: i32 = 10;
                let mut integer_part = amount.trunc(); // -1.234.567

                // TODO Read decimals from format, two as default
                let decimals = 3;

                let decimal_part = (amount.fract() * BigInt::from(base.pow(decimals as u32 + 2)))
                    .abs()
                    .trunc();
                let mut decimal_str = if decimals == 0 {
                    String::new()
                } else {
                    format!("{:0width$}", decimal_part.numer(), width = decimals + 2)
                };
                if decimals > 0 {
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
                    let decimal_separator = format.get_decimal_separator_str();
                    decimal_str =
                        format!("{}{:0width$}", decimal_separator, decimal, width = decimals);
                }

                let integer_str = {
                    match format.get_digit_grouping() {
                        DigitGrouping::None => integer_part.numer().abs().to_string(), // Do nothing
                        grouping => {
                            let mut group_size = 3;
                            let mut counter = 0;
                            let mut reversed = vec![];
                            let thousands_separator = format.get_thousands_separator_str();
                            for c in integer_part.to_string().chars().rev() {
                                if c == '-' {
                                    continue;
                                }

                                if counter == group_size {
                                    reversed.push(thousands_separator);
                                    if grouping == DigitGrouping::Indian {
                                        group_size = 2;
                                    }
                                    counter = 0;
                                }
                                reversed.push(c);
                                counter += 1;
                            }
                            reversed.iter().rev().collect()
                        }
                    }
                };

                let amount_str = format!("{}{}", integer_str, decimal_str);
                match format.symbol_placement {
                    CurrencySymbolPlacement::BeforeAmount => {
                        if amount.is_negative() {
                            match format.negative_amount_display {
                                NegativeAmountDisplay::BeforeSymbolAndNumber => {
                                    write!(f, "-{}{}", currency.get_name(), amount_str)
                                }
                                NegativeAmountDisplay::BeforeNumberBehindCurrency => {
                                    write!(f, "{}-{}", currency.get_name(), amount_str)
                                }
                                NegativeAmountDisplay::AfterNumber => {
                                    write!(f, "{}{}-", currency.get_name(), amount_str)
                                }
                                NegativeAmountDisplay::Parentheses => {
                                    write!(f, "({}{})", currency.get_name(), amount_str)
                                }
                            }
                        } else {
                            write!(f, "{}{}", currency.get_name(), amount_str)
                        }
                    }
                    CurrencySymbolPlacement::AfterAmount => {
                        if amount.is_negative() {
                            match format.negative_amount_display {
                                NegativeAmountDisplay::BeforeSymbolAndNumber
                                | NegativeAmountDisplay::BeforeNumberBehindCurrency => {
                                    write!(f, "-{} {}", amount_str, currency.get_name())
                                }
                                NegativeAmountDisplay::AfterNumber => {
                                    write!(f, "{}- {}", amount_str, currency.get_name())
                                }
                                NegativeAmountDisplay::Parentheses => {
                                    write!(f, "({} {})", amount_str, currency.get_name())
                                }
                            }
                        } else {
                            write!(f, "{} {}", amount_str, currency.get_name())
                        }
                    }
                }
            }
        }
    }
}
