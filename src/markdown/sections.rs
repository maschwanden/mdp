use std::collections::VecDeque;

use crate::models::{
    MDPError, Token, TokenType, Section,
    SectionBuilder, SectionType, Sections,
};

use chrono::NaiveDate;
use std::vec;

#[derive(Clone, Debug)]
pub struct MDPSectionBuilder {}

impl SectionBuilder for MDPSectionBuilder {
    fn sections_from_tokens<'a>(
        &self,
        tokens: Vec<Token<'a>>,
    ) -> Result<Sections<'a>, MDPError> {
        let hierarchized_tokens = hierarchize_tokens_using_headings(tokens);
        sections_from_hierarchized_tokens(
            hierarchized_tokens, 
            None
        )
    }
}

fn sections_from_hierarchized_tokens(
    hierachical_tokens: Vec<HierarchicalToken>,
    parent_date: Option<NaiveDate>,
) -> Result<Sections, MDPError> {
    let mut sections: Vec<Section> = vec![];

    for token in hierachical_tokens {
        let (section_type, title_elements) = match &token.token {
            Token::HeadingH1(t) => (SectionType::H1, t),
            Token::HeadingH2(t) => (SectionType::H2, t),
            Token::HeadingH3(t) => (SectionType::H3, t),
            Token::HeadingH4(t) => (SectionType::H4, t),
            _ => continue,
        };
        let title_element = token.token.clone();

        let tags = token
            .children
            .iter()
            .filter_map(|t| match t.token {
                Token::Tag(s) => Some(s.to_owned()),
                _ => None,
            })
            .collect();

        let date = if let Some(d) = parent_date {
            d
        } else {
            let dates = title_elements
                .iter()
                .filter_map(|t| match t {
                    Token::Date(d) => Some(d.to_owned()),
                    _ => None,
                })
                .collect::<Vec<NaiveDate>>();

            match dates.len() {
                0 => {
                    return Err(MDPError::MDPSyntaxError(format!(
                        "The section title {} doesn't contain a date.",
                        title_element.to_markdown_string()
                    )))
                }
                1 => dates.first().unwrap().to_owned(),
                _ => {
                    return Err(MDPError::MDPSyntaxError(format!(
                        "The section title {} does contain more than one date.",
                        title_element.to_markdown_string()
                    )))
                }
            }
        };

        let mut content = vec![];
        for t in &token.children {
            match t.token {
                Token::HeadingH1(_)
                | Token::HeadingH2(_)
                | Token::HeadingH3(_)
                | Token::HeadingH4(_) => break,
                Token::HRule | Token::Blank => continue,
                _ => content.push(t.token.to_owned()),
            }
        }

        let subsections = sections_from_hierarchized_tokens(token.children, Some(date))?;

        sections.push(Section {
            section_type,
            title: token.token,
            tags,
            date,
            content,
            subsections,
        });
    }

    Ok(Sections::new(sections))
}

fn hierarchize_tokens_using_headings(tokens: Vec<Token>) -> Vec<HierarchicalToken> {
    let mut hierarchical_tokens = tokens.iter().cloned().map(HierarchicalToken::from_token).collect::<Vec<HierarchicalToken>>();

    let hierarchy = TokenHierarchy::from_token_types(vec![
        TokenType::HeadingH1,
        TokenType::HeadingH2,
        TokenType::HeadingH3,
        TokenType::HeadingH4,
    ]);

    let mut status = HierarchizeStatus::new(hierarchy.levels.len(), 10);

    loop {
        hierarchical_tokens = hierarchize_recursive_one_hierarchy_level(
            &hierarchy,
            hierarchical_tokens,
            status.clone(),
            status.is_hierarchy_root(),
        );
        status = match status.one_hierarchy_level_deeper() {
            Some(x) => x,
            None => break,
        };
    }
    hierarchical_tokens
}


