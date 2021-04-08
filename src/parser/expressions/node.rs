use std::{collections::HashMap, ops::Add};

use num::BigRational;
use regex::Regex;

use super::{Binary, Unary};
use crate::parser::utils::parse_rational;
use crate::parser::Rule;
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
    Function {
        name: String,
        args: Vec<Node>,
    },
    Substitution {
        min_width: Option<usize>,
        max_width: Option<usize>,
        child: Box<Node>,
    },
    Regex(Regex),
    String(String),
    Variable(String),
}
impl Add for Node {
    type Output = Node;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Node::String(x) => match rhs {
                Node::String(y) => Node::String(format!("{}{}", x, y)),
                _ => panic!("Can't concatenate {:?} and {:?}", x, &rhs),
            },
            x => panic!("Can't concatenate {:?} and {:?}", x, rhs),
        }
    }
}

/// Build an abstract syntax tree from an expression
pub(super) fn build_ast_from_expr(
    pair: pest::iterators::Pair<Rule>,
    regexes: &mut HashMap<String, Regex>,
) -> Node {
    let rule = pair.as_rule();
    match rule {
        Rule::format_expression_part => {
            let mut args = vec![];
            for part in pair.into_inner() {
                args.push(build_ast_from_expr(part, regexes));
            }
            Node::Function {
                name: "concatenate".to_string(),
                args,
            }
        }
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
                        "abs" => Some(Unary::Abs),
                        "-" => Some(Unary::Neg),
                        "has_tag" => Some(Unary::HasTag),
                        "tag" => Some(Unary::Tag),
                        "to_date" => Some(Unary::ToDate),
                        "not" => Some(Unary::Not),
                        "any" => Some(Unary::Any),
                        unknown => None,
                    };
                    if let Some(op) = op {
                        parse_unary_expr(op, build_ast_from_expr(inner.next().unwrap(), regexes))
                    } else {
                        let mut args = vec![];
                        while let Some(part) = inner.next() {
                            args.push(build_ast_from_expr(part, regexes));
                        }
                        Node::Function {
                            name: first.as_str().to_string(),
                            args,
                        }
                    }
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
                    other => Node::Variable(other.to_string()),
                },
                Rule::expr => build_ast_from_expr(first, regexes),
                unknown => panic!("Unknown rule: {:?}", unknown),
            }
        }
        Rule::substitution => {
            let mut inner = pair.into_inner();
            let mut to_parse = inner.next().unwrap();
            let left_aligned = if to_parse.as_rule() == Rule::left_aligned {
                to_parse = inner.next().unwrap();
                true
            } else {
                false
            };
            let mut min_width = None;
            if to_parse.as_rule() == Rule::min_width {
                min_width =
                    Some(usize::from_str_radix(to_parse.into_inner().as_str(), 10).unwrap());
                to_parse = inner.next().unwrap();
            };
            let mut max_width = None;
            if to_parse.as_rule() == Rule::max_width {
                max_width =
                    Some(usize::from_str_radix(to_parse.into_inner().as_str(), 10).unwrap());
                to_parse = inner.next().unwrap();
            };

            let node = match to_parse.as_rule() {
                Rule::value_expr => {
                    build_ast_from_expr(to_parse.into_inner().next().unwrap(), regexes)
                }
                _ => build_ast_from_expr(to_parse, regexes),
            };
            Node::Substitution {
                min_width,
                max_width,
                child: Box::new(node),
            }
        }
        Rule::if_expr => {
            let mut inner = pair.into_inner();
            let mut args = vec![];
            while let Some(part) = inner.next() {
                args.push(build_ast_from_expr(part, regexes));
            }
            Node::Function {
                name: "if_expression".to_string(),
                args,
            }
        }
        unknown => panic!("Unknown expr: {:?}\n{}", unknown, pair.as_str()),
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
