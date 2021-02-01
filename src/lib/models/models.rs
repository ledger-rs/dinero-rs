use crate::models::HasName;
use chrono::NaiveDate;
use num::rational::Rational64;

#[derive(Debug, Clone)]
pub struct ParsedPrice {
    pub(crate) date: NaiveDate,
    pub(crate) commodity: String,
    pub(crate) other_commodity: String,
    pub(crate) other_quantity: Rational64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Tag {
    pub name: String,
    pub check: Vec<String>,
    pub assert: Vec<String>,
}

impl HasName for Tag {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}