fn hierarchize_recursive_one_hierarchy_level<'a>(
    hierarchy: &TokenHierarchy,
    hierachical_tokens: Vec<HierarchicalToken<'a>>,
    status: HierarchizeStatus,
    insert_blank_root_token: bool,
) -> Vec<HierarchicalToken<'a>> {
    if hierachical_tokens.is_empty() {
        return vec![];
    }

    let markdown_element_type = match hierarchy.token_type_at(status.hierarchy_level) {
        Some(x) => x,
        None => return hierachical_tokens,
    };
    let mut level_tokens: Vec<HierarchicalToken> = vec![];

    for group in split_at_markdown_element(hierachical_tokens.clone(), &markdown_element_type) {
        level_tokens.extend(hierarchize_one_group(
            hierarchy,
            group,
            &status,
            insert_blank_root_token,
        ));
    }

    let mut final_result = vec![];
    for t in level_tokens.iter_mut() {
        let next_status = match status.one_recursion_level_deeper() {
            Some(x) => x,
            None => {
                final_result.push(t.to_owned());
                continue;
            }
        };
        t.children = hierarchize_recursive_one_hierarchy_level(
            hierarchy,
            t.children.clone(),
            next_status,
            false,
        );
        final_result.push(t.to_owned());
    }
    final_result
}

fn hierarchize_one_group<'a>(
    hierarchy: &TokenHierarchy,
    hierachical_tokens: Vec<HierarchicalToken<'a>>,
    status: &HierarchizeStatus,
    insert_blank_root_token: bool,
) -> Vec<HierarchicalToken<'a>> {
    if hierachical_tokens.is_empty() {
        return vec![];
    }

    let mut root_tokens = vec![];

    let split_token_type = match hierarchy.token_type_at(status.hierarchy_level) {
        Some(x) => x,
        None => return hierachical_tokens,
    };
    let mut tokens = VecDeque::from(hierachical_tokens);

    match tokens.pop_front() {
        Some(mut token) => {
            if token.token.token_type() == split_token_type {
                let mut higher_hierarchy_tokens = vec![];
                for t in tokens.iter() {
                    if hierarchy.position(&t.token.token_type())
                        < hierarchy.position(&token.token.token_type())
                    {
                        higher_hierarchy_tokens.push(t.to_owned());
                    } else {
                        token.children.push(t.to_owned());
                    }
                }
                root_tokens.push(token);
                root_tokens.extend(higher_hierarchy_tokens);
            } else if insert_blank_root_token {
                let mut children = vec![token];
                children.extend(tokens);
                let fake_root = HierarchicalToken { token: Token::Blank, children };
                root_tokens.push(fake_root);
            } else {
                tokens.push_front(token);
                root_tokens.extend(tokens);
            }
        }
        None => root_tokens.extend(tokens),
    };

    root_tokens
}

fn split_at_markdown_element<'a>(
    hierachical_tokens: Vec<HierarchicalToken<'a>>,
    token_type: &TokenType,
) -> Vec<Vec<HierarchicalToken<'a>>> {
    let mut groups: Vec<Vec<HierarchicalToken<'a>>> = vec![];
    let mut current_group: Vec<HierarchicalToken<'a>> = vec![];

    for token in hierachical_tokens {
        if &token.token.token_type() == token_type {
            if current_group.is_empty() {
                current_group.push(token);
                continue;
            }

            groups.push(current_group);
            current_group = vec![];
            current_group.push(token);
        } else {
            current_group.push(token);
        }
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HierarchizeStatus {
    max_hierarchy_level: usize,
    hierarchy_level: usize,
    max_recursion_level: usize,
    recursion_level: usize,
}

impl HierarchizeStatus {
    fn new(max_hierarchy_level: usize, max_recursion_level: usize) -> Self {
        Self {
            max_hierarchy_level,
            hierarchy_level: 0,
            max_recursion_level,
            recursion_level: 0,
        }
    }

    fn one_hierarchy_level_deeper(&self) -> Option<Self> {
        if self.hierarchy_level + 1 >= self.max_hierarchy_level {
            return None;
        }
        Some(Self {
            max_hierarchy_level: self.max_hierarchy_level,
            hierarchy_level: self.hierarchy_level + 1,
            max_recursion_level: self.max_recursion_level,
            recursion_level: self.recursion_level,
        })
    }

    fn one_recursion_level_deeper(&self) -> Option<Self> {
        if self.recursion_level + 1 >= self.max_recursion_level {
            return None;
        }
        Some(Self {
            max_hierarchy_level: self.max_hierarchy_level,
            hierarchy_level: self.hierarchy_level,
            max_recursion_level: self.max_recursion_level,
            recursion_level: self.recursion_level + 1,
        })
    }

    fn is_hierarchy_root(&self) -> bool {
        self.hierarchy_level == 0
    }
}


/// Hierarchical Markdown token
#[derive(Clone, Debug, PartialEq, Eq)]
struct HierarchicalToken<'a> {
    token: Token<'a>,
    children: Vec<HierarchicalToken<'a>>,
}

