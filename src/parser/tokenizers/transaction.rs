use super::super::Rule;
use crate::models::{Cleared, Comment, PostingType, PriceType, Transaction, TransactionType};
use crate::parser::utils::{parse_date, parse_rational, parse_string};
use crate::parser::Tokenizer;
use chrono::NaiveDate;
use num::{rational::BigRational, BigInt};
use pest::iterators::Pair;

impl<'a> Tokenizer<'a> {
    /// Parses a transaction
    pub(crate) fn parse_transaction(&self, element: Pair<Rule>) -> Transaction<RawPosting> {
        let mut transaction = Transaction::<RawPosting>::new(match element.as_rule() {
            Rule::transaction => TransactionType::Real,
            Rule::automated_transaction => TransactionType::Automated,
            x => panic!("{:?}", x),
        });

        let mut parsed_transaction = element.into_inner();

        //
        // Parse the transaction head
        //
        let head = parsed_transaction.next().unwrap().into_inner();

        for part in head {
            match part.as_rule() {
                Rule::transaction_date => {
                    transaction.date = Some(parse_date(part.into_inner().next().unwrap()));
                }
                Rule::effective_date => {
                    transaction.effective_date =
                        Some(parse_date(part.into_inner().next().unwrap()));
                }
                Rule::status => {
                    transaction.cleared = match part.as_str() {
                        "!" => Cleared::NotCleared,
                        "*" => Cleared::Cleared,
                        x => panic!("Found '{}', expected '!' or '*'", x),
                    }
                }
                Rule::code => {
                    let mut code = part.as_str().chars();
                    code.next();
                    code.next_back();
                    transaction.code = Some(code.as_str().trim().to_string())
                }
                Rule::description | Rule::automated_description => {
                    transaction.description = parse_string(part).trim().to_string();
                }
                Rule::payee => {
                    transaction.payee = Some(parse_string(part).trim().to_string());
                }
                Rule::comment => {
                    // it can only be a comment
                    transaction.comments.push(Comment::from(parse_string(
                        part.into_inner().next().unwrap(),
                    )));
                }
                x => panic!("Expected amount, cost or balance {:?}", x),
            }
        }

        // Adjust the payee
        if transaction.payee.is_none() {
            if !transaction.description.is_empty() {
                transaction.payee = Some(transaction.description.clone());
            } else {
                transaction.payee = Some("[Unspecified payee]".to_string())
            }
        }

        //
        // Go for the indented part
        //
        for part in parsed_transaction {
            match part.as_rule() {
                Rule::posting | Rule::automated_posting => transaction
                    .postings
                    .borrow_mut()
                    .push(parse_posting(part, &transaction.payee, &transaction.date)),
                Rule::comment => transaction.comments.push(Comment::from(parse_string(
                    part.into_inner().next().unwrap(),
                ))),
                Rule::blank_line => {}
                x => panic!("{:?}", x),
            }
        }
        // dbg!(&transaction);
        transaction
    }
}

#[derive(Debug, Clone)]
pub struct RawPosting {
    pub account: String,
    pub date: Option<NaiveDate>,
    pub money_amount: Option<BigRational>,
    pub money_currency: Option<String>,
    pub money_format: Option<String>,
    pub cost_amount: Option<BigRational>,
    pub cost_currency: Option<String>,
    pub cost_format: Option<String>,
    pub cost_type: Option<PriceType>,
    pub balance_amount: Option<BigRational>,
    pub balance_currency: Option<String>,
    pub balance_format: Option<String>,
    pub comments: Vec<Comment>,
    pub amount_expr: Option<String>,
    pub kind: PostingType,
    pub payee: Option<String>,
}

impl RawPosting {
    fn new() -> RawPosting {
        RawPosting {
            account: String::new(),
            date: None,
            money_amount: None,
            money_currency: None,
            cost_amount: None,
            cost_currency: None,
            cost_type: None,
            balance_amount: None,
            balance_currency: None,
            comments: vec![],
            amount_expr: None,
            kind: PostingType::Real,
            payee: None,
            money_format: None,
            cost_format: None,
            balance_format: None,
        }
    }
}

