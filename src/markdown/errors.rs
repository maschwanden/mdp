use std::{error::Error, fmt::Display};

use nom::error::{ErrorKind, ParseError};

use crate::models::MDPError;

#[derive(Debug, PartialEq)]
pub(super) enum MarkdownParseError<I> {
    InvalidRawURL,
    InvalidEmailAddress,
    InvalidMarkdownHeading,
    InvalidISO8601Date,
    UnbalancedBracketCount,
    IncompleteInput,
    Nom(I, nom::error::ErrorKind),
}

impl<I> ParseError<I> for MarkdownParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        MarkdownParseError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl Display for MarkdownParseError<&str> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::InvalidEmailAddress => {
                "The input could not be interpreted as an email address".to_string()
            }
            Self::InvalidISO8601Date => {
                "The input could not be interpreted as an ISO08601 date".to_string()
            }
            Self::InvalidMarkdownHeading => {
                "The input could not be interpreted as a Markdown heading".to_string()
            }
            Self::InvalidRawURL => "The input could not be interpreted as an URL".to_string(),
            Self::UnbalancedBracketCount => "The input contains unbalanced brackets".to_string(),
            Self::IncompleteInput => "Not enough input was given.".to_string(),
            Self::Nom(i, errorkind) => {
                format!("A Nom error occured for input '{}': {:?}", i, errorkind)
            }
        };
        write!(f, "{}", msg)
    }
}

impl Error for MarkdownParseError<&str> {}

impl MarkdownParseError<&str> {
    pub fn into_mdp_error(self, line_number: usize) -> MDPError {
        MDPError::MarkdownParseError {
            msg: self.to_string(),
            line_number,
        }
    }
}
