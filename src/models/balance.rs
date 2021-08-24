use crate::error::LedgerError;
use crate::models::{Currency, Money};
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Neg, Sub};
use std::rc::Rc;

/// Balance is money with several currencies, for example 100 USD and 50 EUR
#[derive(Debug, Clone)]
pub struct Balance {
    pub balance: HashMap<Option<Rc<Currency>>, Money>,
}
impl Default for Balance {
    fn default() -> Self {
        Self::new()
    }
}
impl Balance {
    pub fn new() -> Balance {
        Balance {
            balance: HashMap::new(),
        }
    }

    /// Automatic conversion from balance to regular money
    /// it can only be done if the balance has only one currency
    pub fn to_money(&self) -> Result<Money, LedgerError> {
        let vec = self
            .balance
            .values()
            .filter(|x| !x.is_zero())
            .collect::<Vec<&Money>>();
        match vec.len() {
            0 => Ok(Money::Zero),
            1 => Ok(vec[0].clone()),
            _ => Err(LedgerError::TransactionIsNotBalanced),
        }
    }
    pub fn is_zero(&self) -> bool {
        match self.balance.is_empty() {
            true => true,
            false => {
                for (_, money) in self.balance.iter() {
                    if !money.is_zero() {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Whether a balance can be zero
    /// To be true, there must be positive and negative amounts
    pub fn can_be_zero(&self) -> bool {
        if self.is_zero() {
            return true;
        }
        let mut positive = false;
        let mut negative = false;
        for (_, m) in self.balance.iter() {
            positive = m.is_positive() | positive;
            negative = m.is_negative() | negative;
            if positive & negative {
                return true;
            }
        }
        false
    }
    pub fn len(&self) -> usize {
        self.balance.len()
    }
    pub fn iter(&self) -> Iter<'_, Option<Rc<Currency>>, Money> {
        self.balance.iter()
    }
}

impl Display for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        for (_, v) in self.balance.iter() {
            string.push_str(format!("{}", v).as_str());
        }
        write!(f, "{}", string)
    }
}

impl<'a> Neg for Balance {
    type Output = Balance;

    fn neg(self) -> Self::Output {
        let mut balance: HashMap<Option<Rc<Currency>>, Money> = HashMap::new();
        for (k, v) in self.balance {
            balance.insert(k, -v);
        }
        Balance { balance }
    }
}

impl Add for Balance {
    type Output = Balance;

    fn add(self, rhs: Self) -> Self::Output {
        let mut total: HashMap<Option<Rc<Currency>>, Money> = HashMap::new();
        let balances = vec![self, rhs];
        for bal in balances.iter() {
            for (cur, money) in bal.balance.iter() {
                match total.to_owned().get(cur) {
                    None => total.insert(cur.clone(), money.clone()),
                    Some(total_money) => match total_money {
                        Money::Zero => total.insert(cur.clone(), money.clone()),
                        Money::Money {
                            amount: already, ..
                        } => match money {
                            Money::Zero => None,
                            Money::Money { amount, .. } => total.insert(
                                cur.clone(),
                                Money::from((
                                    cur.as_ref().unwrap().clone(),
                                    amount + already.to_owned(),
                                )),
                            ),
                        },
                    },
                };
            }
        }

        Balance {
            balance: total.into_iter().filter(|(_, v)| !v.is_zero()).collect(),
        }
    }
}

impl Sub for Balance {
    type Output = Balance;

    fn sub(self, rhs: Self) -> Self::Output {
        let negative = -rhs;
        self + negative
    }
}

// Converter
impl From<Money> for Balance {
    fn from(money: Money) -> Self {
        let mut balance: HashMap<Option<Rc<Currency>>, Money> = HashMap::new();
        match money {
            Money::Zero => balance.insert(None, Money::Zero),
            Money::Money { ref currency, .. } => {
                balance.insert(Some(currency.clone()), money.clone())
            }
        };
        Balance { balance }
    }
}

impl PartialEq for Balance {
    fn eq(&self, other: &Self) -> bool {
        for (k, v) in self.balance.iter() {
            let other_money = other.balance.get(k);
            match other_money {
                None => {
                    if !v.is_zero() {
                        return false;
                    }
                }
                Some(money) => {
                    if money != v {
                        return false;
                    }
                }
            }
        }
        true
    }
}
