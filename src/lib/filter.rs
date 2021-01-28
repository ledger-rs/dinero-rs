use crate::models::{HasName, Posting, Transaction};

pub fn filter(
    predicate: &Vec<String>,
    transaction: &Transaction<Posting>,
    posting: &Posting,
) -> bool {
    let name = posting.account.get_name().to_lowercase();
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
