use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::models::{FromDirective, HasAliases, HasName, Origin};
use std::cmp::Ordering;

/// Currency representation
///
/// A currency (or commodity) has a name and a list of aliases, we have to make sure that when two commodities are
/// created, they are the same, like so:
///
/// # Examples
/// ```rust
/// use dinero::models::{Currency};
/// use dinero::List;
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
/// let mut currencies = List::<Currency>::new();
/// currencies.insert(eur1);
/// currencies.insert(eur2);
/// currencies.insert(usd1);
/// currencies.insert(usd2);
/// assert_eq!(currencies.len_alias(), 2, "Alias len should be 2");
/// let eur = Currency::from("eur");
/// currencies.add_alias("euro".to_string(), &eur);
/// assert_eq!(currencies.len_alias(), 3, "Alias len should be 3");
/// currencies.add_alias("€".to_string(), &eur);
/// assert_eq!(currencies.len(), 2, "List len should be 2");
/// assert_eq!(currencies.len_alias(), 4, "Alias len should be 4");
/// assert_eq!(currencies.get("eur").unwrap().as_ref(), &eur);
/// assert_eq!(currencies.get("€").unwrap().as_ref(), &eur);
///
///
/// assert_eq!(currencies.get("eur").unwrap(), currencies.get("€").unwrap(), "EUR and € should be the same");
///
/// ```
#[derive(Debug, Clone)]
pub struct Currency {
    name: String,
    origin: Origin,
    note: Option<String>,
    aliases: HashSet<String>,
    format: Option<String>,
    default: bool,
    precision: Option<usize>,
}
impl Currency {
    pub fn from_directive(name: String) -> Self {
        Currency {
            name,
            origin: Origin::FromDirective,
            note: None,
            aliases: HashSet::new(),
            format: None,
            default: false,
            precision: None,
        }
    }
    pub fn get_precision(&self) -> usize {
        match self.precision {
            Some(p) => p,
            None => 2,
        }
    }
    pub fn set_precision(&mut self, precision: usize) {
        self.precision = Some(precision);
    }
    pub fn set_note(&mut self, note: String) {
        self.note = Some(note);
    }
    pub fn set_default(&mut self) {
        self.default = true;
    }
    pub fn set_aliases(&mut self, aliases: HashSet<String>) {
        self.aliases = aliases;
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl HasName for Currency {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}
impl HasAliases for Currency {
    fn get_aliases(&self) -> &HashSet<String> {
        &self.aliases
    }
}
impl<'a> From<&'a str> for Currency {
    fn from(name: &'a str) -> Self {
        let mut cur = Currency::from_directive(name.to_string());
        cur.origin = Origin::Other;
        cur
    }
}

impl FromDirective for Currency {
    fn is_from_directive(&self) -> bool {
        match self.origin {
            Origin::FromDirective => true,
            _ => false,
        }
    }
}

impl PartialEq for Currency {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Currency {}

impl Hash for Currency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Ord for Currency {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
impl PartialOrd for Currency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
