use crate::models::{FromDirective, HasAliases, HasName, Origin};
use chrono::NaiveDate;
use num::rational::Rational64;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

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
