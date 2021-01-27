use std::collections::HashMap;
use std::hash::Hash;

use colored::Colorize;

use crate::models::{FromDirective, HasAliases, HasName};
use crate::{Error, LedgerError, ParserError};
use std::collections::hash_map::{Iter, RandomState, Values};
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct List<T> {
    aliases: HashMap<String, String>,
    list: HashMap<String, Rc<T>>,
}

impl<T> Into<HashMap<String, Rc<T>>> for List<T> {
    fn into(self) -> HashMap<String, Rc<T>, RandomState> {
        self.list
    }
}

impl<'a, T: Eq + Hash + HasName + Clone + FromDirective + HasAliases + Debug> List<T> {
    pub fn new() -> Self {
        let aliases: HashMap<String, String> = HashMap::new();
        let list: HashMap<String, Rc<T>> = HashMap::new();
        List { aliases, list }
    }

    pub fn insert(&mut self, element: T) {
        let found = self.list.get(element.get_name());
        match found {
            Some(_) => (), // do nothing
            None => {
                let name = element.get_name().to_string();
                for alias in element.get_aliases().iter() {
                    self.aliases.insert(alias.clone(), name.clone());
                }
                self.list.insert(name.clone(), Rc::new(element));
            }
        }
    }
    pub fn add_alias(&mut self, alias: String, for_element: &'a T) {
        let element = self.aliases.get(&alias);
        match element {
            Some(x) => panic!(
                "Repeated alias {} for {} and {}",
                alias,
                for_element.get_name(),
                x
            ),
            None => {
                self.aliases
                    .insert(alias, for_element.get_name().to_string());
            }
        }
        ()
    }

    pub fn element_in_list(&self, element: &T) -> bool {
        match self.aliases.get(element.get_name()) {
            None => false,
            Some(_) => true,
        }
    }
    pub fn get(&self, index: &str) -> Result<&Rc<T>, LedgerError> {
        match self.list.get(index) {
            None => match self.aliases.get(index) {
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
