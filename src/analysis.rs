//! Module: Analysis
//!
//! Responsibilities:
//! - Build and analyze the copurchase graph
//! - Compute degree distributions
//! - Perform linear regression

use petgraph::prelude::Graph;
use petgraph::Undirected;
use std::collections::HashMap;

/// Builds an undirected graph; nodes=ASIN, edges=copurchases
pub fn build_graph(items: &[crate::data_processing::Item]) -> Graph<String, (), Undirected> {
    let mut g = Graph::new_undirected();
    let mut idx = HashMap::new();
    for item in items {
        let n = g.add_node(item.asin.clone());
        idx.insert(item.asin.clone(), n);
    }
    for item in items {
        let u = idx[&item.asin];
        for nei in &item.similar {
            if let Some(&v) = idx.get(nei) {
                if !g.contains_edge(u, v) {
                    g.add_edge(u, v, ());
                }
            }
        }
    }
    g
}

/// Degree -> count of nodes with that degree
pub fn degree_distribution(graph: &Graph<String, (), Undirected>) -> HashMap<usize, usize> {
    let mut dist = HashMap::new();
    for n in graph.node_indices() {
        let d = graph.neighbors(n).count();
        *dist.entry(d).or_insert(0) += 1;
    }
    dist
}

/// Fits y = m*x + b using least squares
pub fn linear_regression(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    let sx: f64 = x.iter().sum();
    let sy: f64 = y.iter().sum();
    let sxy: f64 = x.iter().zip(y.iter()).map(|(a,b)| a*b).sum();
    let sx2: f64 = x.iter().map(|a| a*a).sum();
    let m = (n*sxy - sx*sy) / (n*sx2 - sx*sx);
    let b = (sy - m*sx) / n;
    (m, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_processing::Item;

    #[test]
    fn test_degree_distribution() {
        let items = vec![
            Item { asin: "A".into(), title: "".into(), group: "Book".into(), similar: vec!["B".into()], book_count: 0, dvd_count: 0, rating: 0.0 },
            Item { asin: "B".into(), title: "".into(), group: "DVD".into(),  similar: vec!["A".into()], book_count: 0, dvd_count: 0, rating: 0.0 },
        ];
        let g = build_graph(&items);
        let dist = degree_distribution(&g);
        assert_eq!(dist.get(&1), Some(&2));
    }

    #[test]
    fn test_regression() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![2.0, 4.0, 6.0];
        let (m, b) = linear_regression(&x, &y);
        assert!((m - 2.0).abs() < 1e-6);
        assert!(b.abs() < 1e-6);
    }
}
