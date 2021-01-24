use crate::ledger::{FromDirective, HasName, Origin, HasAliases};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::RandomState;

/// Currency representation
///
/// A currency has a name and a list of aliases, we have to make sure that when two commodities are
/// created, they are the same, like so:
///
/// # Examples
/// ```rust
/// use dinero::ledger::{Currency};
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
/// assert_eq!(currencies.get("eur").unwrap(), &eur);
/// assert_eq!(currencies.get("€").unwrap(), &eur);
///
///
/// assert_eq!(currencies.get("eur").unwrap(), currencies.get("€").unwrap(), "EUR and € should be the same");
///
/// ```
#[derive(Debug, Clone)]
pub struct Currency {
    pub (crate) name: String,
    pub (crate) origin: Origin,
    pub (crate) note: Option<String>,
    pub (crate) aliases: HashSet<String>,
    pub (crate) format: Option<String>,
    pub (crate) default: bool,
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
        Currency {
            name: String::from(name),
            origin: Origin::Other,
            note: None,
            aliases: Default::default(),
            format: None,
            default: false
        }
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