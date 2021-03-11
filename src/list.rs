use regex::Regex;
use std::collections::hash_map::{Iter, Values};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use crate::models::{FromDirective, HasAliases, HasName};
use crate::LedgerError;
/// A generic container with some search capabilities
///
/// This structure is used to hold master elements of the ledger than can be aliases such as
/// commodities or accounts
///
/// It provides methods for:
/// - Adding new elements to the list
/// - Adding new aliases to existing elements
/// - Retrieving elements
/// - Retrieving elements with a regular expression
#[derive(Debug, Clone)]
pub struct List<T> {
    aliases: HashMap<String, String>,
    list: HashMap<String, Rc<T>>,
    matches: HashMap<String, Option<String>>,
}

impl<'a, T: Eq + Hash + HasName + Clone + FromDirective + HasAliases + Debug> List<T> {
    pub fn new() -> Self {
        let aliases: HashMap<String, String> = HashMap::new();
        let list: HashMap<String, Rc<T>> = HashMap::new();
        let matches: HashMap<String, Option<String>> = HashMap::new();
        List {
            aliases,
            list,
            matches,
        }
    }

    /// Inserts an ```element``` in the list
    pub fn insert(&mut self, element: T) {
        let found = self.list.get(&element.get_name().to_lowercase());
        match found {
            Some(_) => eprintln!("Duplicate element: {:?}", element), // do nothing
            None => {
                // Change the name which will be used as key to lowercase
                let name = element.get_name().to_string().to_lowercase();
                for alias in element.get_aliases().iter() {
                    self.aliases.insert(alias.to_lowercase(), name.clone());
                }
                self.list.insert(name.clone(), Rc::new(element));
            }
        }
    }
    /// Add an alias
    pub fn add_alias(&mut self, alias: String, for_element: &'a T) {
        let element = self.aliases.get(&alias.to_lowercase());
        match element {
            Some(x) => panic!(
                "Repeated alias {} for {} and {}",
                alias,
                for_element.get_name(),
                x
            ),
            None => {
                self.aliases
                    .insert(alias.to_lowercase(), for_element.get_name().to_lowercase());
            }
        }
        ()
    }

    pub fn get(&self, index: &str) -> Result<&Rc<T>, LedgerError> {
        match self.list.get(&index.to_lowercase()) {
            None => match self.aliases.get(&index.to_lowercase()) {
                None => Err(LedgerError::AliasNotInList(format!(
                    "{} {:?} not found",
                    std::any::type_name::<T>(),
                    index
                ))),
                Some(x) => Ok(self.list.get(x).unwrap()),
            },
            Some(x) => Ok(x),
        }
    }
    /// Gets an element from the regex
    pub fn get_regex(&mut self, regex: Regex) -> Option<&Rc<T>> {
        let alias = self.matches.get(regex.as_str());
        match alias {
            Some(x) => match x {
                Some(alias) => Some(self.get(alias).unwrap()),
                None => None,
            },
            None => {
                // cache miss
                for (_alias, value) in self.list.iter() {
                    if regex.is_match(value.get_name()) {
                        self.matches
                            .insert(regex.as_str().to_string(), Some(_alias.clone()));
                        return Some(value);
                    }
                }
                for (alias, value) in self.aliases.iter() {
                    if regex.is_match(alias) {
                        self.matches
                            .insert(regex.as_str().to_string(), Some(value.clone()));
                        return self.list.get(value);
                    }
                }
                self.matches.insert(regex.as_str().to_string(), None);
                None
            }
        }
        // // Try the list

        // None
    }

    pub fn iter(&self) -> Iter<'_, String, Rc<T>> {
        self.list.iter()
    }
    pub fn values(&self) -> Values<'_, String, Rc<T>> {
        self.list.values()
    }
    pub fn len(&self) -> usize {
        self.list.len()
    }
    pub fn len_alias(&self) -> usize {
        self.aliases.len() + self.len()
    }
}

impl<T: Clone> List<T> {
    pub fn append(&mut self, other: &List<T>) {
        self.list.extend(other.to_owned().list.into_iter());
        self.aliases.extend(other.to_owned().aliases.into_iter());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Payee;
    use regex::Regex;
    #[test]
    fn list() {
        let name = "ACME Inc.";
        let payee = Payee::from(name);
        let mut list: List<Payee> = List::new();
        list.insert(payee.clone());

        // Get ACME from the list, using a regex
        let pattern = Regex::new("ACME").unwrap();
        let retrieved = list.get_regex(pattern);

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get_name(), "ACME Inc.");
        assert_eq!(list.len_alias(), 1);

        // Now add and alias
        list.add_alias("ACME is awesome".to_string(), &payee);
        assert_eq!(list.len_alias(), 2);

        // Retrieve an element that is not in the list
        assert!(list.get_regex(Regex::new("Warner").unwrap()).is_none());
        assert!(list.get("Warner").is_err());
        assert!(list.get_regex(Regex::new("awesome").unwrap()).is_some());
    }
    #[test]
    #[should_panic]
    fn list_repeated_alias() {
        let mut list: List<Payee> = List::new();
        list.insert(Payee::from("ACME"));
        for _ in 0..2 {
            let retrieved = list.get("ACME").unwrap().clone();
            list.add_alias("ACME, Inc.".to_string(), &retrieved)
        }
    }
}
