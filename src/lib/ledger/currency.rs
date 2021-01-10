use crate::ErrorType;
use crate::ledger::{Origin, HasName, FromDirective};
use std::hash::{Hash, Hasher};

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
/// currencies.add_element(&eur1);
/// currencies.add_element(&eur2);
/// currencies.add_element(&usd1);
/// currencies.add_element(&usd2);
/// assert_eq!(currencies.len_alias(), 2, "Alias len should be 2");
/// currencies.add_alias("euro".to_string(), &eur1);
/// assert_eq!(currencies.len_alias(), 3, "Alias len should be 3");
/// currencies.add_alias("€".to_string(), &eur1);
/// assert_eq!(currencies.len(), 2, "List len should be 2");
/// assert_eq!(currencies.len_alias(), 4, "Alias len should be 4");
/// assert_eq!(currencies.get("eur").unwrap(), &eur1);
/// assert_eq!(currencies.get("€").unwrap(), &eur2);
///
/// currencies.push(eur1);
/// currencies.push(eur2);
/// assert_eq!(currencies.get("eur").unwrap(), currencies.get("€").unwrap(), "EUR and € should be the same");
///
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Currency<'a> {
    name: &'a str,
    origin: Origin,
}

impl HasName for Currency<'_> {
    fn get_name(&self) -> &str {
        self.name
    }
}

impl<'a> From<&'a str> for Currency<'a> {
    fn from(name: &'a str) -> Self {
        Currency { name, origin: Origin::Other }
    }
}

impl FromDirective for Currency<'_> {
    fn is_from_directive(&self) -> bool {
        match self.origin {
            Origin::FromDirective => true,
            _ => false,
        }
    }
}

impl PartialEq for Currency<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Currency<'_> {}

impl Hash for Currency<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
