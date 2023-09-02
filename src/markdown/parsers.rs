use core::str;
use std::vec;

use chrono::{NaiveDate, Weekday};

use email_address_parser::EmailAddress;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_while1},
    character::{
        complete::{char, multispace1},
        is_newline,
    },
    combinator::{map, map_parser},
    multi::many1_count,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use urlocator::{UrlLocation, UrlLocator};

use super::errors::MarkdownParseError;
use crate::models::{Token, TaskStatus};

/// Take a string delimited by some characters, but track how many times the delimiter pairs
/// themselves also appear in the string.
/// From https://gitlab.com/getreu/parse-hyperlinks/-/blob/master/parse-hyperlinks/src/lib.rs
fn take_until_unbalanced(
    opening_bracket: char,
    closing_bracket: char,
) -> impl Fn(&str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    move |i: &str| {
        let mut index = 0;
        let mut bracket_counter = 0;
        while let Some(n) = &i[index..].find(&[opening_bracket, closing_bracket, '\\'][..]) {
            index += n;
            let mut it = i[index..].chars();
            match it.next().unwrap_or_default() {
                c if c == '\\' => {
                    // Skip the escape char `\`.
                    index += '\\'.len_utf8();
                    // Skip also the following char.
                    let c = it.next().unwrap_or_default();
                    index += c.len_utf8();
                }
                c if c == opening_bracket => {
                    bracket_counter += 1;
                    index += opening_bracket.len_utf8();
                }
                c if c == closing_bracket => {
                    // Closing bracket.
                    bracket_counter -= 1;
                    index += closing_bracket.len_utf8();
                }
                // Can not happen.
                _ => unreachable!(),
            };
            // We found the unmatched closing bracket.
            if bracket_counter == -1 {
                // We do not consume it.
                index -= closing_bracket.len_utf8();
                return Ok((&i[index..], &i[0..index]));
            };
        }

        if bracket_counter == 0 {
            Ok(("", i))
        } else {
            Err(nom::Err::Error(MarkdownParseError::UnbalancedBracketCount))
        }
    }
}

fn nonws_char(c: char) -> bool {
    !c.is_whitespace() && !is_newline(c as u8)
}

fn is_word_finish_char(c: char) -> bool {
    match c {
        ',' | '.' | ':' | ';' | ')' | ']' => true,
        _ => false
    }
}

fn word(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    take_while1(|c| nonws_char(c) && !is_word_finish_char(c))(input)
}

fn fenced<'a>(
    start: &'a str,
    end: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, MarkdownParseError<&str>> {
    map(tuple((tag(start), take_until(end), tag(end))), |x| x.1)
}

fn style<'a>(
    boundary: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Token<'a>>, MarkdownParseError<&str>> {
    map_parser(fenced(boundary, boundary), parse_inline)
}

fn link(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    fenced("[[", "]]")(input)
}

fn markdown_link(input: &str) -> IResult<&str, (&str, &str), MarkdownParseError<&str>> {
    pair(
        fenced("[", "]"),
        delimited(char('('), take_until_unbalanced('(', ')'), char(')')),
    )(input)
}

fn link_or_word(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    alt((link, word))(input)
}

fn hashtag(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    preceded(char('#'), link_or_word)(input)
}

fn triple_backtick(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    fenced("```", "```")(input)
}

fn single_backtick(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    delimited(char('`'), is_not("`"), char('`'))(input)
}

// Parse `((refrence))`
fn block_ref(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    fenced("((", "))")(input)
}

fn bold(input: &str) -> IResult<&str, Vec<Token>, MarkdownParseError<&str>> {
    style("**")(input)
}

fn italic(input: &str) -> IResult<&str, Vec<Token>, MarkdownParseError<&str>> {
    style("*")(input)
}

fn strike(input: &str) -> IResult<&str, Vec<Token>, MarkdownParseError<&str>> {
    style("~~")(input)
}

fn highlight(input: &str) -> IResult<&str, Vec<Token>, MarkdownParseError<&str>> {
    style("^^")(input)
}

fn latex(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    fenced("$$", "$$")(input)
}

fn image(input: &str) -> IResult<&str, (&str, &str), MarkdownParseError<&str>> {
    preceded(char('!'), markdown_link)(input)
}

