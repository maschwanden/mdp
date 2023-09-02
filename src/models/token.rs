use std::fmt::Display;

use chrono::NaiveDate;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Blank,
    HRule,
    Newline,

    BlockRef(&'a str),
    Email(&'a str),
    Hashtag(&'a str),
    Latex(&'a str),
    Link(&'a str),
    Text(&'a str),
    RawHyperlink(&'a str),
    SingleBacktick(&'a str),
    Tag(&'a str),
    TripleBacktick(&'a str),

    Date(NaiveDate),

    BlockQuote(Vec<Token<'a>>),
    Bold(Vec<Token<'a>>),
    Highlight(Vec<Token<'a>>),
    Italic(Vec<Token<'a>>),
    Strike(Vec<Token<'a>>),
    HeadingH1(Vec<Token<'a>>),
    HeadingH2(Vec<Token<'a>>),
    HeadingH3(Vec<Token<'a>>),
    HeadingH4(Vec<Token<'a>>),

    Attribute {
        name: &'a str,
        value: Vec<Token<'a>>,
    },
    Image {
        alt: &'a str,
        url: &'a str,
    },
    MarkdownInternalLink {
        label: &'a str,
        link: &'a str,
    },
    MarkdownExternalLink {
        title: &'a str,
        url: &'a str,
    },
    Task {
        content: Vec<Token<'a>>,
        status: TaskStatus,
    },
}

impl<'a> Token<'a> {
    pub fn to_debug_string(&self) -> String {
        match self {
            Token::Blank => "<Blank>".to_string(),
            Token::HRule => "<HRule>".to_string(),
            Token::Newline => "<Newline>".to_string(),

            Token::BlockRef(s) => format!("<BlockRef: '{}'>", s),
            Token::Email(s) => format!("<Email: '{}'>", s),
            Token::Hashtag(s) => format!("<Hashtag: '{}'>", s),
            Token::Latex(s) => format!("<Latex: '{}'>", s),
            Token::Link(s) => format!("<Link: '{}'>", s),
            Token::RawHyperlink(s) => format!("<RawHyperlink: '{}'>", s),
            Token::SingleBacktick(s) => format!("<SingleBacktick: '{}'>", s),
            Token::Tag(s) => format!("<Tag: '{}'>", s),
            Token::Text(s) => format!("<Text: '{}'>", s),
            Token::TripleBacktick(s) => format!("<TripleBacktick: '{}'>", s),

            Token::Date(date) => format!("<Date: '{}'>", date.format("%Y-%m-%d")),

            Token::BlockQuote(tokens) => {
                format!(
                    "<BlockQuote: '{}'>",
                    Self::child_tokens_as_debug_string(tokens),
                )
            }
            Token::Bold(tokens) => {
                format!("<Bold: '{}'>", Self::child_tokens_as_debug_string(tokens),)
            }
            Token::Highlight(tokens) => {
                format!(
                    "<Highlight: '{}'>",
                    Self::child_tokens_as_debug_string(tokens),
                )
            }
            Token::Italic(tokens) => {
                format!("<Italic: '{}'>", Self::child_tokens_as_debug_string(tokens),)
            }
            Token::Strike(tokens) => {
                format!("<Strike: '{}'>", Self::child_tokens_as_debug_string(tokens),)
            }
            Token::HeadingH1(tokens) => {
                format!(
                    "<HeadingH1: '{}'>",
                    Self::child_tokens_as_debug_string(tokens)
                )
            }
            Token::HeadingH2(tokens) => {
                format!(
                    "<HeadingH2: '{}'>",
                    Self::child_tokens_as_debug_string(tokens)
                )
            }
            Token::HeadingH3(tokens) => {
                format!(
                    "<HeadingH3: '{}'>",
                    Self::child_tokens_as_debug_string(tokens)
                )
            }
            Token::HeadingH4(tokens) => {
                format!(
                    "<HeadingH4: '{}'>",
                    Self::child_tokens_as_debug_string(tokens)
                )
            }

            Token::Attribute { name, value } => {
                format!(
                    "<Attribute: '{}::{}'>",
                    name,
                    Self::child_tokens_as_debug_string(value)
                )
            }
            Token::Image { alt, url } => format!("<Image: '[{}]({})'>", alt, url),
            Token::MarkdownExternalLink { title, url } => {
                format!("<MarkdownExternalLink: '[{}]({})'>", title, url)
            }
            Token::MarkdownInternalLink { label, link } => {
                format!("<MarkdownInternalLink: '[{}]({})'>", label, link)
            }
            Token::Task { content, status } => format!(
                "<Task({}): {}>",
                status,
                Self::child_tokens_as_debug_string(content),
            ),
        }
    }

    fn child_tokens_as_debug_string(tokens: &[Token<'a>]) -> String {
        tokens
            .iter()
            .map(|t| t.to_debug_string())
            .collect::<String>()
    }

    pub fn to_markdown_string(&self) -> String {
        match self {
            Token::Blank => "".to_string(),
            Token::HRule => "---".to_string(),
            Token::Newline => "\n".to_string(),

            Token::BlockRef(s) => format!("(({}))", s),
            Token::Email(s) => s.to_string(),
            Token::Hashtag(s) => format!("#{}", s),
            Token::Latex(s) => format!("$${}$$", s),
            Token::Link(s) => format!("[[{}]]", s),
            Token::RawHyperlink(s) => s.to_string(),
            Token::SingleBacktick(s) => format!("`{}`", s),
            Token::Tag(s) => format!("@{}", s),
            Token::Text(s) => s.to_string(),
            Token::TripleBacktick(s) => format!("```{}```", s),

            Token::Date(date) => format!("{}", date.format("%Y-%m-%d")),

            Token::BlockQuote(tokens) => {
                format!("> {}", Self::child_tokens_as_markdown_string(tokens),)
            }
            Token::Bold(tokens) => {
                format!("**{}**", Self::child_tokens_as_markdown_string(tokens),)
            }
            Token::Highlight(tokens) => {
                format!("^^{}^^", Self::child_tokens_as_markdown_string(tokens),)
            }
            Token::Italic(tokens) => {
                format!("*{}*", Self::child_tokens_as_markdown_string(tokens),)
            }
            Token::Strike(tokens) => {
                format!("~~{}~~", Self::child_tokens_as_markdown_string(tokens),)
            }
            Token::HeadingH1(tokens) => {
                format!("# {}", Self::child_tokens_as_markdown_string(tokens))
            }
            Token::HeadingH2(tokens) => {
                format!("## {}", Self::child_tokens_as_markdown_string(tokens))
            }
            Token::HeadingH3(tokens) => {
                format!("### {}", Self::child_tokens_as_markdown_string(tokens))
            }
            Token::HeadingH4(tokens) => {
                format!("#### {}", Self::child_tokens_as_markdown_string(tokens))
            }

            Token::Attribute { name, value } => {
                format!("{}::{}", name, Self::child_tokens_as_markdown_string(value))
            }
            Token::Image { alt, url } => format!("![{}]({})", alt, url),
            Token::MarkdownExternalLink { title, url } => format!("[{}]({})", title, url),
            Token::MarkdownInternalLink { label, link } => {
                format!("[{}]({})", label, link)
            }
            Token::Task { content, status } => format!(
                "{}: {}",
                status,
                Self::child_tokens_as_markdown_string(content),
            ),
        }
    }

    fn child_tokens_as_markdown_string(tokens: &[Token<'a>]) -> String {
        tokens
            .iter()
            .map(|t| t.to_markdown_string())
            .collect::<String>()
    }

    pub fn token_type(&self) -> TokenType {
        match self {
            Token::Blank => TokenType::Blankline,
            Token::HRule => TokenType::HRule,
            Token::Newline => TokenType::Newline,

            Token::BlockRef(_) => TokenType::BlockRef,
            Token::Email(_) => TokenType::Email,
            Token::Hashtag(_) => TokenType::Hashtag,
            Token::Latex(_) => TokenType::Latex,
            Token::Link(_) => TokenType::Link,
            Token::RawHyperlink(_) => TokenType::RawHyperlink,
            Token::SingleBacktick(_) => TokenType::SingleBacktick,
            Token::Tag(_) => TokenType::Tag,
            Token::Text(_) => TokenType::Text,
            Token::TripleBacktick(_) => TokenType::TripleBacktick,

            Token::Date(_) => TokenType::Date,

            Token::BlockQuote(_) => TokenType::BlockQuote,
            Token::Bold(_) => TokenType::Bold,
            Token::Highlight(_) => TokenType::Highlight,
            Token::Italic(_) => TokenType::Italic,
            Token::Strike(_) => TokenType::Strike,
            Token::HeadingH1(_) => TokenType::HeadingH1,
            Token::HeadingH2(_) => TokenType::HeadingH2,
            Token::HeadingH3(_) => TokenType::HeadingH3,
            Token::HeadingH4(_) => TokenType::HeadingH4,

            Token::Attribute { .. } => TokenType::Attribute,
            Token::Image { .. } => TokenType::Image,
            Token::MarkdownExternalLink { .. } => TokenType::MarkdownInternalLink,
            Token::MarkdownInternalLink { .. } => TokenType::MarkdownInternalLink,
            Token::Task { .. } => TokenType::Task,
        }
    }

    pub fn contains(&self, token: &Self) -> bool {
        match self {
            Token::BlockQuote(tokens)
            | Token::Bold(tokens)
            | Token::Highlight(tokens)
            | Token::Italic(tokens)
            | Token::Strike(tokens)
            | Token::HeadingH1(tokens)
            | Token::HeadingH2(tokens)
            | Token::HeadingH3(tokens)
            | Token::HeadingH4(tokens)
            | Token::Attribute { value: tokens, .. }
            | Token::Task {
                content: tokens, ..
            } => {
                let mut found = false;
                for t in tokens {
                    if t == token || t.contains(token) {
                        found = true;
                    }
                }
                found
            }
            t => t == token,
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_markdown_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    TodoUntil(NaiveDate),
    Doing,
    Review,
    Done,
}

impl From<&TaskStatus> for String {
    fn from(task_status: &TaskStatus) -> String {
        match task_status {
            TaskStatus::Todo => "TODO".to_owned(),
            TaskStatus::TodoUntil(d) => format!("TODO UNTIL {}", d),
            TaskStatus::Doing => "DOING".to_owned(),
            TaskStatus::Review => "REVIEW".to_owned(),
            TaskStatus::Done => "DONE".to_owned(),
        }
    }
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    Blankline,
    HRule,
    Newline,

    BlockRef,
    Email,
    Hashtag,
    Latex,
    Link,
    Text,
    RawHyperlink,
    SingleBacktick,
    Tag,
    TripleBacktick,

    Date,

    BlockQuote,
    Bold,
    Highlight,
    Italic,
    Strike,
    HeadingH1,
    HeadingH2,
    HeadingH3,
    HeadingH4,

    Attribute,
    Image,
    MarkdownInternalLink,
    MarkdownExternalLink,
    Task,
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_display_blankline() {
        assert_eq!(Token::Blank.to_string(), "");
    }

    #[test]
    fn test_display_hrule() {
        assert_eq!(Token::HRule.to_string(), "---");
    }

    #[test]
    fn test_display_block_ref() {
        let input = "((abc))";
        assert_eq!(Token::BlockRef("abc").to_string(), input);
    }

    #[test]
    fn test_display_hashtag() {
        let input = "#tag";
        assert_eq!(Token::Hashtag("tag").to_string(), input)
    }

    #[test]
    fn test_display_latex() {
        let input = r#"$$x^2 = x \cdot x$$"#;
        assert_eq!(Token::Latex(r#"x^2 = x \cdot x"#).to_string(), input)
    }

    #[test]
    fn test_display_link() {
        let input = "[[Hallo]]";
        assert_eq!(Token::Link("Hallo").to_string(), input)
    }

    #[test]
    fn test_display_text() {
        let input = "abc";
        assert_eq!(Token::Text(input).to_string(), input);
    }

    #[test]
    fn test_display_raw_hyperlink() {
        let input = "www.google.com";
        assert_eq!(Token::RawHyperlink(input).to_string(), input);

        let input = "http://www.comed.ch";
        assert_eq!(Token::RawHyperlink(input).to_string(), input);
    }

    #[test]
    fn test_display_single_backtick() {
        let input = r"`javascript\nmap`";
        assert_eq!(Token::SingleBacktick(r"javascript\nmap").to_string(), input);
    }

    #[test]
    fn test_display_tag() {
        let input = "@tag1";
        assert_eq!(Token::Tag("tag1").to_string(), input);
    }

    #[test]
    fn test_display_triple_backticks() {
        let input = r##"```javascript\nmap $regex_domain $domain {\n  app defaultskin;\n  tm defaultskin;\n  www defaultskin;\n  '' defaultskin;\n  dev defaultskin;\n  default $regex_domain;\n}```"##;
        assert_eq!(Token::TripleBacktick(r##"javascript\nmap $regex_domain $domain {\n  app defaultskin;\n  tm defaultskin;\n  www defaultskin;\n  '' defaultskin;\n  dev defaultskin;\n  default $regex_domain;\n}"##,
        ).to_string(), input);
    }

    #[test]
    fn test_display_block_quote() {
        let input = "> This is a block quote [[link123]]";
        assert_eq!(
            Token::BlockQuote(vec![
                Token::Text("This is a block quote "),
                Token::Link("link123")
            ])
            .to_string(),
            input
        );
    }

    #[test]
    fn test_display_bold() {
        let input = "**abc**";
        assert_eq!(Token::Bold(vec![Token::Text("abc")]).to_string(), input);

        let _input = "**abc [spiped](https://www.tarsnap.com/spiped.html)**";
        assert_eq!(
            Token::Bold(vec![
                Token::Text("abc "),
                Token::MarkdownExternalLink {
                    title: "spiped",
                    url: "https://www.tarsnap.com/spiped.html",
                },
            ])
            .to_string(),
            "**abc [spiped](https://www.tarsnap.com/spiped.html)**"
        );
    }

    #[test]
    fn test_display_highlight() {
        let input = "^^abc^^";
        assert_eq!(
            Token::Highlight(vec![Token::Text("abc")]).to_string(),
            input
        );
    }

    #[test]
    fn test_display_italic() {
        let input = "*abc*";
        assert_eq!(Token::Italic(vec![Token::Text("abc")]).to_string(), input);
    }

    #[test]
    fn test_display_strike() {
        let input = "~~abc~~";
        assert_eq!(Token::Strike(vec![Token::Text("abc")]).to_string(), input);
    }

    #[test]
    fn test_display_attribute() {
        let _not_an_attribute = "Source:: some blog";

        let attribute = "Source::some blog";

        assert_eq!(
            Token::Attribute {
                name: "Source",
                value: vec![Token::Text("some blog")],
            }
            .to_string(),
            attribute,
        )
    }

    #[test]
    fn test_display_simple_heading() {
        let input = "# Source:: some blog";
        assert_eq!(
            Token::HeadingH1(vec![Token::Text("Source:: some blog")],).to_string(),
            input,
        )
    }

    #[test]
    fn test_display_image() {
        let input = "![](https://firebasestorage.googleapis.com/v0/b/firescript-577a2.appspot.com/o/some-id?abc)";
        assert_eq!(Token::Image { alt: "", url: "https://firebasestorage.googleapis.com/v0/b/firescript-577a2.appspot.com/o/some-id?abc" }.to_string(), input);
    }

    #[test]
    fn test_display_markdown_internal_link() {
        let input = "[Hallo](# title1)";
        assert_eq!(
            Token::MarkdownInternalLink {
                label: "Hallo",
                link: "# title1"
            }
            .to_string(),
            input
        )
    }

    #[test]
    fn test_display_markdown_external_link() {
        let input = "[Hallo](www.google.com)";
        assert_eq!(
            Token::MarkdownInternalLink {
                label: "Hallo",
                link: "www.google.com"
            }
            .to_string(),
            input
        )
    }

    #[test]
    fn test_display_task() {
        let input = r##"TODO: Get things done"##;
        assert_eq!(
            Token::Task {
                content: vec![Token::Text("Get things done")],
                status: TaskStatus::Todo,
            }
            .to_string(),
            input
        );

        let input = r##"DONE: Get things done"##;
        assert_eq!(
            Token::Task {
                content: vec![Token::Text("Get things done")],
                status: TaskStatus::Done,
            }
            .to_string(),
            input
        );
    }
}
