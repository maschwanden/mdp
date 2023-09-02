use super::{MDPError, Sections, Token};

pub trait MarkdownTokenizer {
    /// Tokenize Markdown string into Markdown tokens
    fn tokenize<'a>(&self, markdown_string: &'a str) -> Result<Vec<Token<'a>>, MDPError>;
}

pub trait SectionBuilder {
    /// Create sections from Markdown tokens
    fn sections_from_tokens<'a>(&self, tokens: Vec<Token<'a>>) -> Result<Sections<'a>, MDPError>;
}
