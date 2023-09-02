use std::io::BufWriter;

use anyhow::Result;
use ptree::{write_tree, TreeBuilder};

use super::config::TreeConfig;
use crate::{
    commands::io::{FileReader, OutputWriter},
    models::{
        Token, TokenType, MarkdownTokenizer, Section, SectionBuilder,
    },
};

pub fn run<T, S, R>(
    config: TreeConfig,
    tokenizer: T,
    section_builder: S,
    reader: R,
    writers: Vec<Box<dyn OutputWriter>>,
) -> Result<()>
where
    T: MarkdownTokenizer,
    S: SectionBuilder,
    R: FileReader,
{
    let markdown_string = reader.read_file(config.input_path.clone())?;
    let tokens = tokenizer.tokenize(&markdown_string)?;
    let sections = section_builder.sections_from_tokens(tokens)?;

    let output_string = sections_as_ptree_string(&sections, config.debug);
    for writer in writers {
        writer.write_output(&output_string)?;
    }

    Ok(())
}

fn sections_as_ptree_string(sections: &[Section], debug: bool) -> String {
    let mut tb = TreeBuilder::new("".to_string());

    for section in sections {
        add_section_to_tree(section, &mut tb, debug);
    }

    let mut buf = BufWriter::new(Vec::new());
    write_tree(&tb.build(), &mut buf).unwrap();
    let bytes = buf.into_inner().unwrap();

    String::from_utf8(bytes).unwrap()
}

fn add_section_to_tree(section: &Section, tb: &mut TreeBuilder, debug: bool) {
    tb.begin_child(match debug {
        true => section.title.to_debug_string(),
        false => section.title.to_markdown_string(),
    });

    for c in &section.content {
        match c.token_type() {
            TokenType::Newline | TokenType::Blankline => continue,
            _ => {
                if !token_is_empty(c) {
                    tb.add_empty_child(match debug {
                        true => c.to_debug_string(),
                        false => c.to_markdown_string(),
                    });
                };
            }
        };
    }

    for s in &section.subsections {
        if s.subsections.is_empty() && s.content.is_empty() {
            if token_is_empty(&s.title) {
                tb.add_empty_child(match debug {
                    true => s.title.to_debug_string(),
                    false => s.title.to_markdown_string(),
                });
            };
        } else {
            add_section_to_tree(&s, tb, debug);
        }
    }

    tb.end_child();
}

fn token_is_empty(token: &Token) -> bool {
    token.to_markdown_string().trim().is_empty()
}