/// Parses a posting
fn parse_posting(
    raw: Pair<Rule>,
    default_payee: &Option<String>,
    default_date: &Option<NaiveDate>,
) -> RawPosting {
    let mut posting = RawPosting::new();
    let elements = raw.into_inner();
    for part in elements {
        let rule = part.as_rule();
        match rule {
            Rule::posting_kind => {
                let kind = part.into_inner().next().unwrap();
                posting.kind = match kind.as_rule() {
                    Rule::virtual_no_balance => PostingType::Virtual,
                    Rule::virtual_balance => PostingType::VirtualMustBalance,
                    _ => PostingType::Real,
                };
                posting.account = kind.into_inner().next().unwrap().as_str().to_string();
            }
            Rule::amount | Rule::cost | Rule::balance => {
                let cost_type = if part.as_str().starts_with("@@") {
                    Some(PriceType::Total)
                } else {
                    Some(PriceType::PerUnit)
                };
                let mut inner = part.into_inner();
                let negative = inner.as_str().starts_with('-');
                let mut money = inner.next().unwrap().into_inner();
                let money_format = money.as_str().to_string();
                let amount: BigRational;
                let mut currency = None;
                match money.next() {
                    Some(money_part) => match money_part.as_rule() {
                        Rule::number => {
                            amount = parse_rational(money_part);
                            currency = Some(parse_string(money.next().unwrap()));
                        }
                        Rule::currency => {
                            currency = Some(parse_string(money_part));
                            if negative {
                                amount = -parse_rational(money.next().unwrap());
                            } else {
                                amount = parse_rational(money.next().unwrap());
                            }
                        }
                        _ => amount = BigRational::new(BigInt::from(0), BigInt::from(1)),
                    },
                    None => amount = BigRational::new(BigInt::from(0), BigInt::from(1)),
                }

                match rule {
                    Rule::amount => {
                        posting.money_amount = Some(amount);
                        posting.money_currency = currency;
                        posting.money_format = Some(money_format);
                    }
                    Rule::cost => {
                        posting.cost_amount = Some(amount);
                        posting.cost_currency = currency;
                        posting.cost_type = cost_type;
                        posting.cost_format = Some(money_format);
                    }
                    Rule::balance => {
                        posting.balance_amount = Some(amount);
                        posting.balance_currency = currency;
                        posting.balance_format = Some(money_format);
                    }
                    x => panic!("Expected amount, cost or balance {:?}", x),
                }
            }
            Rule::number => posting.amount_expr = Some(format!("({})", part.as_str())),
            Rule::value_expr => posting.amount_expr = Some(part.as_str().to_string()),
            Rule::comment => posting.comments.push(Comment::from(parse_string(
                part.into_inner().next().unwrap(),
            ))),
            Rule::blank_line => {}
            Rule::EOI => {}
            x => panic!("{:?}", x),
        }
    }
    for c in posting.comments.iter() {
        if let Some(payee) = c.get_payee_str() {
            posting.payee = Some(payee);
        }
        if let Some(date) = c.get_date() {
            posting.date = Some(date);
        }
    }
    if posting.payee.is_none() {
        posting.payee = default_payee.clone();
    }
    if posting.date.is_none() {
        posting.date = *default_date;
    }
    posting
}
#[cfg(test)]
mod tests {
    use structopt::StructOpt;

    use super::*;
    use crate::models::{Cleared, TransactionStatus};
    use crate::{parser::Tokenizer, CommonOpts};

    #[test]

    fn difficult_transaction_head() {
        let mut tokenizer = Tokenizer::from(
            "2022-05-13 ! (8760) Intereses | EstateGuru
            EstateGuru               1.06 EUR
            Ingresos:Rendimientos
            "
            .to_string(),
        );

        let parsed = tokenizer.tokenize(&CommonOpts::from_iter(["", "-f", ""].iter()));
        let transaction = &parsed.transactions[0];
        assert_eq!(transaction.cleared, Cleared::NotCleared);
        assert_eq!(transaction.status, TransactionStatus::NotChecked);
        assert_eq!(transaction.code, Some(String::from("8760")));
        assert_eq!(transaction.payee, Some(String::from("EstateGuru")));
        assert_eq!(transaction.description, String::from("Intereses"));
        for p in transaction.postings.borrow().iter() {
            assert_eq!(p.kind, PostingType::Real);
        }
    }
}
