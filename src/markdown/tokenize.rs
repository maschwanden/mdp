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

        for (line_number, line) in split_into_lines(markdown_string).drain(..).enumerate() {
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


#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::models::TaskStatus;

    use super::*;

    #[test]
    fn test_mdp_markdown_tokenizer() -> Result<()> {
        let markdown_string = r#"
# 2022-11-02

## School

@school

Today in school something happened.

## Freetime

After school I went home

DONE: Clean room

---

# 2022-11-03

## Meeting

In the morning i had a meeting with @roger (roger.example@gmail.com).

TODO: Inform roger about the state of the project
        "#;

        let mdp_tokenizer = MDPMarkdownTokenizer {};
        let should_tokens = vec![
            Token::Blank,
            Token::Newline,
            Token::HeadingH1(vec![
                Token::Date(NaiveDate::from_ymd_opt(2022, 11, 2).unwrap())
            ]),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::HeadingH2(vec![Token::Text("School")]),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::Tag("school"),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::Text("Today in school something happened."),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::HeadingH2(vec![Token::Text("Freetime")]),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::Text("After school I went home"),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::Task { content: vec![Token::Text("Clean room")], status: TaskStatus::Done },
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::HRule,
            Token::Newline,
            Token::Blank,
            Token::Newline,Token::HeadingH1(vec![
                Token::Date(NaiveDate::from_ymd_opt(2022, 11, 3).unwrap())
            ]),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::HeadingH2(vec![Token::Text("Meeting")]),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::Text("In the morning i had a meeting with "),
            Token::Tag("roger"),
            Token::Text(" ("),
            Token::Email("roger.example@gmail.com"),
            Token::Text(")."),
            Token::Newline,
            Token::Blank,
            Token::Newline,
            Token::Task { 
                content: vec![Token::Text("Inform roger about the state of the project")], 
                status: TaskStatus::Todo, 
            },
            Token::Newline,
            Token::Blank,
            Token::Newline,
        ];

        assert_eq!(
            mdp_tokenizer.tokenize(&markdown_string),
            Ok(should_tokens),
        );
        Ok(())
    }
}