//! Module: Data Processing
//!
//! Responsibilities:
//! - Read and parse the Amazon meta-data file into `Item` structs
//! - Compute copurchase counts for books and DVDs
//! - Assign each item to a preference `Bin` based on cross-group copurchase ratios

use std::fs::File;
use std::io::{BufRead, BufReader};

/// Four-tier preference bin for items and users
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bin {
    StrongMovie,
    MildMovie,
    MildBook,
    StrongBook,
}

impl Bin {
    /// Returns the complementary bin (e.g. StrongMovie <-> StrongBook)
    pub fn complement(self) -> Bin {
        match self {
            Bin::StrongMovie => Bin::StrongBook,
            Bin::MildMovie   => Bin::MildBook,
            Bin::MildBook    => Bin::MildMovie,
            Bin::StrongBook  => Bin::StrongMovie,
        }
    }
}

/// Product metadata plus computed fields
#[derive(Debug)]
pub struct Item {
    pub asin:       String,
    pub title:      String,
    pub group:      String,
    pub similar:    Vec<String>,
    pub book_count: usize,
    pub dvd_count:  usize,
    pub rating:     f64,
}

impl Item {
    /// Assigns this item to a `Bin` based on the ratio
    /// of cross-group copurchases to same-group copurchases.
    pub fn bin(&self) -> Bin {
        if self.group == "Book" {
            // Ratio of DVD-copurchases to Book-copurchases
            let ratio = (self.dvd_count as f64) / (self.book_count as f64).max(1.0);
            if ratio < 1.0 {
                Bin::MildBook
            } else if ratio < 2.0 {
                Bin::MildMovie
            } else {
                Bin::StrongBook
            }
        } else {
            // Ratio of Book-copurchases to DVD-copurchases
            let ratio = (self.book_count as f64) / (self.dvd_count as f64).max(1.0);
            if ratio < 1.0 {
                Bin::MildMovie
            } else if ratio < 2.0 {
                Bin::MildBook
            } else {
                Bin::StrongMovie
            }
        }
    }
}

/// Reads the raw TXT file and constructs `Item` records
pub fn read_data(path: &str) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut items = Vec::new();
    let mut curr_asin   = None;
    let mut curr_title  = None;
    let mut curr_group  = None;
    let mut curr_similar = Vec::new();
    let mut curr_rating = None;

    for line in reader.lines() {
        let t = line?.trim().to_string();
        if t.starts_with("Id:") {
            if let (Some(asin), Some(group)) = (curr_asin.take(), curr_group.take()) {
                let title  = curr_title.take().unwrap_or_default();
                let rating = curr_rating.take().unwrap_or(0.0);
                items.push(Item {
                    asin,
                    title,
                    group,
                    similar: curr_similar.clone(),
                    book_count: 0,
                    dvd_count:  0,
                    rating,
                });
            }
            curr_asin    = None;
            curr_title   = None;
            curr_group   = None;
            curr_similar.clear();
            curr_rating  = None;
        } else if t.starts_with("ASIN:") {
            curr_asin = Some(t[5..].trim().to_string());
        } else if t.starts_with("title:") {
            curr_title = Some(t[6..].trim().to_string());
        } else if t.starts_with("group:") {
            curr_group = Some(t[6..].trim().to_string());
        } else if t.starts_with("similar:") {
            let parts: Vec<&str> = t[8..].trim().split_whitespace().collect();
            if parts.len() > 1 {
                curr_similar = parts[1..].iter().map(|s| s.to_string()).collect();
            }
        } else if let Some(avg) = t.split("avg rating:").nth(1) {
            curr_rating = avg.trim().parse().ok();
        }
    }
    // push final record
    if let (Some(asin), Some(group)) = (curr_asin, curr_group) {
        let title  = curr_title.unwrap_or_default();
        let rating = curr_rating.unwrap_or(0.0);
        items.push(Item {
            asin,
            title,
            group,
            similar: curr_similar,
            book_count: 0,
            dvd_count:  0,
            rating,
        });
    }
    Ok(items)
}

/// Computes `book_count` & `dvd_count` for each `Item`
pub fn compute_counts(items: &mut [Item]) {
    let map: std::collections::HashMap<_, _> =
        items.iter().map(|i| (i.asin.clone(), i.group.clone())).collect();

    for item in items.iter_mut() {
        item.book_count = item.similar.iter()
            .filter(|s| map.get(*s).map(|g| g == "Book").unwrap_or(false))
            .count();
        item.dvd_count  = item.similar.iter()
            .filter(|s| map.get(*s).map(|g| g == "DVD").unwrap_or(false))
            .count();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_item_bin_book() {
        let item = Item {
            asin: "A".into(),
            title: "T".into(),
            group: "Book".into(),
            similar: vec![],
            book_count: 4,
            dvd_count: 1,
            rating: 0.0,
        };
        // dvd/book = 1/4 = 0.25 < 1.0 => MildBook
        assert_eq!(item.bin(), Bin::MildBook);
    }
}
