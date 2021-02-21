use crate::models::{FromDirective, HasAliases, HasName, Origin};
use regex::Regex;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Payee {
    pub name: String,
    pub note: Option<String>,
    pub alias: HashSet<String>,
    pub(crate) origin: Origin,
}

impl Eq for Payee {}

impl PartialEq for Payee {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Display for Payee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl HasName for Payee {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl HasAliases for Payee {
    fn get_aliases(&self) -> &HashSet<String, RandomState> {
        &self.alias
    }
}

impl FromDirective for Payee {
    fn is_from_directive(&self) -> bool {
        match self.origin {
            Origin::FromDirective => true,
            _ => false,
        }
    }
}

impl Hash for Payee {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl Payee {
    pub fn is_match(&self, regex: Regex) -> bool {
        regex.is_match(self.get_name())
    }
}

impl From<&str> for Payee {
    fn from(name: &str) -> Self {
        Payee {
            name: String::from(name),
            note: None,
            alias: Default::default(),
            origin: Origin::FromTransaction,
        }
    }
}
