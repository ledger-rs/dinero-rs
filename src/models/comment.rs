use crate::models::Tag;
use lazy_static::lazy_static;
use regex::Regex;
#[derive(Debug, Clone)]
pub struct Comment {
    pub comment: String,
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

        match RE_FLAGS.is_match(&self.comment) {
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
                        return vec![];
                    }
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
                false => Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HasName;
    #[test]
    fn multi_tag() {
        let comment = Comment {
            comment: ":tag_1:tag_2:tag_3:".to_string(),
        };
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 3, "There should be three tags");
        assert_eq!(tags[0].get_name(), "tag_1");
        assert_eq!(tags[1].get_name(), "tag_2");
        assert_eq!(tags[2].get_name(), "tag_3");
    }
    #[test]
    fn no_tag() {
        let comment = Comment {
            comment: ":tag_1:tag_2:tag_3: this is not valid".to_string(),
        };
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 0, "There should no tags");
    }
    #[test]
    fn not_a_tag() {
        let comment = Comment {
            comment: "not a tag whatsoever".to_string(),
        };
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 0, "There should no tags");
    }
    #[test]
    fn tag_value() {
        let comment = Comment {
            comment: "tag: value".to_string(),
        };
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 1, "There should be one tag");
        let tag = tags[0].clone();
        assert_eq!(tag.get_name(), "tag");
        assert_eq!(tag.value.unwrap(), "value".to_string());
    }
    #[test]
    fn tag_value_spaces() {
        let comment = Comment {
            comment: " tag: value with spaces".to_string(),
        };
        let tags = comment.get_tags();
        assert_eq!(tags.len(), 1, "There should be one tag");
        let tag = tags[0].clone();
        assert_eq!(tag.get_name(), "tag");
        assert_eq!(tag.value.unwrap(), "value with spaces".to_string());
    }
}
