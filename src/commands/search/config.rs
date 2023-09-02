use std::{error::Error, fmt, path::PathBuf};

use chrono::NaiveDate;

#[derive(Clone, Debug)]
pub struct TagSearchConfig {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub ordering: SectionOrderingCriterion,
    pub search_terms: Vec<SearchTerm>,
    pub search_mode: TagSearchMode,
    pub from: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
}

#[derive(Clone, Debug)]
pub struct SearchTerm(String);

impl TryFrom<String> for SearchTerm {
    type Error = InvalidSearchTermError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // REVIEW: Check if more restrictions are neccessary for
        // a String to be a valid search term.
        if value.contains(char::is_whitespace) {
            return Err(InvalidSearchTermError(value));
        }
        Ok(Self(value))
    }
}

impl SearchTerm {
    pub fn inner(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug)]
pub struct InvalidSearchTermError(String);

impl fmt::Display for InvalidSearchTermError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The provided search term '{}' is invalid.", self.0)
    }
}

impl Error for InvalidSearchTermError {}

#[derive(Clone, Debug)]
pub enum TagSearchMode {
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum SectionOrderingCriterion {
    Relevance,
    Date,
}