fn email(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    const MIN_EMAIL_LENGTH: usize = 5;
    // Set upper limit to email length in case of very long input
    // -> Only consider the first 50 characters of input
    const MAX_EMAIL_LENGTH: usize = 50;

    let considered_input = match input.get(..MAX_EMAIL_LENGTH) {
        Some(s) => s,
        None => input,
    };

    for i in (MIN_EMAIL_LENGTH..considered_input.len() + 1).rev() {
        // Note: These unwraps are safe because we know that input/considered_input is
        // at least `max_email_length` long.
        let current_input = match considered_input.get(..i) {
            Some(s) => s,
            None => continue,
        };

        // Note: To exclude some false positives we ignore the fact that email address 
        // can contain whitespaces!
        if current_input.contains(" ") {
            continue;
        }

        if EmailAddress::parse(current_input, None).is_some() {
            let remaining_input = input.get(i..).unwrap_or("");
            return Ok((remaining_input, current_input));
        }
    }
    Err(nom::Err::Error(MarkdownParseError::InvalidEmailAddress))
}

fn tag_token(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    preceded(char('@'), word)(input)
}

fn raw_url(input: &str) -> IResult<&str, &str, MarkdownParseError<&str>> {
    let mut locator = UrlLocator::new();
    let mut end = 0;
    for c in input.chars() {
        match locator.advance(c) {
            UrlLocation::Url(s, _e) => {
                end = s as usize;
            }
            UrlLocation::Reset => break,
            UrlLocation::Scheme => {}
        }
    }

    if end > 0 {
        Ok((&input[end..], &input[0..end]))
    } else {
        Err(nom::Err::Error(MarkdownParseError::InvalidRawURL))
    }
}

fn directive(input: &str) -> IResult<&str, Token, MarkdownParseError<&str>> {
    alt((
        map(markdown_link, |(title, url)| {
            if url.starts_with('#') {
                Token::MarkdownInternalLink {
                    label: title,
                    link: url,
                }
            } else {
                Token::MarkdownExternalLink { title, url }
            }
        }),
        map(date, Token::Date),
        map(email, Token::Email),
        map(tag_token, Token::Tag),
        map(triple_backtick, Token::TripleBacktick),
        map(single_backtick, Token::SingleBacktick),
        map(hashtag, Token::Hashtag),
        map(block_ref, Token::BlockRef),
        map(image, |(alt, url)| Token::Image { alt, url }),
        map(link, Token::Link),
        map(bold, Token::Bold),
        map(italic, Token::Italic),
        map(strike, Token::Strike),
        map(highlight, Token::Highlight),
        map(latex, Token::Latex),
        map(raw_url, Token::RawHyperlink),
    ))(input)
}

/// Parse a line of text, counting anything that doesn't match a directive as plain text.
pub(super) fn parse_inline(
    input: &str,
) -> IResult<&str, Vec<Token>, MarkdownParseError<&str>> {
    let mut output = Vec::with_capacity(4);

    let mut current_input = input;

    while !current_input.is_empty() {
        let mut found_directive = false;
        for (current_index, _) in current_input.char_indices() {
            match directive(&current_input[current_index..]) {
                Ok((remaining, parsed)) => {
                    let leading_text = &current_input[0..current_index];
                    if !leading_text.is_empty() {
                        output.push(Token::Text(leading_text));
                    }
                    output.push(parsed);

                    current_input = remaining;
                    found_directive = true;
                    break;
                }
                Err(nom::Err::Error(_)) => {
                    // None of the parsers matched at the current position, so this character is just part of the text.
                    // The iterator will go to the next character so there's nothing to do here.
                }
                Err(e) => {
                    // On any other error, just return the error.
                    return Err(e);
                }
            }
        }

        if !found_directive {
            output.push(Token::Text(current_input));
            break;
        }
    }

    Ok(("", output))
}

/// Parses `Name:: Arbitrary [[text]]`
pub(super) fn attribute(
    input: &str,
) -> IResult<&str, (&str, Vec<Token>), MarkdownParseError<&str>> {
    separated_pair(is_not(":`"), tag("::"), parse_inline)(input)
}

pub(super) fn task(input: &str) -> IResult<&str, Token, MarkdownParseError<&str>> {
    let (task_description, task) = terminated(
        alt((
            map(tag("TODO:"), |_| Token::Task { content: vec![], status: TaskStatus::Todo }),
            map(tag("DOING:"), |_| Token::Task { content: vec![], status: TaskStatus::Doing }),
            map(tag("REVIEW:"), |_| Token::Task { content: vec![], status: TaskStatus::Review }),
            map(tag("DONE:"), |_| Token::Task { content: vec![], status: TaskStatus::Done }),
            map(tuple((tag("TODO UNTIL "), date, tag(":"))), |(_, d, _)| Token::Task { content: vec![], status: TaskStatus::TodoUntil(d) })
        )),
        multispace1,
    )(input)?;

    match task {
        Token::Task { status, .. } => {
            let (_, content) = parse_inline(task_description)?;
            Ok(("", Token::Task { content, status }))
        },
        _ => unreachable!(),
    }
   
}

