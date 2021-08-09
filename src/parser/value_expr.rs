use super::utils::parse_rational;
use super::{GrammarParser, Rule};
use crate::app;
use crate::models::{Account, Currency, Money, Payee, Posting, Transaction};
use crate::List;
use chrono::NaiveDate;

use num::{abs, BigRational};
use pest::Parser;
use regex::Regex;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

/// Builds the abstract syntax tree, to be able to evaluate expressions
///
/// This all comes from the defined grammar.pest
pub fn build_root_node_from_expression(
    expression: &str,
    regexes: &mut HashMap<String, Regex>,
) -> Node {
    let parsed = GrammarParser::parse(Rule::value_expr, expression)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    // Build the abstract syntax tree
    build_ast_from_expr(parsed, regexes)
}

pub fn eval_expression(
    expression: &str,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
    regexes: &mut HashMap<String, Regex>,
) -> EvalResult {
    let parsed = GrammarParser::parse(Rule::value_expr, expression)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    // Build the abstract syntax tree
    let root = build_ast_from_expr(parsed, regexes);
    eval(&root, posting, transaction, commodities, regexes)
}

pub fn eval_value_expression(
    expression: &str,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
    regexes: &mut HashMap<String, Regex>,
) -> Money {
    match eval_expression(expression, posting, transaction, commodities, regexes) {
        EvalResult::Number(n) => posting.amount.clone().unwrap() * n,
        EvalResult::Money(m) => m,
        x => {
            eprintln!("Found {:?}.", x);
            panic!("Should be money.");
        }
    }
}

#[derive(Clone, Debug)]
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
    Note,
}

/// Only use this for >, <, >=, <=, not for equality
impl PartialOrd for EvalResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            EvalResult::Number(left) => match other {
                EvalResult::Number(right) => Some(left.cmp(right)),
                EvalResult::Money(right) => Some(left.cmp(right.get_amount().borrow())),
                _ => None,
            },
            EvalResult::Money(left) => match other {
                EvalResult::Number(right) => Some(left.get_amount().cmp(right)),
                EvalResult::Money(right) => Some(left.cmp(&right)),
                _ => None,
            },
            EvalResult::String(left) => match other {
                EvalResult::String(right) => Some(left.cmp(&right)),
                _ => None,
            },
            EvalResult::Date(left) => match other {
                EvalResult::Date(right) => Some(left.cmp(&right)),
                _ => None,
            },
            _ => None,
        }
    }
}

impl PartialEq for EvalResult {
    fn eq(&self, other: &Self) -> bool {
        match self {
            EvalResult::Number(left) => match other {
                EvalResult::Number(right) => left == right,
                EvalResult::Money(right) => left == &right.get_amount(),
                x => panic!("Expected number, found {:?}", x),
            },
            EvalResult::Money(left) => match other {
                EvalResult::Number(right) => &left.get_amount() == right,
                EvalResult::Money(right) => left == right,
                x => panic!("Can't compare money and {:?}", x),
            },

            EvalResult::Boolean(left) => match other {
                EvalResult::Boolean(right) => left == right,
                x => panic!("Expected boolean, found {:?}", x),
            },
            EvalResult::String(left) => match other {
                EvalResult::String(right) => left == right,
                x => panic!("Expected string, found {:?}", x),
            },
            EvalResult::Date(left) => match other {
                EvalResult::Date(right) => left == right,
                x => panic!("Expected string, found {:?}", x),
            },
            x => panic!("Can't compare {:?}", x),
        }
    }
}

