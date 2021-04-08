mod functions;
mod node;
mod result;
use super::{GrammarParser, Rule};
use crate::models::{Cleared, Currency, Money, Posting, PostingType, Transaction};
use crate::List;
use crate::{app, CommonOpts};
use colored::{Color, Styles};
pub use result::EvalResult;

use node::build_ast_from_expr;
pub use node::Node;

use num::abs;
use pest::Parser;
use regex::Regex;
use std::{collections::HashMap, rc::Rc};

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

fn eval_expression(
    expression: &str,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
    regexes: &mut HashMap<String, Regex>,
    options: &CommonOpts,
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
    eval(&root, posting, transaction, commodities, regexes, options)
}

pub fn eval_value_expression(
    expression: &str,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
    regexes: &mut HashMap<String, Regex>,
    options: &CommonOpts,
) -> Money {
    match eval_expression(
        expression,
        posting,
        transaction,
        commodities,
        regexes,
        options,
    ) {
        EvalResult::Number(n) => posting.amount.clone().unwrap() * n,
        EvalResult::Money(m) => m,
        x => {
            eprintln!("Found {:?}.", x);
            panic!("Should be money.");
        }
    }
}

/// Evaluate a format expression
///
/// Besides the expression itself, a format expression needs to now about some variables of the posting and the transaction and also whether it is the first element or not.
/// It evaluates to a (colored) string, so that it is properly displayed
pub fn format_expression_to_string(
    expression: &str,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    options: &CommonOpts,
    first: bool,
    commodities: &mut List<Currency>,
    regexes: &mut HashMap<String, Regex>,
) -> String {
    let mut parsed = GrammarParser::parse(Rule::format_expression, expression)
        .expect("unsuccessful parse")
        .next()
        .unwrap()
        .into_inner(); // unwrap the parse result

    let mut expression = parsed.next().unwrap();

    if !first {
        match parsed.next() {
            Some(x) => expression = x,
            None => {} // Do nothing
        }
    }

    // Now expression is sure to be a format_expression_part, which is what must be evaluated
    assert_eq!(expression.as_rule(), Rule::format_expression_part);

    // Build the abstract syntax tree
    let root = build_ast_from_expr(expression, regexes);

    match eval(&root, posting, transaction, commodities, regexes, options) {
        EvalResult::String(s) => s,
        x => {
            eprintln!("Found {:?}.", x);
            panic!("Should be a string.");
        }
    }
}

pub fn eval(
    node: &Node,
    posting: &Posting,
    transaction: &Transaction<Posting>,
    commodities: &mut List<Currency>,
    regexes: &mut HashMap<String, Regex>,
    options: &CommonOpts,
) -> EvalResult {
    let res = match node {
        Node::Amount => EvalResult::Money(posting.amount.clone().unwrap()),
        Node::Account => EvalResult::Account(posting.account.clone()),
        Node::Payee => EvalResult::Payee(posting.payee.clone().unwrap()),
        Node::Note => EvalResult::Note,
        Node::Date => EvalResult::Date(posting.date.clone()),
        Node::Regex(r) => EvalResult::Regex(r.clone()),
        Node::String(r) => EvalResult::String(r.to_string()),
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
            let res = eval(child, posting, transaction, commodities, regexes, options);
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
                            eval(child, p, transaction, commodities, regexes, options)
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
                    EvalResult::Regex(r) => EvalResult::MaybeString(posting.get_tag(r)),
                    EvalResult::String(r) => EvalResult::MaybeString(posting.get_exact_tag(r)),
                    x => panic!("Expected regex. Found {:?}", x),
                },
                Unary::ToDate => match res {
                    EvalResult::String(r) => {
                        EvalResult::Date(app::date_parser(r.as_str()).unwrap())
                    }
                    x => panic!("Expected String. Found {:?}", x),
                },
            }
        }
        Node::BinaryExpr { op, lhs, rhs } => {
            let left = eval(lhs, posting, transaction, commodities, regexes, options);
            let right = eval(rhs, posting, transaction, commodities, regexes, options);
            match op {
                Binary::Eq => match right {
                    EvalResult::Regex(rhs) => match left {
                        EvalResult::Account(lhs) => EvalResult::Boolean(lhs.is_match(rhs)),
                        EvalResult::Payee(lhs) => EvalResult::Boolean(lhs.is_match(rhs)),
                        EvalResult::String(lhs) => EvalResult::Boolean(rhs.is_match(lhs.as_str())),

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
        Node::Function { name, args } => {
            let partial_results = args
                .iter()
                .map(|x| eval(x, posting, transaction, commodities, regexes, options))
                .collect::<Vec<EvalResult>>();
            match name.as_str() {
                "concatenate" => functions::concatenate(partial_results),
                "format_date" => functions::format_date(partial_results, options),
                "int" => functions::int(&partial_results[0]),
                "justify" => functions::justify(partial_results, options),
                "scrub" => functions::scrub(partial_results, options),
                "if_expression" => functions::if_expression(
                    partial_results[0].to_owned(),
                    partial_results[1].to_owned(),
                ),
                "ansify_if" => functions::ansify_if(&partial_results[0], &partial_results[1]),
                "truncated" => functions::truncated(&partial_results[0], &partial_results[1]),
                "!" => functions::not(&partial_results[0]),
                x => unimplemented!("Function {}, args: {:?}", x, partial_results),
            }
        }
        Node::Variable(name) => match name.as_str() {
            "date_width" => EvalResult::Usize(options.date_width),
            "payee_width" => EvalResult::Usize(options.payee_width),
            "account_width" => EvalResult::Usize(options.account_width),
            // todo deal with color better
            "color" => EvalResult::Boolean(options.force_color),
            "today" => EvalResult::Date(options.now()),
            "green" => EvalResult::Color(Color::Green),
            "bold" => EvalResult::Style(Styles::Bold),
            "should_bold" => match &options.bold_if {
                Some(expression) => eval_expression(
                    expression.as_str(),
                    posting,
                    transaction,
                    commodities,
                    regexes,
                    options,
                ),
                None => EvalResult::Boolean(false),
            },
            "cleared" => {
                if let Cleared::Cleared = transaction.cleared {
                    EvalResult::Boolean(true)
                } else {
                    EvalResult::Boolean(false)
                }
            }
            "actual" => EvalResult::Boolean(posting.kind == PostingType::Real),
            "display_account" => EvalResult::String(posting.account.to_string()),
            x => unimplemented!("Variable {}", x),
        },
        Node::Substitution {
            min_width,
            max_width,
            child,
        } => {
            let res = eval(child, posting, transaction, commodities, regexes, options);
            let res_str = res.to_string();
            // todo min_width, max_width
            EvalResult::String(res_str)
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
