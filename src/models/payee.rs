use crate::models::{FromDirective, HasAliases, HasName, Origin};
use regex::Regex;
use std::cell::RefCell;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Payee {
    name: String,
    note: Option<String>,
    alias: HashSet<String>,
    alias_regex: Vec<Regex>,
    origin: Origin,
    matches: RefCell<HashMap<String, bool>>,
}

impl Payee {
    pub fn new(
        name: String,
        note: Option<String>,
        alias: HashSet<String>,
        alias_regex: Vec<Regex>,
        origin: Origin,
    ) -> Payee {
        Payee {
            name,
            note,
            alias,
            alias_regex,
            origin,
            matches: RefCell::new(HashMap::new()),
        }
    }
    pub fn is_match(&self, regex: Regex) -> bool {
        let mut list = self.matches.borrow_mut();
        match list.get(regex.as_str()) {
            Some(x) => *x,

            None => {
                let value = regex.is_match(self.get_name());
                list.insert(regex.as_str().to_string(), value);
                value
            }
        }
    }
    pub fn get_aliases(&self) -> &Vec<Regex> {
        &self.alias_regex
    }
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
        matches!(self.origin, Origin::FromDirective)
    }
}

impl Hash for Payee {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<&str> for Payee {
    fn from(name: &str) -> Self {
        Payee::new(
            String::from(name),
            None,
            Default::default(),
            Default::default(),
            Origin::FromTransaction,
        )
    }
}
