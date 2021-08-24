use std::cell::RefCell;
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
/// currencies.add_alias('€'.to_string(), &eur);
/// assert_eq!(currencies.len(), 2, "List len should be 2");
/// assert_eq!(currencies.len_alias(), 4, "Alias len should be 4");
/// assert_eq!(currencies.get("eur").unwrap().as_ref(), &eur);
/// assert_eq!(currencies.get('€').unwrap().as_ref(), &eur);
///
///
/// assert_eq!(currencies.get("eur").unwrap(), currencies.get('€').unwrap(), "EUR and € should be the same");
///
/// ```
#[derive(Debug, Clone)]
pub struct Currency {
    name: String,
    origin: Origin,
    note: Option<String>,
    aliases: HashSet<String>,
    pub(crate) format: Option<String>,
    default: bool,
    pub(crate) display_format: RefCell<CurrencyDisplayFormat>,
}

/// Definition of how to display a currency
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyDisplayFormat {
    pub symbol_placement: CurrencySymbolPlacement,
    pub negative_amount_display: NegativeAmountDisplay,
    pub decimal_separator: Separator,
    pub digit_grouping: DigitGrouping,
    pub thousands_separator: Option<Separator>,
    pub precision: usize,
    pub max_decimals: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencySymbolPlacement {
    BeforeAmount,
    AfterAmount,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            display_format: RefCell::new(DEFAULT_DISPLAY_FORMAT),
        }
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
    pub fn set_precision(&self, precision: usize) {
        self.display_format.borrow_mut().set_precision(precision);
    }
    pub fn get_precision(&self) -> usize {
        self.display_format.borrow().precision
    }
    pub fn update_precision(&self, precision: usize) {
        self.display_format.borrow_mut().update_precision(precision);
    }
    pub fn set_format(&self, format: &CurrencyDisplayFormat) {
        let mut current_format = self.display_format.borrow_mut();
        current_format.symbol_placement = format.symbol_placement;
        current_format.negative_amount_display = format.negative_amount_display;
        current_format.decimal_separator = format.decimal_separator;
        current_format.digit_grouping = format.digit_grouping;
        current_format.thousands_separator = format.thousands_separator;
        current_format.precision = format.precision;
        current_format.max_decimals = format.max_decimals;
        // dbg!(format);
    }
}
impl CurrencyDisplayFormat {
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
    pub fn get_thousands_separator_str(&self) -> Option<char> {
        match self.thousands_separator {
            Some(Separator::Dot) => Some('.'),
            Some(Separator::Comma) => Some(','),
            Some(Separator::Space) => Some('\u{202f}'),
            Some(Separator::Other(x)) => Some(x),
            None => None,
        }
    }
    pub fn set_thousands_separator(&mut self, separator: char) {
        self.thousands_separator = match separator {
            '.' => Some(Separator::Dot),
            ',' => Some(Separator::Comma),
            '\u{202f}' => Some(Separator::Space),
            x => Some(Separator::Other(x)),
        };
    }
    pub fn get_digit_grouping(&self) -> DigitGrouping {
        self.digit_grouping
    }
    pub fn set_digit_grouping(&mut self, grouping: DigitGrouping) {
        self.digit_grouping = grouping
    }
    pub fn update_precision(&mut self, precision: usize) {
        if precision > self.precision {
            self.precision = precision;
        }
    }
    pub fn set_precision(&mut self, precision: usize) {
        self.max_decimals = Some(precision);
    }
    pub fn get_precision(&self) -> usize {
        match self.max_decimals {
            Some(precision) => precision,
            None => self.precision,
        }
    }
}

