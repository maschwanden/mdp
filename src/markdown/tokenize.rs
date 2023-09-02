use std::vec;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{all_consuming, map},
    sequence::{pair, preceded},
};

use super::{
    errors::MarkdownParseError,
    parsers::{attribute, heading, parse_inline, task},
};
use crate::models::{MDPError, Token, MarkdownTokenizer};

pub struct MDPMarkdownTokenizer {}

impl MarkdownTokenizer for MDPMarkdownTokenizer {
    fn tokenize<'a>(&self, markdown_string: &'a str) -> Result<Vec<Token<'a>>, MDPError> {
        let mut errors: Vec<MDPError> = vec![];
        let mut markdown_elements: Vec<Token> = vec![];

        // let mut it = markdown_string.split('\n').peekable();
        for (line_number, line) in split_into_lines(markdown_string).drain(..).enumerate() {
            // while let Some(line) = it.next() {
            match parse_line(line).map_err(|e| e.into_mdp_error(line_number)) {
                Ok(elements) => markdown_elements.extend(elements),
                Err(e) => errors.push(e),
            }
            markdown_elements.push(Token::Newline)
        }

        if errors.is_empty() {
            Ok(markdown_elements)
        } else {
            Err(MDPError::MultiError(errors))
        }
    }
}

struct Line<'a>(&'a str);

impl<'a> From<Line<'a>> for &'a str {
    fn from(value: Line<'a>) -> Self {
        value.0
    }
}

fn split_into_lines(input: &str) -> Vec<Line<'_>> {
    input.split('\n').map(Line).collect()
}

// TODO: Instead of splitting line by line make a parser that parses newlines
fn parse_line(input: Line<'_>) -> Result<Vec<Token<'_>>, MarkdownParseError<&str>> {
    let r = alt((
        map(all_consuming(multispace0), |_| vec![Token::Blank]),
        map(all_consuming(tag("---")), |_| vec![Token::HRule]),
        map(all_consuming(preceded(tag("> "), parse_inline)), |values| {
            vec![Token::BlockQuote(values)]
        }),
        map(all_consuming(attribute), |(name, value)| {
            vec![Token::Attribute { name, value }]
        }),
        all_consuming(map(pair(task, parse_inline), |(todo_token, mut tokens)| {
            tokens.insert(0, todo_token);
            tokens
        })),
        all_consuming(map(heading, |h| vec![h])),
        all_consuming(parse_inline),
    ))(input.into());

    match r {
        Ok((_, tokens)) => Ok(tokens),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => Err(e),
        Err(nom::Err::Incomplete(_)) => Err(MarkdownParseError::IncompleteInput),
    }
}