pub(super) fn heading(input: &str) -> IResult<&str, Token, MarkdownParseError<&str>> {
    let (content_raw, hashtag_count) = terminated(many1_count(tag("#")), multispace1)(input)?;

    let (i, content) = parse_inline(content_raw)?;
    match hashtag_count {
        1 => Ok((i, Token::HeadingH1(content))),
        2 => Ok((i, Token::HeadingH2(content))),
        3 => Ok((i, Token::HeadingH3(content))),
        4 => Ok((i, Token::HeadingH4(content))),
        _ => Err(nom::Err::Error(MarkdownParseError::InvalidMarkdownHeading)),
    }
}

pub(super) fn date(input: &str) -> IResult<&str, NaiveDate, MarkdownParseError<&str>> {
    let (i, iso_date) = iso8601::parsers::parse_date(input.as_bytes())
        .map_err(|_| nom::Err::Error(MarkdownParseError::InvalidISO8601Date))?;

    let date_opt: Option<NaiveDate> = match iso_date {
        iso8601::Date::YMD { year, month, day } => NaiveDate::from_ymd_opt(year, month, day),
        iso8601::Date::Week { year, ww, d } => {
            NaiveDate::from_isoywd_opt(year, ww, Weekday::try_from(d as u8).unwrap())
        }
        iso8601::Date::Ordinal { year, ddd } => NaiveDate::from_yo_opt(year, ddd),
    };

    match date_opt {
        Some(date) => Ok((str::from_utf8(i).unwrap(), date)),
        None => Err(nom::Err::Error(MarkdownParseError::InvalidISO8601Date)),
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_inline_markdown_internal_link() {
        let (remaining_input, tokens) = parse_inline("[link123](#section123)").unwrap();
        assert_eq!(
            tokens,
            vec![Token::MarkdownInternalLink {
                label: "link123",
                link: "#section123",
            }]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_markdown_link() {
        let (remaining_input, tokens) = parse_inline("[link123](#section123)").unwrap();
        assert_eq!(
            tokens,
            vec![Token::MarkdownInternalLink {
                label: "link123",
                link: "#section123",
            }]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_markdown_external_link() {
        let (remaining_input, tokens) = parse_inline("[link123](www.google.com)").unwrap();
        assert_eq!(
            tokens,
            vec![Token::MarkdownExternalLink {
                title: "link123",
                url: "www.google.com",
            }]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_date() {
        let (remaining_input, tokens) = parse_inline("2013-03-08").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Date(
                NaiveDate::from_ymd_opt(2013, 3, 8).unwrap()
            )]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_email() {
        let (remaining_input, tokens) = parse_inline("mathias.aschwanden@gmail.com").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Email("mathias.aschwanden@gmail.com")]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_tag_token() {
        let (remaining_input, tokens) = parse_inline("@rega @bafu").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Tag("rega"),
                Token::Text(" "),
                Token::Tag("bafu")
            ]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_triple_backtick() {
        let (remaining_input, tokens) = parse_inline("```import sys```").unwrap();
        assert_eq!(tokens, vec![Token::TripleBacktick("import sys")]);
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_single_backtick() {
        let (remaining_input, tokens) = parse_inline("`import sys`").unwrap();
        assert_eq!(tokens, vec![Token::SingleBacktick("import sys")]);
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_hashtag() {
        let (remaining_input, tokens) = parse_inline("#hallo").unwrap();
        assert_eq!(tokens, vec![Token::Hashtag("hallo")]);
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_block_ref() {
        let (remaining_input, tokens) = parse_inline("((ref123))").unwrap();
        assert_eq!(tokens, vec![Token::BlockRef("ref123")]);
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_image() {
        let (remaining_input, tokens) = parse_inline("![alt](url)").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Image {
                alt: "alt",
                url: "url"
            }]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_link() {
        let (remaining_input, tokens) = parse_inline("[[link123]]").unwrap();
        assert_eq!(tokens, vec![Token::Link("link123")]);
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_bold() {
        let (remaining_input, tokens) = parse_inline("**Haha**").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Bold(vec![Token::Text("Haha")])]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_italic() {
        let (remaining_input, tokens) = parse_inline("*Haha* Some other text").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Italic(vec![Token::Text("Haha")]),
                Token::Text(" Some other text")
            ]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_strike() {
        let (remaining_input, tokens) = parse_inline("~~link123~~").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Strike(vec![Token::Text(
                "link123"
            )])]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_highlight() {
        let (remaining_input, tokens) = parse_inline("^^link123^^").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Highlight(vec![Token::Text(
                "link123"
            )])]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_latex() {
        let (remaining_input, tokens) = parse_inline("$$link123$$").unwrap();
        assert_eq!(tokens, vec![Token::Latex("link123")]);
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_parse_inline_raw_hyperlink() {
        let (remaining_input, tokens) = parse_inline("https://example.org").unwrap();
        assert_eq!(
            tokens,
            vec![Token::RawHyperlink("https://example.org")]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_attribute() {
        let (remaining_input, (attribute_name, tokens)) =
            attribute("attr123:: https://google.com").unwrap();
        assert_eq!(attribute_name, "attr123",);
        assert_eq!(
            tokens,
            vec![
                Token::Text(" "),
                Token::RawHyperlink("https://google.com")
            ]
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_task_todo() {
        let (remaining_input, tokens) = task("TODO: here comes the task").unwrap();
        assert_eq!(
            tokens,
            Token::Task {
                content: vec![Token::Text("here comes the task")],
                status: TaskStatus::Todo
            },
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_task_todo_until() {
        let (remaining_input, tokens) = task("TODO UNTIL 2023-10-10: here comes the task").unwrap();
        assert_eq!(
            tokens,
            Token::Task {
                content: vec![Token::Text("here comes the task")],
                status: TaskStatus::TodoUntil(NaiveDate::from_ymd_opt(2023, 10, 10).unwrap())
            },
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_task_doing() {
        let (remaining_input, tokens) = task("DOING: here comes the task").unwrap();
        assert_eq!(
            tokens,
            Token::Task {
                content: vec![Token::Text("here comes the task")],
                status: TaskStatus::Doing
            },
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_task_review() {
        let (remaining_input, tokens) = task("REVIEW: here comes the task").unwrap();
        assert_eq!(
            tokens,
            Token::Task {
                content: vec![Token::Text("here comes the task")],
                status: TaskStatus::Review
            },
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_task_done() {
        let (remaining_input, tokens) = task("DONE: here comes the task").unwrap();
        assert_eq!(
            tokens,
            Token::Task {
                content: vec![Token::Text("here comes the task")],
                status: TaskStatus::Done
            },
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_heading_h1() {
        let (remaining_input, tokens) = heading("# Titel").unwrap();
        assert_eq!(
            tokens,
            Token::HeadingH1(vec![Token::Text("Titel")]),
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_heading_h2() {
        let (remaining_input, tokens) = heading("## Titel").unwrap();
        assert_eq!(
            tokens,
            Token::HeadingH2(vec![Token::Text("Titel")]),
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_heading_h3() {
        let (remaining_input, tokens) = heading("### Titel").unwrap();
        assert_eq!(
            tokens,
            Token::HeadingH3(vec![Token::Text("Titel")]),
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_heading_h4() {
        let (remaining_input, tokens) = heading("#### Titel").unwrap();
        assert_eq!(
            tokens,
            Token::HeadingH4(vec![Token::Text("Titel")]),
        );
        assert_eq!(remaining_input, "");
    }

    #[test]
    fn test_heading_invalid() {
        let res = heading("##### Titel");
        assert!(res.is_err());
    }

    #[test]
    fn test_date() {
        assert_eq!(
            date("2010-01-10"),
            Ok(("", NaiveDate::from_ymd_opt(2010, 1, 10).unwrap())),
        );
        assert_eq!(
            date("2010-010"),
            Ok(("", NaiveDate::from_ymd_opt(2010, 1, 10).unwrap())),
        );

        assert_eq!(
            date("2010-01-40"),
            Err(nom::Err::Error(MarkdownParseError::InvalidISO8601Date)),
        );
    }

    #[test]
    fn test_word() {
        assert_eq!(
            word("roger"),
            Ok(("", "roger")),
        );

        assert_eq!(
            word("roger."),
            Ok((".", "roger")),
        );
    }
}