impl From<&str> for CurrencyDisplayFormat {
    /// Sets the format of the currency representation
    fn from(format: &str) -> Self {
        // dbg!(&format);
        let mut display_format = DEFAULT_DISPLAY_FORMAT;
        let mut parsed = GrammarParser::parse(Rule::currency_format, format)
            .unwrap()
            .next()
            .unwrap()
            .into_inner();

        let mut first = parsed.next().unwrap();
        let integer_format;

        if first.as_rule() == Rule::currency_format_positive {
            display_format.negative_amount_display = NegativeAmountDisplay::BeforeSymbolAndNumber;
            if first.as_str().starts_with('(') {
                display_format.negative_amount_display = NegativeAmountDisplay::Parentheses;
            }
            parsed = first.into_inner();
            first = parsed.next().unwrap();
        }
        match first.as_rule() {
            Rule::integer_part => {
                integer_format = Some(first);
                let rule = parsed.next().unwrap();
                if rule.as_rule() == Rule::space {
                    parsed.next();
                }
                parsed.next();
            }
            Rule::currency_string => {
                let mut rule = parsed.next().unwrap();
                if rule.as_rule() == Rule::space {
                    rule = parsed.next().unwrap();
                }
                integer_format = Some(rule);
                display_format.symbol_placement = CurrencySymbolPlacement::BeforeAmount;
                display_format.negative_amount_display =
                    NegativeAmountDisplay::BeforeSymbolAndNumber;
            }
            other => {
                panic!("Other: {:?}", other);
            }
        }

        // Get thousands separator and type of separation
        match integer_format {
            Some(x) => {
                let mut separators = vec![];
                let num_str = x.as_str();
                for sep in x.into_inner() {
                    separators.push((sep.as_str().chars().nth(0).unwrap(), sep.as_span().start()));
                }
                let len = separators.len();
                display_format.thousands_separator = None;
                if len == 0 {
                    display_format.digit_grouping = DigitGrouping::None;
                } else {
                    display_format.set_decimal_separator(separators[len - 1].0);
                    // Get the precision
                    display_format.max_decimals =
                        Some(num_str.split(separators[len - 1].0).last().unwrap().len());
                }
                if len > 1 {
                    display_format.set_thousands_separator(separators[len - 2].0);
                }
                if len > 2 {
                    let n = separators[len - 2].1 - separators[len - 3].1;
                    match n {
                        2 => display_format.digit_grouping = DigitGrouping::Indian,
                        3 => display_format.digit_grouping = DigitGrouping::Thousands,
                        _ => eprintln!("Wrong number format: {}", &format),
                    }
                }
            }
            None => display_format.digit_grouping = DigitGrouping::None,
        }
        display_format
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

const DEFAULT_DISPLAY_FORMAT: CurrencyDisplayFormat = CurrencyDisplayFormat {
    symbol_placement: CurrencySymbolPlacement::AfterAmount,
    negative_amount_display: NegativeAmountDisplay::BeforeSymbolAndNumber,
    decimal_separator: Separator::Dot,
    digit_grouping: DigitGrouping::Thousands,
    thousands_separator: Some(Separator::Comma),
    precision: 0,
    max_decimals: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_1() {
        let format_str = "-1.234,00 €";
        let format = CurrencyDisplayFormat::from(format_str);

        assert_eq!(format.get_precision(), 2);
        assert_eq!(format.get_thousands_separator_str(), Some('.'));
        assert_eq!(format.get_decimal_separator_str(), ',');
        assert_eq!(format.get_digit_grouping(), DigitGrouping::Thousands);
        assert_eq!(
            format.symbol_placement,
            CurrencySymbolPlacement::AfterAmount
        );
        assert_eq!(
            format.negative_amount_display,
            NegativeAmountDisplay::BeforeSymbolAndNumber
        );
    }
    #[test]
    fn format_2() {
        let format_str = "($1,234.00)";
        let format = CurrencyDisplayFormat::from(format_str);

        assert_eq!(format.get_precision(), 2);
        assert_eq!(format.get_thousands_separator_str(), Some(','));
        assert_eq!(format.get_decimal_separator_str(), '.');
        assert_eq!(format.get_digit_grouping(), DigitGrouping::Thousands);
        assert_eq!(
            format.symbol_placement,
            CurrencySymbolPlacement::BeforeAmount
        );
        assert_eq!(
            format.negative_amount_display,
            NegativeAmountDisplay::BeforeSymbolAndNumber
        );
    }
    #[test]
    fn format_3() {
        let format_str = "-1234,00 €";
        let format = CurrencyDisplayFormat::from(format_str);

        assert_eq!(format.get_precision(), 2);
        assert_eq!(format.get_thousands_separator_str(), None);
        assert_eq!(format.get_decimal_separator_str(), ',');
        // assert_eq!(format.get_digit_grouping(), DigitGrouping::Thousands);
        assert_eq!(
            format.symbol_placement,
            CurrencySymbolPlacement::AfterAmount
        );
        assert_eq!(
            format.negative_amount_display,
            NegativeAmountDisplay::BeforeSymbolAndNumber
        );
    }
}
