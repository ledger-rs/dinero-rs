use crate::models::{Currency, HasName, Money};
use chrono::{Duration, NaiveDate};
use num::rational::BigRational;
use num::BigInt;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

/// A price relates two commodities
#[derive(Debug, Clone)]
pub struct Price {
    date: NaiveDate,
    commodity: Rc<Currency>,
    price: Money,
}

impl Price {
    pub fn new(date: NaiveDate, commodity: Rc<Currency>, price: Money) -> Price {
        Price {
            date,
            commodity,
            price,
        }
    }
    pub fn get_price(&self) -> Money {
        self.price.clone()
    }
}

impl Display for Price {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.date, self.commodity, self.get_price())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PriceType {
    Total,
    PerUnit,
}

/// Convert from one currency to every other currency
///
/// This uses an implementation of the [Dijkstra algorithm](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm) to find the shortest path from every
/// currency to the desired one
pub fn conversion(
    currency: Rc<Currency>,
    date: NaiveDate,
    prices: &Vec<Price>,
) -> HashMap<Rc<Currency>, BigRational> {
    // Build the graph
    let source = Node {
        currency: currency.clone(),
        date,
    };
    let mut graph = Graph::from_prices(prices, source.clone());
    let mut distances = HashMap::new();
    let mut paths: HashMap<Rc<Node>, Vec<Rc<Edge>>> = HashMap::new();
    let mut queue = vec![];

    // Initialize distances
    for node in graph.nodes.iter() {
        // println!("{} {} ", node.currency.get_name(), node.date);
        if node.currency == currency {
            distances.insert(node.clone(), Some(date - node.date));
            // distances.insert(node.clone(), Some(date - date));
            // println!("{}", date - node.date);
        } else {
            distances.insert(node.clone(), None);
            // println!("None");
        }
        queue.push(node.clone());
    }
    while !queue.is_empty() {
        // Sort largest to smallest
        queue.sort_by(|a, b| cmp(distances.get(b).unwrap(), distances.get(a).unwrap()));

        // Take the closest node
        let v = queue.pop().unwrap();
        // This means there is no path to the node
        if distances.get(v.as_ref()).unwrap().is_none() {
            break;
        }

        // The path from the starting currency to the node
        let current_path = if let Some(path) = paths.get(v.as_ref()) {
            path.clone()
        } else {
            Vec::new()
        };

        // Update the distances
        for (u, e) in graph.get_neighbours(v.as_ref()).iter() {
            // println!("Neighbour: {} {}", u.currency.get_name(), u.date);
            let alt = distances.get(v.as_ref()).unwrap().unwrap() + e.length();
            let distance = distances.get(u.as_ref()).unwrap();
            let mut update = distance.is_none();
            if !update {
                update = alt < distance.unwrap();
            }
            if !update {
                continue;
            }
            distances.insert(u.clone(), Some(alt));
            let mut u_path = current_path.clone();
            u_path.push(e.clone());
            paths.insert(u.clone(), u_path);
        }
    }

    // Return not the paths but the multipliers
    let mut multipliers = HashMap::new();
    let mut inserted = HashMap::new();
    for (k, v) in paths.iter() {
        // println!("{} {} ~{:?}", k.currency.get_name(), k.date, v.len());
        let mut mult = BigRational::new(BigInt::from(1), BigInt::from(1));
        let mut currency = k.currency.clone();
        match inserted.get(&k.currency) {
            Some(x) => {
                if *x > k.date {
                    continue;
                }
            }
            None => {
                inserted.insert(currency.clone(), k.date);
            }
        }
        for edge in v.iter().rev() {
            match edge.as_ref().price.as_ref() {
                None => (), // do nothing, multiply by one and keep the same currency
                Some(price) => {
                    if currency == edge.from.currency {
                        mult *= price.get_price().get_amount();
                        currency = edge.to.currency.clone();
                    } else {
                        mult /= price.get_price().get_amount();
                        currency = edge.from.currency.clone();
                    }
                }
            }
        }
        multipliers.insert(k.currency.clone(), mult);
    }
    multipliers
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Node {
    pub(crate) currency: Rc<Currency>,
    date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct Edge {
    price: Option<Price>,
    from: Rc<Node>,
    to: Rc<Node>,
}

impl Edge {
    fn length(&self) -> Duration {
        if self.from.date > self.to.date {
            self.from.date - self.to.date
        } else {
            self.to.date - self.from.date
        }
    }
}

#[derive(Debug, Clone)]
struct Graph {
    nodes: Vec<Rc<Node>>,
    edges: Vec<Rc<Edge>>,
    _neighbours: HashMap<Rc<Node>, Vec<(Rc<Node>, Rc<Edge>)>>,
}

impl Graph {
    fn from_prices(prices: &Vec<Price>, source: Node) -> Self {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        let mut currency_dates = HashSet::new();
        currency_dates.insert((source.currency.clone(), source.date));
        // Remove redundant prices and create the nodes
        let mut prices_nodup = HashMap::new();
        for p in prices.iter() {
            if p.date > source.date {
                continue;
            };
            let commodities =
                if p.price.get_commodity().unwrap().get_name() < p.commodity.as_ref().get_name() {
                    (p.price.get_commodity().unwrap(), p.commodity.clone())
                } else {
                    (p.commodity.clone(), p.price.get_commodity().unwrap())
                };
            match prices_nodup.get(&commodities) {
                None => {
                    prices_nodup.insert(commodities.clone(), p.clone());
                }
                Some(x) => {
                    if x.date < p.date {
                        prices_nodup.insert(commodities.clone(), p.clone());
                    }
                }
            }
        }
        for (_, p) in prices_nodup.iter() {
            let commodities =
                if p.price.get_commodity().unwrap().get_name() < p.commodity.as_ref().get_name() {
                    (p.price.get_commodity().unwrap(), p.commodity.clone())
                } else {
                    (p.commodity.clone(), p.price.get_commodity().unwrap())
                };
            let c_vec = vec![commodities.0.clone(), commodities.1.clone()];
            for c in c_vec {
                currency_dates.insert((c.clone(), p.date));
            }
        }
        // Create the nodes
        for (c, d) in currency_dates.iter() {
            nodes.insert(
                (c.clone(), d.clone()),
                Rc::new(Node {
                    currency: c.clone(),
                    date: d.clone(),
                }),
            );
        }
        // Edges from the prices
        for (_, p) in prices_nodup.iter() {
            let from = nodes
                .get(&(p.commodity.clone(), p.date.clone()))
                .unwrap()
                .clone();
            let to = nodes
                .get(&(p.price.get_commodity().unwrap(), p.date.clone()))
                .unwrap()
                .clone();
            edges.push(Rc::new(Edge {
                price: Some(p.clone()),
                from: from.clone(),
                to: to.clone(),
            }));
        }

        // println!("Nodes: {}", nodes.len());
        // println!("Edges: {}", edges.len());
        let vec_node: Vec<Rc<Node>> = nodes.iter().map(|x| x.1.clone()).collect();
        let n = vec_node.len();
        for i in 0..n {
            for j in i..n {
                if vec_node[i].currency == vec_node[j].currency {
                    edges.push(Rc::new(Edge {
                        price: None,
                        from: vec_node[i].clone(),
                        to: vec_node[j].clone(),
                    }))
                }
            }
        }
        // println!("Edges: {}", edges.len());

        Graph {
            nodes: nodes.iter().map(|x| x.1.clone()).collect(),
            edges,
            _neighbours: HashMap::new(),
        }
    }
    fn get_neighbours(&mut self, node: &Node) -> Vec<(Rc<Node>, Rc<Edge>)> {
        match self._neighbours.get(node) {
            None => {
                let mut neighbours = Vec::new();
                for edge in self.edges.iter() {
                    if edge.from.as_ref() == node {
                        neighbours.push((edge.to.clone(), edge.clone()));
                    } else if edge.to.as_ref() == node {
                        neighbours.push((edge.from.clone(), edge.clone()));
                    }
                }
                self._neighbours
                    .insert(Rc::new(node.clone()), neighbours.clone());
                neighbours
            }
            Some(x) => x.clone(),
        }
    }
}

fn cmp(this: &Option<Duration>, other: &Option<Duration>) -> Ordering {
    match this {
        None => match other {
            None => Ordering::Equal,
            Some(_) => Ordering::Greater,
        },
        Some(s) => match other {
            None => Ordering::Less,
            Some(o) => s.cmp(o),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Tokenizer;
    use crate::CommonOpts;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_graph() {
        // Copy from balance command
        let path = PathBuf::from("tests/example_files/demo.ledger");
        let mut tokenizer = Tokenizer::from(&path);
        let options = CommonOpts::new();
        let items = tokenizer.tokenize(&options);
        let ledger = items.to_ledger(&options).unwrap();

        let currency = ledger.commodities.get("EUR").unwrap();
        let multipliers = conversion(
            currency.clone(),
            Utc::now().naive_local().date(),
            &ledger.prices,
        );
        assert_eq!(multipliers.len(), 7);
    }
}
