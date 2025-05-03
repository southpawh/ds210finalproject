mod data_processing;
mod analysis;
mod recommendation;

use std::io;
use data_processing::{read_data, compute_counts};
use analysis::{build_graph, degree_distribution, linear_regression};
use recommendation::{user_bin_quantile, recommend_quantile, Bin};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load & prepare
    let mut items = read_data("data/amazon-meta-data.txt")?;
    compute_counts(&mut items);

    // Quartile thresholds
    let mut ratios: Vec<f64> = items.iter().map(|i| i.ratio()).collect();
    ratios.sort_by(|a,b| a.partial_cmp(b).unwrap());
    let n = ratios.len();
    let idx = |pct: f64| -> usize { ((n as f64) * pct / 100.0).floor() as usize };
    let max_idx = n.saturating_sub(1);
    let q25 = ratios[idx(25.0).min(max_idx)];
    let q50 = ratios[idx(50.0).min(max_idx)];
    let q75 = ratios[idx(75.0).min(max_idx)];

    // Analysis
    let graph = build_graph(&items);
    println!("Degree distribution: {:?}", degree_distribution(&graph));
    let (xs, ys): (Vec<f64>, Vec<f64>) = items.iter().map(|i| (i.book_count as f64, i.dvd_count as f64)).unzip();
    let (m, b) = linear_regression(&xs, &ys);
    println!("Linear regression: y = {}x + {}", m, b);

    // User input
    println!("How many days/week do you read? (1-7)");
    let mut buf = String::new(); io::stdin().read_line(&mut buf)?;
    let rd: f64 = buf.trim().parse().unwrap_or(1.0);
    buf.clear();
    println!("How many days/week do you watch movies? (1-7)");
    io::stdin().read_line(&mut buf)?;
    let md: f64 = buf.trim().parse().unwrap_or(1.0);

    let ub = user_bin_quantile(rd, md, q25, q50, q75);
    println!("You're in {:?} bin.", ub);

    // Recommendations
    println!("Top 10 recommendations:");
    let (books, dvds) = recommend_quantile(&items, ub, 10, q25, q50, q75);
    println!("Books: {:?}\nDVDs: {:?}", books, dvds);
    Ok(())
}
