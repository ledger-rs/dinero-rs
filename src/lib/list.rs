use std::collections::HashMap;
use std::hash::Hash;

use colored::Colorize;

use crate::ledger::{FromDirective, HasName, HasAliases};
use crate::{Error, ErrorType};
use std::collections::hash_map::{Iter, Values};

#[derive(Debug, Clone)]
pub struct List<T> {
    aliases: HashMap<String, String>,
    list: HashMap<String, T>,
}

impl<'a, T: Eq + Hash + HasName + Clone + FromDirective + HasAliases> List<T> {
    pub fn new() -> Self {
        let aliases: HashMap<String, String> = HashMap::new();
        let list: HashMap<String, T> = HashMap::new();
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
                self.list.insert(name.clone(), element);
            }
        }
    }
    pub fn add_alias(&mut self, alias: String, for_element: &'a T) {
        let element = self.aliases.get(&alias);
        match element {
            Some(x) => panic!("Repeated alias {} for {} and {}", alias, for_element.get_name(), x),
            None => {
                self.aliases.insert(alias, for_element.get_name().to_string());
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
    pub fn get(&self, index: &str) -> Result<&T, Error> {
        match self.list.get(index) {
            None => match self.aliases.get(index) {
                None => Err(Error {
                    error_type: ErrorType::CommodityNotInList,
                    message: vec![format!("{:?} not found", index).as_str().bold()],
                }),
                Some(x) => Ok(self.list.get(x).unwrap()),
            },
            Some(x) => Ok(x),
        }
    }

    pub fn iter(&self) -> Iter<'_, String, T> {
        self.list.iter()
    }
    pub fn values(&self) -> Values<'_, String, T> {
        self.list.values()
    }
    pub fn len(&self) -> usize {
        self.list.len()
    }
    pub fn len_alias(&self) -> usize {
        self.aliases.len() + self.len()
    }
}
