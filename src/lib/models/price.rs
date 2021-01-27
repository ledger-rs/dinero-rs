use crate::models::Money;
use chrono::NaiveDate;
use std::fmt;
use std::fmt::{Display, Formatter};

/// A price relates two commodities
#[derive(Debug, Clone)]
pub struct Price {
    pub date: NaiveDate,
    pub commodity: Money,
    pub price: Money,
}

impl Price {
    pub fn get_price(&self) -> Money {
        Money::Money {
            currency: self.price.get_commodity().unwrap(),
            amount: self.price.get_amount() / self.commodity.get_amount(),
        }
    }
}

impl Display for Price {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.date,
            self.commodity.get_commodity().unwrap(),
            self.get_price()
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PriceType {
    Total,
    PerUnit,
}
