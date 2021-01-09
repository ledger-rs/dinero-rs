use std::collections::{HashMap, HashSet};
use crate::ErrorType;
use std::hash::Hash;


pub struct List<'a, T> {
    aliases: HashMap<&'a str, &'a T>,
    list: HashSet<&'a T>,
}

pub trait HasName {
    fn get_name(&self) -> &str;
}

impl<'a, T: Eq + Hash + HasName> List<'a, T> {
    pub fn new() -> Self {
        let aliases: HashMap<&str, &T> = HashMap::new();
        let list: HashSet<&T> = HashSet::new();
        List { aliases, list }
    }
    pub fn add_element(&mut self, T: &'a T) {
        self.list.insert(T);
        self.aliases.insert(T.get_name(), T);
    }
    pub fn add_alias(&mut self, alias: &'a str, for_element: &'a T) {
        self.aliases.insert(alias, for_element);
        self.list.insert(for_element);
    }
    pub fn element_in_list(self, element: &T) -> bool {
        match self.list.get(element) {
            None => false,
            Some(_) => true,
        }
    }
    pub fn get(&self, index: &str) -> Result<&T, ErrorType> {
        match self.aliases.get(index) {
            None => Err(ErrorType::CommodityNotInList),
            Some(x) => Ok(x)
        }
    }
    pub fn len(&self) -> usize {
        self.list.len()
    }
    pub fn len_alias(&self) -> usize {
        self.aliases.len()
    }
}

impl<'a, T: HasName + Eq + Hash> From<&'a T> for List<'a, T> {
    fn from(element: &'a T) -> Self {
        let mut l = List::<T>::new();
        l.add_element(element);
        l
    }
}
