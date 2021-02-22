use crate::app;
use crate::models::{Account, Currency, Money, Payee, Posting, Transaction};
use crate::pest::Parser;
use crate::List;
use chrono::NaiveDate;
use num::{abs, BigInt, BigRational};
use regex::Regex;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "grammar/expressions.pest"]
pub struct ValueExpressionParser;

pub fn eval_expression(
    expression: &str,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
) -> EvalResult {
    let parsed = ValueExpressionParser::parse(Rule::value_expr, expression)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    // Build the abstract syntax tree
    let root = build_ast_from_expr(parsed);

    eval(&root, posting, transaction, commodities)
}

pub fn eval_value_expression(
    expression: &str,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
) -> Money {
    match eval_expression(expression, posting, transaction, commodities) {
        EvalResult::Number(n) => posting.amount.clone().unwrap() * n,
        EvalResult::Money(m) => m,
        _ => panic!("Should be money"),
    }
}

#[derive(Clone)]
pub enum Node {
    Amount,
    Account,
    Payee,
    Note,
    Date,
    Number(BigRational),
    Money {
        currency: String,
        amount: BigRational,
    },
    UnaryExpr {
        op: Unary,
        child: Box<Node>,
    },
    BinaryExpr {
        op: Binary,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Regex(Regex),
    String(String),
}

#[derive(Debug)]
pub enum EvalResult {
    Number(BigRational),
    Money(Money),
    Boolean(bool),
    Account(Rc<Account>),
    Payee(Rc<Payee>),
    Regex(Regex),
    String(Option<String>),
    Date(NaiveDate),
}

pub fn eval(
    node: &Node,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
) -> EvalResult {
    match node {
        Node::Amount => EvalResult::Money(posting.amount.clone().unwrap()),
        Node::Account => EvalResult::Account(posting.account.clone()),
        Node::Payee => EvalResult::Payee(posting.payee.clone()),
        Node::Note => EvalResult::String(transaction.note.clone()),
        Node::Date => EvalResult::Date(transaction.date.clone().unwrap()),
        Node::Regex(r) => EvalResult::Regex(r.clone()),
        Node::String(r) => EvalResult::String(Some(r.clone())),
        Node::Number(n) => EvalResult::Number(n.clone()),
        Node::Money { currency, amount } => {
            let cur = match commodities.get(&currency) {
                Ok(c) => c.clone(),
                Err(_) => {
                    let c = Currency::from(currency.as_str());
                    commodities.insert(c.clone());
                    Rc::new(c)
                }
            };
            EvalResult::Money(Money::from((cur.clone(), amount.clone())))
        }
        Node::UnaryExpr { op, child } => {
            let res = eval(child, posting, transaction, commodities);
            match op {
                Unary::Not => match res {
                    EvalResult::Boolean(b) => EvalResult::Boolean(!b),
                    x => panic!("Can't do neg of {:?}", x),
                },
                Unary::Any => {
                    let mut res = false;
                    for p in transaction.postings_iter() {
                        if let EvalResult::Boolean(b) = eval(child, p, transaction, commodities) {
                            if b {
                                res = true;
                                break;
                            }
                        } else {
                            panic!("Should evaluate to boolean")
                        }
                    }
                    EvalResult::Boolean(res)
                }
                Unary::Neg => match res {
                    EvalResult::Number(n) => EvalResult::Number(-n),
                    EvalResult::Money(money) => EvalResult::Money(-money),
                    EvalResult::Boolean(b) => EvalResult::Boolean(!b),
                    x => panic!("Can't do neg of {:?}", x),
                },
                Unary::Abs => match res {
                    EvalResult::Number(n) => EvalResult::Number(abs(n)),
                    EvalResult::Money(money) => EvalResult::Money(match money {
                        Money::Zero => Money::Zero,
                        Money::Money { amount, currency } => Money::from((currency, abs(amount))),
                    }),
                    EvalResult::Boolean(_b) => panic!("Can't do abs of boolean"),
                    x => panic!("Can't do abs of {:?}", x),
                },
                Unary::HasTag => match res {
                    EvalResult::Regex(r) => EvalResult::Boolean(posting.has_tag(r)),
                    x => panic!("Expected regex. Found {:?}", x),
                },
                Unary::Tag => match res {
                    EvalResult::Regex(r) => EvalResult::String(posting.get_tag(r)),
                    EvalResult::String(r) => EvalResult::String(posting.get_exact_tag(r.unwrap())),
                    x => panic!("Expected regex. Found {:?}", x),
                },
                Unary::ToDate => match res {
                    EvalResult::String(r) => {
                        EvalResult::Date(app::date_parser(r.unwrap().as_str()).unwrap())
                    }
                    x => panic!("Expected String. Found {:?}", x),
                },
            }
        }
        Node::BinaryExpr { op, lhs, rhs } => {
            let left = eval(lhs, posting, transaction, commodities);
            let right = eval(rhs, posting, transaction, commodities);
            match op {
                Binary::Eq => match right {
                    EvalResult::Regex(rhs) => match left {
                        EvalResult::Account(lhs) => EvalResult::Boolean(lhs.is_match(rhs)),
                        EvalResult::Payee(lhs) => EvalResult::Boolean(lhs.is_match(rhs)),
                        EvalResult::String(lhs) => match lhs {
                            Some(lhs) => EvalResult::Boolean(rhs.is_match(lhs.as_str())),
                            None => EvalResult::Boolean(false),
                        },
                        x => panic!("Found {:?}", x),
                    },
                    EvalResult::Money(rhs) => match left {
                        EvalResult::Money(lhs) => EvalResult::Boolean(lhs == rhs),
                        EvalResult::Number(lhs) => EvalResult::Boolean(lhs == rhs.get_amount()),

                        unknown => panic!("Don't know what to do with {:?}", unknown),
                    },
                    unknown => panic!("Don't know what to do with {:?}", unknown),
                },
                Binary::Lt | Binary::Gt | Binary::Ge | Binary::Le => {
                    if let EvalResult::Date(lhs) = left {
                        match right {
                            EvalResult::Date(rhs) => match op {
                                Binary::Lt => EvalResult::Boolean(lhs < rhs),
                                Binary::Gt => EvalResult::Boolean(lhs > rhs),
                                Binary::Ge => EvalResult::Boolean(lhs >= rhs),
                                Binary::Le => EvalResult::Boolean(lhs <= rhs),
                                x => panic!("Found {:?}", x),
                            },
                            x => panic!("Found {:?}", x),
                        }
                    } else {
                        panic!("Expected Date");
                    }
                }
                Binary::Add | Binary::Subtract => {
                    if let EvalResult::Number(lhs) = left {
                        if let EvalResult::Number(rhs) = right {
                            EvalResult::Number(match op {
                                Binary::Add => lhs + rhs,
                                Binary::Subtract => lhs - rhs,
                                _ => unreachable!(),
                            })
                        } else {
                            panic!("Should be numbers")
                        }
                    } else if let EvalResult::Money(lhs) = left {
                        if let EvalResult::Money(rhs) = right {
                            EvalResult::Money(match op {
                                Binary::Add => (lhs + rhs).to_money().unwrap(),
                                Binary::Subtract => (lhs - rhs).to_money().unwrap(),
                                _ => unreachable!(),
                            })
                        } else {
                            panic!("Should be money")
                        }
                    } else {
                        panic!("Should be money")
                    }
                }
                Binary::Mult | Binary::Div => {
                    if let EvalResult::Number(lhs) = left {
                        if let EvalResult::Number(rhs) = right {
                            EvalResult::Number(match op {
                                Binary::Mult => lhs * rhs,
                                Binary::Div => lhs / rhs,
                                _ => unreachable!(),
                            })
                        } else if let EvalResult::Money(rhs) = right {
                            EvalResult::Money(match op {
                                Binary::Mult => rhs * lhs, // the other way around is not implemented
                                Binary::Div => panic!("Can't divide number by money"),
                                _ => unreachable!(),
                            })
                        } else {
                            panic!("Should be numbers")
                        }
                    } else if let EvalResult::Money(lhs) = left {
                        if let EvalResult::Number(rhs) = right {
                            EvalResult::Money(match op {
                                Binary::Mult => lhs * rhs,
                                Binary::Div => lhs / rhs,
                                _ => unreachable!(),
                            })
                        } else {
                            panic!("rhs should be a number")
                        }
                    } else {
                        panic!("Should be numbers")
                    }
                }
                Binary::Or | Binary::And => {
                    if let EvalResult::Boolean(lhs) = left {
                        if let EvalResult::Boolean(rhs) = right {
                            EvalResult::Boolean(match op {
                                Binary::Or => lhs | rhs,
                                Binary::And => lhs & rhs,
                                _ => unreachable!(),
                            })
                        } else {
                            panic!("Should be booleans")
                        }
                    } else {
                        panic!("Should be booleans")
                    }
                }
                unknown => panic!("Not implemented: {:?}", unknown),
            }
        }
    }
}

#[derive(Clone)]
pub enum Unary {
    Not,
    Neg,
    Abs,
    Any,
    HasTag,
    Tag,
    ToDate,
}

#[derive(Clone, Debug)]
pub enum Binary {
    Add,
    Subtract,
    Mult,
    Div,
    Or,
    And,
    Eq,
    Ge,
    Gt,
    Le,
    Lt,
}

#[derive(Clone)]
pub enum Ternary {}

fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> Node {
    let rule = pair.as_rule();
    match rule {
        Rule::expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
        Rule::comparison_expr
        | Rule::or_expr
        | Rule::and_expr
        | Rule::additive_expr
        | Rule::multiplicative_expr => {
            let mut pair = pair.into_inner();
            let lhspair = pair.next().unwrap();
            let lhs = build_ast_from_expr(lhspair);
            match pair.next() {
                None => lhs,
                Some(x) => {
                    let op = match rule {
                        Rule::or_expr => Binary::Or,
                        Rule::and_expr => Binary::And,
                        _ => match x.as_str() {
                            "+" => Binary::Add,
                            "-" => Binary::Subtract,
                            "*" => Binary::Mult,
                            "/" => Binary::Div,
                            "=~" | "==" => Binary::Eq,
                            "<" => Binary::Lt,
                            ">" => Binary::Gt,
                            "<=" => Binary::Le,
                            ">=" => Binary::Ge,
                            x => unreachable!("{}", x),
                        },
                    };
                    let rhspair = pair.next().unwrap();
                    let rhs = build_ast_from_expr(rhspair);
                    parse_binary_expr(op, lhs, rhs)
                }
            }
        }
        Rule::primary => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            match first.as_rule() {
                Rule::function | Rule::unary => {
                    let op = match first.as_str() {
                        "abs" => Unary::Abs,
                        "-" => Unary::Neg,
                        "has_tag" => Unary::HasTag,
                        "tag" => Unary::Tag,
                        "to_date" => Unary::ToDate,
                        "not" => Unary::Not,
                        "any" => Unary::Any,
                        unknown => panic!("Unknown expr: {:?}", unknown),
                    };
                    parse_unary_expr(op, build_ast_from_expr(inner.next().unwrap()))
                }
                Rule::money => {
                    let mut money = first.into_inner();
                    let child = money.next().unwrap();
                    match child.as_rule() {
                        Rule::number => Node::Money {
                            currency: money.next().unwrap().as_str().to_string(),
                            amount: parse_big_rational(child.as_str()),
                        },
                        Rule::currency => Node::Money {
                            currency: child.as_str().to_string(),
                            amount: parse_big_rational(money.next().unwrap().as_str()),
                        },
                        unknown => panic!("Unknown rule: {:?}", unknown),
                    }
                }
                Rule::number => Node::Number(parse_big_rational(first.as_str())),
                Rule::regex | Rule::string => {
                    let full = first.as_str().to_string();
                    let n = full.len() - 1;
                    let slice = &full[1..n];
                    match first.as_rule() {
                        Rule::regex => Node::Regex(Regex::new(slice).unwrap()),
                        Rule::string => Node::String(slice.to_string()),
                        unknown => unreachable!("This cannot happen {:?}", unknown),
                    }
                }
                Rule::variable => match first.as_str() {
                    "account" => Node::Account,
                    "amount" => Node::Amount,
                    "payee" => Node::Payee,
                    "note" => Node::Note,
                    "date" => Node::Date,
                    unknown => panic!("Unknown variable: {:?}", unknown),
                },
                Rule::expr => build_ast_from_expr(first),

                unknown => panic!("Unknown rule: {:?}", unknown),
            }
        }
        unknown => panic!("Unknown expr: {:?}", unknown),
    }
}

fn parse_binary_expr(operation: Binary, lhs: Node, rhs: Node) -> Node {
    Node::BinaryExpr {
        op: operation,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn parse_unary_expr(operation: Unary, child: Node) -> Node {
    Node::UnaryExpr {
        op: operation,
        child: Box::new(child),
    }
}

fn parse_big_rational(input: &str) -> BigRational {
    let mut num = String::new();
    let mut den = "1".to_string();
    let mut decimal = false;
    for c in input.chars() {
        if c == '.' {
            decimal = true
        } else {
            num.push(c);
            if decimal {
                den.push('0')
            };
        }
    }
    BigRational::new(
        BigInt::from_str(num.as_str()).unwrap(),
        BigInt::from_str(den.as_str()).unwrap(),
    )
}
