use crate::models::{HasName, Posting, PostingType, Transaction};

pub fn filter(
    predicate: &Vec<String>,
    transaction: &Transaction<Posting>,
    posting: &Posting,
    real: bool,
) -> bool {
    let name = posting.account.get_name().to_lowercase();
    if real {
        if let PostingType::Real = posting.kind {
        } else {
            return false;
        }
    }
    if predicate.len() == 0 {
        return true;
    }
    for p in predicate {
        match name.find(p) {
            None => continue,
            Some(_) => return true,
        }
    }
    false
}
