use crate::ledger::{Origin, FromDirective, HasName, Balance, Money};
use std::hash::{Hash, Hasher};
use num::rational::Rational64;
use crate::List;
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Account<'a> {
    name: String,
    origin: Origin,
    parent: Option<&'a Account<'a>>,
}

impl Display for Account<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Account<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Account<'_> {}

impl Hash for Account<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<String> for Account<'_> {
    fn from(name: String) -> Self {
        Account {
            name,
            origin: Origin::Other,
            parent: None,
        }
    }
}

impl FromDirective for Account<'_> {
    fn is_from_directive(&self) -> bool {
        match self.origin {
            Origin::FromDirective => true,
            _ => false
        }
    }
}

impl HasName for Account<'_> {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl<'a> Account<'a> {
    /// Depth of the account, useful for filters and other
    pub fn depth(&self) -> usize {
        self.name.chars().filter(|c| *c == ':').collect::<Vec<char>>().len() + 1
    }

    pub fn set_parent(&mut self, parent: &'a Account<'a>) {
        self.parent = Some(parent)
    }
    pub fn get_parent(&self) -> Option<&'a Account<'a>> {
        self.parent
    }
    /// Parent name
    ///
    /// Returns the name of the parent account should have
    /// ```rust
    /// use dinero::ledger::Account;
    /// let mut account = Account::from("Expenses:Groceries".to_string());
    /// let mut parent = account.parent_name();
    /// assert_eq!(parent, Some("Expenses".to_string()));
    ///
    /// account = Account::from("Expenses:Groceries:Supermarket".to_string());
    /// parent = account.parent_name();
    /// assert_eq!(parent, Some("Expenses:Groceries".to_string()));
    ///
    /// account = Account::from("Expenses".to_string());
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
                    Some((_, elements)) => {
                        Some(elements.iter()
                            .map(|x| x.to_string()).collect::<Vec<String>>()
                            .join(":"))
                    }
                }
            }
        }
    }
}

impl<'a> List<'a, Account<'a>> {
    /// Given a list of accounts it builds the account tree structure
    pub fn to_account_tree(&'a mut self) {
        let mut finished = false;
        while !finished {
            let mut new_accounts = Vec::<Account>::new();
            for (k, a) in self.list.clone().iter_mut() {
                if a.depth() > 1 {
                    let parent_name = a.parent_name().unwrap();
                    match self.list.get(&parent_name) {
                        None => {
                            new_accounts.push(Account::from(parent_name.to_string()));
                        }
                        Some(p) => a.parent = Some(p),
                    }
                }
            }
            for acc in new_accounts {
                self.push(acc);
            }
            finished = self.list.values()
                .filter(|x| x.parent.is_none() & (x.depth() > 1))
                .collect::<Vec<&Account>>().len() == 0;
        }
    }
}
