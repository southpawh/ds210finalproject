mod data_processing;
mod analysis;
mod recommendation;

use std::io;
use data_processing::{read_data, compute_counts};
use analysis::{build_graph, degree_distribution, linear_regression};
use recommendation::{user_bin, recommend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load data and compute copurchase counts
    let mut items = read_data("data/amazon-meta-data.txt")?;
    compute_counts(&mut items);

    // Graph analysis
    let graph = build_graph(&items);
    println!("Degree distribution: {:?}", degree_distribution(&graph));
    let (xs, ys): (Vec<f64>, Vec<f64>) = items.iter()
        .map(|i| (i.book_count as f64, i.dvd_count as f64)).unzip();
    let (m, b) = linear_regression(&xs, &ys);
    println!("Linear regression: y = {}x + {}", m, b);

    // User interaction
    println!("How many days in the week do you dedicate time to reading (1-7)?");
    let mut buf = String::new(); io::stdin().read_line(&mut buf)?;
    let rd: f64 = buf.trim().parse().unwrap_or(1.0);
    buf.clear();
    println!("How many days in the week do you spend watching a movie (1-7)?");
    io::stdin().read_line(&mut buf)?;
    let md: f64 = buf.trim().parse().unwrap_or(1.0);

    let ub = user_bin(rd, md);
    if rd > md {
        println!("It seems you prefer books. Here's top books and some DVDs:");
    } else if md > rd {
        println!("It seems you prefer movies. Here's top DVDs and some books:");
    } else {
        println!("You enjoy both equally. Here's top picks for each:");
    }

    println!("I know the full list is large, so here are the top 10 of each by rating:");
    let (books, dvds) = recommend(&items, ub, 10);
    println!("Recommended books: {:?}\nRecommended DVDs: {:?}", books, dvds);

    Ok(())
}
