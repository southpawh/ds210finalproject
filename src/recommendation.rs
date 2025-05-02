//! Module: Recommendation
//!
//! Responsibilities:
//! - Determine user preference `Bin`
//! - Filter items by bin
//! - Sort by rating and select top N

use crate::data_processing::{Bin, Item};

/// Categorizes user into a `Bin` based on read/watch days
pub fn user_bin(read_days: f64, movie_days: f64) -> Bin {
    let r = read_days / movie_days.max(1.0);
    match r {
        x if x < 0.5 => Bin::StrongMovie,
        x if x < 1.0 => Bin::MildMovie,
        x if x < 2.0 => Bin::MildBook,
        _            => Bin::StrongBook,
    }
}

/// Filters and returns top `n` book and DVD titles based on `Bin` and rating
pub fn recommend(
    items: &[Item],
    ubin: Bin,
    top_n: usize,
) -> (Vec<String>, Vec<String>) {
    let mut books: Vec<&Item> = items.iter()
        .filter(|i| i.group == "Book" && i.bin() == ubin)
        .collect();
    let mut dvds: Vec<&Item> = items.iter()
        .filter(|i| i.group == "DVD" && i.bin() == ubin.complement())
        .collect();
    books.sort_by(|a,b| b.rating.partial_cmp(&a.rating).unwrap());
    dvds.sort_by(|a,b| b.rating.partial_cmp(&a.rating).unwrap());
    (
        books.iter().take(top_n).map(|i| i.title.clone()).collect(),
        dvds.iter().take(top_n).map(|i| i.title.clone()).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_processing::Bin;

    #[test]
    fn test_user_bin() {
        assert_eq!(user_bin(1.0, 7.0), Bin::StrongMovie);
        assert_eq!(user_bin(3.0, 3.0), Bin::MildBook);
        assert_eq!(user_bin(5.0, 1.0), Bin::StrongBook);
    }
}
