use crate::models::{FromDirective, HasAliases, HasName, Origin};
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Payee {
    pub name: String,
    pub note: Option<String>,
    pub alias: HashSet<String>,
    pub(crate) origin: Origin,
}

impl Eq for Payee {}

impl PartialEq for Payee {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl HasName for Payee {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl HasAliases for Payee {
    fn get_aliases(&self) -> &HashSet<String, RandomState> {
        &self.alias
    }
}

impl FromDirective for Payee {
    fn is_from_directive(&self) -> bool {
        match self.origin {
            Origin::FromDirective => true,
            _ => false,
        }
    }
}

impl Hash for Payee {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
