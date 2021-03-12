use std::cell::RefCell;

use crate::models::Tag;
use lazy_static::lazy_static;
use regex::Regex;

use super::HasName;
#[derive(Debug, Clone)]
pub struct Comment {
    pub comment: String,
    calculated_tags: RefCell<bool>,
    tags: RefCell<Vec<Tag>>,
    calculated_payee: RefCell<bool>,
    payee: RefCell<Option<String>>,
}

impl From<String> for Comment {
    fn from(comment: String) -> Self {
        Comment {
            comment,
            calculated_tags: RefCell::new(false),
            tags: RefCell::new(vec![]),
            calculated_payee: RefCell::new(false),
            payee: RefCell::new(None),
        }
    }
}

impl From<&str> for Comment {
    fn from(comment: &str) -> Self {
        Comment::from(comment.to_string())
    }
}

impl Comment {
    pub fn get_tags(&self) -> Vec<Tag> {
        lazy_static! {
            static ref RE_FLAGS: Regex = Regex::new(format!("{}",
            r"(:.+:) *$" , // the tags
            ).as_str()).unwrap();
            static ref RE_TAG_VALUE: Regex = Regex::new(format!("{}",
            r" *(.*): *(.*) *$"
            ).as_str()).unwrap();
        }
        let calculated_tags = *self.calculated_tags.borrow_mut();
        let tags = if !calculated_tags {
            self.calculated_tags.replace(true);
            self.tags
                .borrow_mut()
                .append(&mut match RE_FLAGS.is_match(&self.comment) {
                    true => {
                        let value = RE_FLAGS
                            .captures(&self.comment)
                            .unwrap()
                            .iter()
                            .nth(1)
                            .unwrap()
                            .unwrap()
                            .as_str();
                        let mut tags: Vec<Tag> = value
                            .split(":")
                            .map(|x| Tag {
                                name: x.clone().to_string(),
                                check: vec![],
                                assert: vec![],
                                value: None,
                            })
                            .collect();
                        tags.pop();
                        tags.remove(0);
                        tags
                    }
                    false => match RE_TAG_VALUE.is_match(&self.comment) {
                        true => {
                            let captures = RE_TAG_VALUE.captures(&self.comment).unwrap();
                            let name: String = captures
                                .iter()
                                .nth(1)
                                .unwrap()
                                .unwrap()
                                .as_str()
                                .to_string();
                            if name.contains(":") {
                                vec![]
                            } else {
                                vec![Tag {
                                    name,
                                    check: vec![],
                                    assert: vec![],
                                    value: Some(
                                        captures
                                            .iter()
                                            .nth(2)
                                            .unwrap()
                                            .unwrap()
                                            .as_str()
                                            .to_string(),
                                    ),
                                }]
                            }
                        }
                        false => vec![],
                    },
                });
            self.tags.borrow().clone()
        } else {
            self.tags.borrow().clone()
        };
        tags
    }

    pub fn get_payee_str(&self) -> Option<String> {
        let calculated_payee = *self.calculated_payee.borrow_mut();
        if calculated_payee {
            return self.payee.borrow().clone();
        }
        self.calculated_payee.replace(true);
        for tag in self.get_tags().iter() {
            if tag.value.is_none() | (tag.get_name().to_lowercase() != "payee") {
                continue;
            }
            self.payee.replace(tag.value.clone());
            return tag.value.clone();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HasName;
    #[test]
    fn multi_tag() {
        let comment = Comment::from(":tag_1:tag_2:tag_3:");
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 3, "There should be three tags");
        assert_eq!(tags[0].get_name(), "tag_1");
        assert_eq!(tags[1].get_name(), "tag_2");
        assert_eq!(tags[2].get_name(), "tag_3");
    }
    #[test]
    fn no_tag() {
        let comment = Comment::from(":tag_1:tag_2:tag_3: this is not valid");
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 0, "There should no tags");
    }
    #[test]
    fn not_a_tag() {
        let comment = Comment::from("not a tag whatsoever");
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 0, "There should no tags");
    }
    #[test]
    fn tag_value() {
        let comment = Comment::from("tag: value");
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 1, "There should be one tag");
        let tag = tags[0].clone();
        assert_eq!(tag.get_name(), "tag");
        assert_eq!(tag.value.unwrap(), "value".to_string());
    }
    #[test]
    fn tag_value_spaces() {
        let comment = Comment::from("tag: value with spaces");
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 1, "There should be one tag");
        let tag = tags[0].clone();
        assert_eq!(tag.get_name(), "tag");
        assert_eq!(tag.value.unwrap(), "value with spaces".to_string());
    }
}
