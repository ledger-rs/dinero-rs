use super::super::Rule;
use crate::models::{Comment, PostingType, PriceType, Transaction, TransactionType};
use crate::parser::utils::{parse_date, parse_rational, parse_string};
use crate::parser::Tokenizer;
use num::{BigInt, rational::BigRational};
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
        let mut head = parsed_transaction.next().unwrap().into_inner();

        while let Some(part) = head.next() {
            match part.as_rule() {
                Rule::transaction_date => {
                    transaction.date = Some(parse_date(part.into_inner().next().unwrap()));
                }
                Rule::effective_date => {
                    transaction.effective_date =
                        Some(parse_date(part.into_inner().next().unwrap()));
                }
                Rule::status => {}
                Rule::code => {}
                Rule::description => {
                    transaction.description = parse_string(part).trim().to_string();
                }
                Rule::payee => {
                    transaction.payee = Some(parse_string(part).trim().to_string());
                }
                Rule::comment => {
                    // it can only be a comment
                    transaction.comments.push(Comment {
                        comment: parse_string(part),
                    });
                }
                x => panic!("Expected amount, cost or balance {:?}", x),
            }
        }

        // Adjust the payee
        if transaction.payee.is_none() {
            if transaction.description.len() > 0 {
                transaction.payee = Some(transaction.description.clone());
            } else {
                transaction.payee = Some("[Unspecified payee]".to_string())
            }
        }

        //
        // Go for the indented part
        //
        while let Some(part) = parsed_transaction.next() {
            match part.as_rule() {
                Rule::posting | Rule::automated_posting => transaction
                    .postings
                    .borrow_mut()
                    .push(parse_posting(part, &transaction.payee)),
                Rule::comment => transaction.comments.push(Comment {
                    comment: parse_string(part),
                }),
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
    pub money_amount: Option<BigRational>,
    pub money_currency: Option<String>,
    pub cost_amount: Option<BigRational>,
    pub cost_currency: Option<String>,
    pub cost_type: Option<PriceType>,
    pub balance_amount: Option<BigRational>,
    pub balance_currency: Option<String>,
    pub comments: Vec<Comment>,
    pub amount_expr: Option<String>,
    pub kind: PostingType,
    pub payee: Option<String>,
}

impl RawPosting {
    fn new() -> RawPosting {
        RawPosting {
            account: String::new(),
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
        }
    }
}

/// Parses a posting
fn parse_posting(raw: Pair<Rule>, default_payee: &Option<String>) -> RawPosting {
    let mut posting = RawPosting::new();
    let mut elements = raw.into_inner();
    while let Some(part) = elements.next() {
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
                let mut money = part.into_inner().next().unwrap().into_inner();
                let amount: BigRational;
                let mut currency=None;
                let money_part = money.next().unwrap();
                match money_part.as_rule() {
                    Rule::number => {
                        amount = parse_rational(money_part);
                        currency = Some(parse_string(money.next().unwrap()));
                    }
                    Rule::currency => {
                        currency = Some(parse_string(money_part));
                        amount = parse_rational(money.next().unwrap());
                    }
                    _ => amount = BigRational::new(BigInt::from(0),BigInt::from(1)),
                }

                match rule {
                    Rule::amount => {
                        posting.money_amount = Some(amount);
                        posting.money_currency = currency;
                    }
                    Rule::cost => {
                        posting.cost_amount = Some(amount);
                        posting.cost_currency = currency;
                        posting.cost_type = cost_type;
                    }
                    Rule::balance => {
                        posting.balance_amount = Some(amount);
                        posting.balance_currency = currency;
                    }
                    x => panic!("Expected amount, cost or balance {:?}", x),
                }
            }
            Rule::number => posting.amount_expr = Some(format!("({})", part.as_str())),
            Rule::value_expr => posting.amount_expr = Some(part.as_str().to_string()),
            Rule::comment => posting.comments.push(Comment {
                comment: parse_string(part),
            }),
            x => panic!("{:?}", x),
        }
    }
    posting.payee = default_payee.clone();
    posting
}
