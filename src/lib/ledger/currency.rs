use crate::ErrorType;
use crate::list::HasName;

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
impl HasName for Currency<'_> {
    fn get_name(&self) -> &str {
        self.name
    }
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
