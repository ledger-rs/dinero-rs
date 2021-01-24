use crate::ledger::{FromDirective, HasAliases, HasName, Origin};
use crate::List;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Account {
    pub(crate) name: String,
    pub(crate) origin: Origin,
    pub(crate) note: Option<String>,
    pub(crate) isin: Option<String>,
    pub(crate) aliases: HashSet<String>,
    pub(crate) check: Vec<String>,
    pub(crate) assert: Vec<String>,
    pub(crate) payee: Vec<String>,
    pub(crate) default: bool,
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
            isin: None,
            aliases: Default::default(),
            check: vec![],
            assert: vec![],
            payee: vec![],
            default: false,
        }
    }
}

impl FromDirective for Account {
    fn is_from_directive(&self) -> bool {
        match self.origin {
            Origin::FromDirective => true,
            _ => false,
        }
    }
}

impl HasName for Account {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl<'a> Account {
    /// Depth of the account, useful for filters and other
    pub fn depth(&self) -> usize {
        self.name
            .chars()
            .filter(|c| *c == ':')
            .collect::<Vec<char>>()
            .len()
            + 1
    }

    /// Parent name
    ///
    /// Returns the name of the parent account should have
    /// ```rust
    /// use dinero::ledger::Account;
    /// let mut account = Account::from("Expenses:Groceries");
    /// let mut parent = account.parent_name();
    /// assert_eq!(parent, Some("Expenses".to_string()));
    ///
    /// account = Account::from("Expenses:Groceries:Supermarket");
    /// parent = account.parent_name();
    /// assert_eq!(parent, Some("Expenses:Groceries".to_string()));
    ///
    /// account = Account::from("Expenses");
    /// parent = account.parent_name();
    /// assert_eq!(parent, None);
    /// ```
    pub fn parent_name(&self) -> Option<String> {
        match self.depth() {
            1 => None,
            _ => {
                let split = self.name.split(":").collect::<Vec<&str>>();
                match split.split_last() {
                    None => panic!("Could not get parent of {}", self.name),
                    Some((_, elements)) => Some(
                        elements
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join(":"),
                    ),
                }
            }
        }
    }
}
