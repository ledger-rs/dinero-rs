use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use super::super::parser::{GrammarParser, Rule};
use crate::models::{FromDirective, HasAliases, HasName, Origin};
use pest::Parser;
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
    pub symbol_placement: CurrencySymbolPlacement,
    pub negative_amount_display: NegativeAmountDisplay,
    decimal_separator: Separator,
    digit_grouping: DigitGrouping,
    thousands_separator: Separator,
}

#[derive(Clone, Copy, Debug)]
pub enum CurrencySymbolPlacement {
    BeforeAmount,
    AfterAmount,
}

#[derive(Clone, Copy, Debug)]
pub enum NegativeAmountDisplay {
    BeforeSymbolAndNumber,      // UK   -£127.54   or Spain  -127,54 €
    BeforeNumberBehindCurrency, // Denmark	kr-127,54
    AfterNumber,                // Netherlands € 127,54-
    Parentheses,                // US	($127.54)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigitGrouping {
    Thousands,
    Indian,
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum Separator {
    Dot,
    Comma,
    Space,
    Other(char),
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
            symbol_placement: CurrencySymbolPlacement::AfterAmount,
            negative_amount_display: NegativeAmountDisplay::BeforeSymbolAndNumber,
            decimal_separator: Separator::Comma,
            digit_grouping: DigitGrouping::Thousands,
            thousands_separator: Separator::Space,
        }
    }
    pub fn get_precision(&self) -> Option<usize> {
        self.precision
    }
    pub fn get_decimal_separator_str(&self) -> char {
        match self.decimal_separator {
            Separator::Dot => '.',
            Separator::Comma => ',',
            Separator::Space => '\u{202f}',
            Separator::Other(x) => x,
        }
    }
    pub fn set_decimal_separator(&mut self, separator: char) {
        self.decimal_separator = match separator {
            '.' => Separator::Dot,
            ',' => Separator::Comma,
            x => Separator::Other(x),
        };
    }
    pub fn get_thousands_separator_str(&self) -> char {
        match self.thousands_separator {
            Separator::Dot => '.',
            Separator::Comma => ',',
            Separator::Space => '\u{202f}',
            Separator::Other(x) => x,
        }
    }
    pub fn set_thousands_separator(&mut self, separator: char) {
        self.thousands_separator = match separator {
            '.' => Separator::Dot,
            ',' => Separator::Comma,
            '\u{202f}' => Separator::Space,
            x => Separator::Other(x),
        };
    }
    pub fn get_digit_grouping(&self) -> DigitGrouping {
        self.digit_grouping
    }
    pub fn set_digit_grouping(&mut self, grouping: DigitGrouping) {
        self.digit_grouping = grouping
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
    /// Sets the format of the currency representation
    pub fn set_format(&mut self, format: String) {
        let mut parsed = GrammarParser::parse(Rule::currency_format, format.as_str())
            .unwrap()
            .next()
            .unwrap()
            .into_inner();

        let mut first = parsed.next().unwrap();
        let mut integer_format = None;
        let mut currency_format = None;
        
        if first.as_rule() == Rule::currency_format_positive {
            if first.as_str().starts_with("-") {
                self.negative_amount_display = NegativeAmountDisplay::BeforeSymbolAndNumber;
            } else if first.as_str().starts_with("(") {
                self.negative_amount_display = NegativeAmountDisplay::Parentheses;
            }
            parsed = first.into_inner();
            first = parsed.next().unwrap();
        }
        match first.as_rule() {
            Rule::integer_part => {
                integer_format = Some(first);
                let mut rule = parsed.next().unwrap();
                if rule.as_rule() == Rule::space {
                    // TODO ??? decimal_format = Some(rule);
                    rule = parsed.next().unwrap();
                }
                currency_format = parsed.next();
            }
            Rule::currency_string => {
                currency_format = Some(first);
                let mut rule = parsed.next().unwrap();
                if rule.as_rule() == Rule::space {
                    // TODO ??? decimal_format = Some(rule);
                    rule = parsed.next().unwrap();
                }
                integer_format = Some(rule);
                self.negative_amount_display = NegativeAmountDisplay::AfterNumber;
            }
            other => {
                panic!("Other: {:?}", other);
            }
        }

        // Get thousands separator and type of separation
        match integer_format {
            Some(x) => {
                let start = x.as_span().start();
                let mut separators = vec![];
                let num_chars = x.as_str().len();
                for sep in x.into_inner() {
                    separators.push((sep.as_str().chars().nth(0).unwrap(), sep.as_span().start()));
                }
                let len = separators.len();
                if len == 0 {
                    self.set_precision(0);
                    self.digit_grouping = DigitGrouping::None;
                } else {
                    self.set_precision(num_chars - separators[len - 1].1 + start - 1);
                    self.set_decimal_separator(separators[len - 1].0);
                }
                if len > 1 {
                    self.set_thousands_separator(separators[len - 2].0);
                }
                if len > 2 {
                    let n = separators[len - 2].1 - separators[len - 3].1;
                    match n {
                        2 => self.digit_grouping = DigitGrouping::Indian,
                        3 => self.digit_grouping = DigitGrouping::Thousands,
                        _ => eprintln!("Wrong number format: {}", &format),
                    }
                }
            }
            None => self.digit_grouping = DigitGrouping::None,
        }

        self.format = Some(format);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_1() {
        let format = "-1.234,00 €";
        let mut currency = Currency::from_directive(format.to_string());
        currency.set_format(format.to_string());

        assert_eq!(currency.get_precision(), Some(2));
        assert_eq!(currency.get_thousands_separator_str(), '.');
        assert_eq!(currency.get_decimal_separator_str(), ',');
    }
    #[test]
    fn format_2() {
        let format = "($1,234.00)";
        let mut currency = Currency::from_directive(format.to_string());
        currency.set_format(format.to_string());

        assert_eq!(currency.get_precision(), Some(2));
        assert_eq!(currency.get_thousands_separator_str(), ',');
        assert_eq!(currency.get_decimal_separator_str(), '.');
    }
}