impl<'a> HierarchicalToken<'a> {
    fn from_token(token: Token<'a>) -> Self {
        Self {
            token,
            children: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TokenHierarchy {
    levels: Vec<TokenType>,
}

impl TokenHierarchy {
    fn from_token_types(token_types: Vec<TokenType>) -> Self {
        Self {
            levels: token_types,
        }
    }

    /// Returns the hierarchy position given a MarkdownElementType (the lower the higher up in the hierarchy).
    /// Note: Returns usize::MAX if the MarkdownElementType is not contained in self.hierarchy_levels.
    fn position(&self, token_type: &TokenType) -> usize {
        self.levels
            .iter()
            .position(|r| r == token_type)
            .map_or(usize::MAX, |t| t)
    }

    /// Returns the markdown element type at a given position.
    /// Note: Return None if hierarchy position is out of range.
    fn token_type_at(
        &self,
        hierarchy_position: usize,
    ) -> Option<TokenType> {
        self.levels.get(hierarchy_position).map(|t| t.to_owned())
    }
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use pretty_assertions::assert_eq;

//     #[test]
//     fn test_buildup_hierarchy1() {
//         let mut tokens = vec![
//             heading_token("Titel H1", 1),
//             MarkdownElement::Text("Text H1"),
//             heading_token("Titel H3", 3),
//             MarkdownElement::Text("Text H3"),
//             heading_token("Titel H4", 4),
//             MarkdownElement::Text("Text H4"),
//             heading_token("Titel H3", 3),
//             MarkdownElement::Text("Text H3"),
//             heading_token("Titel H2", 2),
//             MarkdownElement::Text("Text H2"),
//         ];

//         let tokens: Vec<Token> = tokens
//             .drain(0..)
//             .map(|t| Token::from_markdown_element(t))
//             .collect();

//         let hierarchy = MarkdownElementHierarchy::from_markdown_element_types(vec![
//             MarkdownElementType::HeadingH1,
//             MarkdownElementType::HeadingH2,
//             MarkdownElementType::HeadingH3,
//             MarkdownElementType::HeadingH4,
//         ]);

//         assert_eq!(
//             MDPMarkdownHierarchizer {}.hierarchize(hierarchy, tokens).0,
//             vec![Token {
//                 markdown_element: heading_token("Titel H1", 1),
//                 children: vec![
//                     Token::from_markdown_element(MarkdownElement::Text("Text H1")),
//                     Token {
//                         markdown_element: heading_token("Titel H3", 3),
//                         children: vec![
//                             Token::from_markdown_element(MarkdownElement::Text("Text H3")),
//                             Token {
//                                 markdown_element: heading_token("Titel H4", 4),
//                                 children: vec![Token::from_markdown_element(
//                                     MarkdownElement::Text("Text H4")
//                                 ),]
//                             },
//                         ]
//                     },
//                     Token {
//                         markdown_element: heading_token("Titel H3", 3),
//                         children: vec![Token::from_markdown_element(MarkdownElement::Text(
//                             "Text H3"
//                         )),]
//                     },
//                     Token {
//                         markdown_element: heading_token("Titel H2", 2),
//                         children: vec![Token::from_markdown_element(MarkdownElement::Text(
//                             "Text H2"
//                         )),]
//                     },
//                 ]
//             },],
//         );
//     }

//     fn heading_token<'a>(t: &'a str, heading_type: usize) -> MarkdownElement<'a> {
//         match heading_type {
//             1 => MarkdownElement::HeadingH1(vec![MarkdownElement::Text(t)]),
//             2 => MarkdownElement::HeadingH2(vec![MarkdownElement::Text(t)]),
//             3 => MarkdownElement::HeadingH3(vec![MarkdownElement::Text(t)]),
//             4 => MarkdownElement::HeadingH4(vec![MarkdownElement::Text(t)]),
//             _ => panic!("Can only generate H1, H2, H3, and H4 headings."),
//         }
//     }
// }
