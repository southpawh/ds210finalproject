//! Module: Recommendation
//!
//! Responsibilities:
//! - Compute dataset quartile thresholds
//! - Bin user by those thresholds
//! - Filter, sort, pick top-N recommendations

use crate::data_processing::Item;

/// Four quartile bins
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bin {
    Q1, Q2, Q3, Q4,
}

/// Assigns ratio to quartile bin
fn quantile_bin(r: f64, q25: f64, q50: f64, q75: f64) -> Bin {
    if r < q25 { Bin::Q1 } else if r < q50 { Bin::Q2 } else if r < q75 { Bin::Q3 } else { Bin::Q4 }
}

/// User bin based on their read/watch ratio and dataset quartiles
pub fn user_bin_quantile(read_days: f64, movie_days: f64, q25: f64, q50: f64, q75: f64) -> Bin {
    let ratio = read_days / movie_days.max(1.0);
    quantile_bin(ratio, q25, q50, q75)
}

/// Top-N titles matching user quartile
pub fn recommend_quantile(items: &[Item], ubin: Bin, top_n: usize, q25: f64, q50: f64, q75: f64) -> (Vec<String>, Vec<String>) {
    let mut books: Vec<&Item> = items.iter().filter(|i| i.group == "Book" && quantile_bin(i.ratio(), q25, q50, q75) == ubin).collect();
    let mut dvds:  Vec<&Item> = items.iter().filter(|i| i.group == "DVD"  && quantile_bin(i.ratio(), q25, q50, q75) == ubin).collect();
    if books.is_empty() { books = items.iter().filter(|i| i.group=="Book").collect(); }
    if dvds.is_empty()  { dvds  = items.iter().filter(|i| i.group=="DVD").collect(); }
    books.sort_by(|a,b| b.rating.partial_cmp(&a.rating).unwrap());
    dvds.sort_by(|a,b| b.rating.partial_cmp(&a.rating).unwrap());
    ( books.into_iter().take(top_n).map(|i| i.title.clone()).collect(),
      dvds.into_iter().take(top_n).map(|i| i.title.clone()).collect() )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_processing::Item;

    #[test]
    fn test_user_bin_quantile_edges() {
        // thresholds 0.5,1,2
        assert_eq!(user_bin_quantile(1.0,5.0,0.5,1.0,2.0), Bin::Q1);
        assert_eq!(user_bin_quantile(0.75,1.0,0.5,1.0,2.0), Bin::Q2);
        assert_eq!(user_bin_quantile(1.5,1.0,0.5,1.0,2.0), Bin::Q3);
        assert_eq!(user_bin_quantile(3.0,1.0,0.5,1.0,2.0), Bin::Q4);
    }

    #[test]
    fn test_recommend_quantile_fallback() {
        let items = vec![
            Item{asin:"A".into(),title:"Book A".into(),group:"Book".into(),similar:vec![],book_count:0,dvd_count:0,rating:4.0},
            Item{asin:"B".into(),title:"DVD B".into(),group:"DVD".into(),similar:vec![],book_count:0,dvd_count:0,rating:5.0},
        ];
        let (books, dvds) = recommend_quantile(&items, Bin::Q4, 1, 0.1,0.2,0.3);
        assert_eq!(books, vec!["Book A".to_string()]);
        assert_eq!(dvds,   vec!["DVD B".to_string()]);
    }
}
