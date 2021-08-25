use crate::models::{FromDirective, HasAliases, HasName, Origin};
use regex::Regex;
use std::cell::RefCell;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// An account
#[derive(Debug, Clone)]
pub struct Account {
    name: String,
    pub(crate) origin: Origin,
    pub(crate) note: Option<String>,
    pub(crate) iban: Option<String>,
    pub(crate) country: Option<String>,
    pub(crate) aliases: HashSet<String>,
    pub(crate) check: Vec<String>,
    pub(crate) assert: Vec<String>,
    pub(crate) payee: Vec<Regex>,
    pub(crate) default: bool,
    matches: RefCell<HashMap<String, bool>>,
}

impl Account {
    pub fn from_directive(name: String) -> Account {
        Account {
            name,
            origin: Origin::FromDirective,
            note: None,
            iban: None,
            country: None,
            aliases: HashSet::new(),
            check: vec![],
            assert: vec![],
            payee: vec![],
            default: false,
            matches: RefCell::new(HashMap::new()),
        }
    }
    pub fn is_default(&self) -> bool {
        self.default
    }
    pub fn payees(&self) -> &Vec<Regex> {
        &self.payee
    }

    /// Depth of the account, useful for filters and other
    pub fn depth(&self) -> usize {
        self.name.chars().filter(|c| *c == ':').count() + 1
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
}
impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Account {}

impl HasAliases for Account {
    fn get_aliases(&self) -> &HashSet<String, RandomState> {
        &self.aliases
    }
}

impl Hash for Account {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<&str> for Account {
    fn from(name: &str) -> Self {
        Account {
            name: String::from(name),
            origin: Origin::Other,
            note: None,
            iban: None,
            country: None,
            aliases: Default::default(),
            check: vec![],
            assert: vec![],
            payee: vec![],
            default: false,
            matches: RefCell::new(HashMap::new()),
        }
    }
}

impl FromDirective for Account {
    fn is_from_directive(&self) -> bool {
        matches!(self.origin, Origin::FromDirective)
    }
}

impl HasName for Account {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}