pub fn eval(
    node: &Node,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &List<Currency>,
    regexes: &mut HashMap<String, Regex>,
) -> EvalResult {
    let res = match node {
        Node::Amount => EvalResult::Money(posting.amount.clone().unwrap()),
        Node::Account => EvalResult::Account(posting.account.clone()),
        Node::Payee => EvalResult::Payee(posting.payee.clone().unwrap()),
        Node::Note => EvalResult::Note,
        Node::Date => EvalResult::Date(posting.date.clone()),
        Node::Regex(r) => EvalResult::Regex(r.clone()),
        Node::String(r) => EvalResult::String(Some(r.clone())),
        Node::Number(n) => EvalResult::Number(n.clone()),
        Node::Money { currency, amount } => {
            let cur = match commodities.get(&currency) {
                Ok(c) => c.clone(),
                Err(_) => {
                    panic!("Can't find commodity {:?}", currency);
                }
            };
            EvalResult::Money(Money::from((cur.clone(), amount.clone())))
        }
        Node::UnaryExpr { op, child } => {
            let res = eval(child, posting, transaction, commodities, regexes);
            match op {
                Unary::Not => match res {
                    EvalResult::Boolean(b) => EvalResult::Boolean(!b),
                    x => panic!("Can't do neg of {:?}", x),
                },
                Unary::Any => {
                    let mut res = false;
                    for p in transaction.postings.borrow().iter() {
                        // if p.origin != PostingOrigin::FromTransaction {
                        //     continue;
                        // }
                        if let EvalResult::Boolean(b) =
                            eval(child, p, transaction, commodities, regexes)
                        {
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
            let left = eval(lhs, posting, transaction, commodities, regexes);
            let right = eval(rhs, posting, transaction, commodities, regexes);
            match op {
                Binary::Eq => match right {
                    EvalResult::Regex(rhs) => match left {
                        EvalResult::Account(lhs) => EvalResult::Boolean(lhs.is_match(rhs)),
                        EvalResult::Payee(lhs) => EvalResult::Boolean(lhs.is_match(rhs)),
                        EvalResult::String(lhs) => match lhs {
                            Some(lhs) => EvalResult::Boolean(rhs.is_match(lhs.as_str())),
                            None => EvalResult::Boolean(false),
                        },
                        EvalResult::Note => {
                            let mut result = false;
                            for comment in transaction.comments.iter() {
                                if rhs.is_match(comment.comment.as_str()) {
                                    result = true;
                                    break;
                                }
                            }
                            EvalResult::Boolean(result)
                        }
                        x => panic!("Found {:?}", x),
                    },
                    _ => EvalResult::Boolean(left == right),
                },
                Binary::Lt => EvalResult::Boolean(left < right),
                Binary::Gt => EvalResult::Boolean(left > right),
                Binary::Ge => EvalResult::Boolean(left >= right),
                Binary::Le => EvalResult::Boolean(left <= right),
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
                        EvalResult::Boolean(match op {
                            Binary::Or => {
                                if lhs {
                                    true
                                } else if let EvalResult::Boolean(rhs) = right {
                                    rhs
                                } else {
                                    panic!("Should be booleans")
                                }
                            }
                            Binary::And => {
                                if !lhs {
                                    false
                                } else if let EvalResult::Boolean(rhs) = right {
                                    rhs
                                } else {
                                    panic!("Should be booleans")
                                }
                            }
                            _ => unreachable!(),
                        })
                    } else {
                        panic!("Should be booleans")
                    }
                }
            }
        }
    };
    res
}

#[derive(Clone, Debug)]
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

fn build_ast_from_expr(
    pair: pest::iterators::Pair<Rule>,
    regexes: &mut HashMap<String, Regex>,
) -> Node {
    let rule = pair.as_rule();
    match rule {
        Rule::expr => build_ast_from_expr(pair.into_inner().next().unwrap(), regexes),
        Rule::comparison_expr
        | Rule::or_expr
        | Rule::and_expr
        | Rule::additive_expr
        | Rule::multiplicative_expr => {
            let mut pair = pair.into_inner();
            let lhspair = pair.next().unwrap();
            let lhs = build_ast_from_expr(lhspair, regexes);
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
                    let rhs = build_ast_from_expr(rhspair, regexes);
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
                    parse_unary_expr(op, build_ast_from_expr(inner.next().unwrap(), regexes))
                }
                Rule::money => {
                    let mut money = first.into_inner();
                    let child = money.next().unwrap();
                    match child.as_rule() {
                        Rule::number => Node::Money {
                            currency: money.next().unwrap().as_str().to_string(),
                            amount: parse_rational(child),
                        },
                        Rule::currency => Node::Money {
                            currency: child.as_str().to_string(),
                            amount: parse_rational(money.next().unwrap()),
                        },
                        unknown => panic!("Unknown rule: {:?}", unknown),
                    }
                }
                Rule::number => Node::Number(parse_rational(first)),
                Rule::regex | Rule::string => {
                    let full = first.as_str().to_string();
                    let n = full.len() - 1;
                    let slice = &full[1..n];
                    match first.as_rule() {
                        Rule::regex => match regexes.get(slice) {
                            None => {
                                let regex = Regex::new(slice).unwrap();
                                regexes.insert(slice.to_string(), regex.clone());
                                Node::Regex(regex)
                            }
                            Some(regex) => Node::Regex(regex.clone()),
                        },
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
                Rule::expr => build_ast_from_expr(first, regexes),

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
