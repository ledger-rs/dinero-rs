use std::collections::{HashSet, HashMap};
use crate::Error;

/// Currency representation
///
/// A currency has a name and a list of aliases, we have to make sure that when two commodities are
/// created, they are the same, like so:
///
/// # Examples
/// ```rust
/// use dinero::ledger::{Currency, CurrencyList};
///
/// let usd1 = Currency::from("usd");
/// let usd2 = Currency::from("usd");
/// assert_eq!(usd1, usd2);
///
/// let eur1 = Currency::from("eur");
/// assert_ne!(eur1, usd1);
///
/// let mut eur2 =  Currency::from("eur");
/// assert_eq!(eur1, eur2);
///
/// let mut currencies = CurrencyList::new();
/// currencies.add_element(&eur1);
/// currencies.add_element(&eur2);
/// currencies.add_element(&usd1);
/// currencies.add_element(&usd2);
/// currencies.add_alias("euro", &eur1);
/// currencies.add_alias("€", &eur1);
/// assert_eq!(currencies.len(), 2);
/// assert_eq!(currencies.len_alias(), 4);
/// assert_eq!(currencies.get("eur").unwrap(), &eur1);
/// assert_eq!(currencies.get("eur").unwrap(), &eur2);
/// ```
#[derive(Debug, Copy, Clone, Eq, Hash)]
pub struct Currency<'a> {
    name: &'a str,
}

impl<'a> From<&'a str> for Currency<'a> {
    fn from(name: &'a str) -> Self {
        Currency { name }
    }
}

impl PartialEq for Currency<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct CurrencyList<'a> {
    aliases: HashMap<&'a str, &'a Currency<'a>>,
    list: HashSet<&'a Currency<'a>>,
}

impl<'a> CurrencyList<'a> {
    pub fn new() -> Self {
        let aliases: HashMap<&str, &Currency> = HashMap::new();
        let list: HashSet<&Currency> = HashSet::new();
        CurrencyList { aliases, list }
    }
    pub fn add_element(&mut self, currency: &'a Currency) {
        self.list.insert(currency);
        self.aliases.insert(currency.name, currency);
    }
    pub fn add_alias(&mut self, alias: &'a str, for_currency: &'a Currency) {
        self.aliases.insert(alias, for_currency);
        self.list.insert(for_currency);
    }
    pub fn element_in_list(self, element: &Currency) -> bool {
        match self.list.get(element) {
            None => false,
            Some(_) => true,
        }
    }
    pub fn get(&self, index: &str) -> Result<&Currency, Error> {
        match self.aliases.get(index) {
            None => Err(Error::CommodityNotInList),
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

impl<'a> From<&'a Currency<'a>> for CurrencyList<'a> {
    fn from(currency: &'a Currency<'a>) -> Self {
        let mut cl = CurrencyList::new();
        cl.add_element(currency);
        cl
    }
}
