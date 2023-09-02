use std::fmt::Display;

use chrono::NaiveDate;

use super::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section<'a> {
    pub title: Token<'a>,
    pub section_type: SectionType,
    pub tags: Vec<String>,
    pub date: NaiveDate,
    pub content: Vec<Token<'a>>,
    pub subsections: Vec<Section<'a>>,
}

impl<'a> Section<'a> {
    pub fn contains_tag(&self, tag: String) -> bool {
        if self.tags.contains(&tag) {
            return true;
        }
        for subsection in &self.subsections {
            if subsection.contains_tag(tag.clone()) {
                return true;
            }
        }
        false
    }
}

impl<'a> Display for Section<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s += &self.title.to_markdown_string();

        for c in &self.content {
            s += &c.to_markdown_string();
        }
        for sub in &self.subsections {
            s += &sub.to_string();
        }
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SectionType {
    H1,
    H2,
    H3,
    H4,
}