use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::rc::Rc;

use num::rational::BigRational;
use num::{self, ToPrimitive};
use num::{BigInt, Signed, Zero};

use crate::models::balance::Balance;
use crate::models::currency::{CurrencySymbolPlacement, DigitGrouping, NegativeAmountDisplay};
use crate::models::{Currency, HasName};
use std::borrow::Borrow;
use std::cmp::Ordering;

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
/// let b1 = m1.clone() + m2.clone(); // 300 euros
/// # assert_eq!(*b1.balance.get(&Some(eur.clone())).unwrap(), m3);
///
/// // Multi-currency works as well
/// let d1 = Money::from((usd.clone(), BigRational::from(BigInt::from(50))));
/// let b2 = d1.clone() + m1.clone(); // 100 euros and 50 usd
/// assert_eq!(b2.balance.len(), 2);
/// assert_eq!(*b2.balance.get(&Some(eur.clone())).unwrap(), m1);
/// assert_eq!(*b2.balance.get(&Some(usd.clone())).unwrap(), d1);
///
/// let b3 = b1 - Balance::from(m2.clone()) + Balance::from(Money::new());
/// assert_eq!(b3.to_money().unwrap(), m1);
/// ```
#[derive(Clone, Debug)]
pub enum Money {
    Zero,
    Money {
        amount: num::rational::BigRational,
        currency: Rc<Currency>,
    },
}
impl Default for Money {
    fn default() -> Self {
        Self::new()
    }
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
        match self.partial_cmp(other) {
            Some(c) => c,
            None => {
                let self_commodity = self.get_commodity().unwrap();
                let other_commodity = other.get_commodity().unwrap();
                panic!(
                    "Can't compare different currencies. {} and {}.",
                    self_commodity, other_commodity
                );
            }
        }
    }
}
impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_amount = self.get_amount();
        let other_amount = other.get_amount();
        match self.get_commodity() {
            None => self_amount.partial_cmp(other_amount.borrow()),
            Some(self_currency) => match other.get_commodity() {
                None => self_amount.partial_cmp(other_amount.borrow()),
                Some(other_currency) => {
                    if self_currency == other_currency {
                        self_amount.partial_cmp(other_amount.borrow())
                    } else {
                        None
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
        match self {
            Money::Zero => Money::new(),
            Money::Money { amount, currency } => Money::from((currency, amount / rhs)),
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

impl Display for Money {
    // [This is what Microsoft says about currency formatting](https://docs.microsoft.com/en-us/globalization/locale/currency-formatting)

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Money::Zero => write!(f, "0"),
            Money::Money { amount, currency } => {
                // Suppose: -1.234.567,000358 EUR
                // Get the format
                let format = currency.display_format.borrow();
                // Read number of decimals from format
                let decimals = match format.max_decimals {
                    Some(d) => d,
                    None => format.precision,
                };

                let full_str = format!("{:.*}", decimals, amount.to_f64().unwrap());

                // num = trunc + fract
                let integer_part: String = full_str
                    .split('.')
                    .next()
                    .unwrap()
                    .split('-')
                    .last()
                    .unwrap()
                    .into(); // -1.234.567

                let decimal_part = amount.fract().to_f64().unwrap();

                // decimal_str holds the decimal part without the dot (decimal separator)
                let mut decimal_str = if decimals == 0 {
                    String::new()
                } else {
                    format!("{:.*}", decimals, &decimal_part)
                        .split('.')
                        .last()
                        .unwrap()
                        .into()
                };
                // now add the dot
                if decimals > 0 {
                    decimal_str = format!("{}{}", format.get_decimal_separator_str(), decimal_str);
                }

                let integer_str = {
                    match format.get_digit_grouping() {
                        DigitGrouping::None => integer_part, // Do nothing
                        grouping => {
                            let mut group_size = 3;
                            let mut counter = 0;
                            let mut reversed = vec![];
                            match format.get_thousands_separator_str() {
                                Some(character) => {
                                    let thousands_separator = character;
                                    for c in integer_part.chars().rev() {
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
                                None => integer_part,
                            }
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use num::BigRational;

    use crate::models::{Currency, CurrencyDisplayFormat};

    use super::Money;

    #[test]
    fn rounding() {
        let one_decimal_format = CurrencyDisplayFormat::from("-1234.5 EUR");
        let no_decimal_format = CurrencyDisplayFormat::from("-1234 EUR");
        let eur = Rc::new(Currency::from("EUR"));

        // Money amount
        let m1 = Money::from((eur.clone(), BigRational::from_float(-17.77).unwrap()));

        eur.set_format(&one_decimal_format);
        assert_eq!(format!("{}", &m1), "-17.8 EUR");

        eur.set_format(&no_decimal_format);
        assert_eq!(format!("{}", &m1), "-18 EUR");
    }
}
