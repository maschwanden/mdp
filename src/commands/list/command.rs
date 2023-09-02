use std::{cmp::Ordering, collections::HashMap};

use anyhow::Result;

use super::config::{TagListConfig, TagOrderingCriterion};
use crate::{
    commands::io::{FileReader, OutputWriter},
    models::{MarkdownTokenizer, Token},
};

pub fn run<T, R>(
    config: TagListConfig,
    tokenizer: T,
    reader: R,
    writers: Vec<Box<dyn OutputWriter>>,
) -> Result<()>
where
    T: MarkdownTokenizer,
    R: FileReader,
{
    let markdown_string = reader.read_file(config.input_path.clone())?;
    let tokens = tokenizer.tokenize(&markdown_string)?;
    let count = count_tags(tokens);

    if count.is_empty() {
        log::warn!("No tags found!");
        return Ok(());
    }

    let output_string = count_to_string(&count, &config.ordering);
    for writer in writers {
        writer.write_output(&output_string)?;
    }

    Ok(())
}

fn count_tags(tokens: Vec<Token>) -> HashMap<String, usize> {
    let mut count: HashMap<String, usize> = HashMap::new();
    for token in tokens {
        match &token {
            Token::Tag(s) => *count
                .entry(s.to_string())
                .and_modify(|x| *x += 1)
                .or_insert(1),
            _ => continue,
        };
    }
    count
}

fn count_to_string(count: &HashMap<String, usize>, ordering: &TagOrderingCriterion) -> String {
    let mut counts = count
        .to_owned()
        .into_iter()
        .collect::<Vec<(String, usize)>>();
    match ordering {
        TagOrderingCriterion::Count => counts.sort_by(|a, b| match a.1.cmp(&b.1) {
            Ordering::Equal => a.0.cmp(&b.0),
            other => other,
        }),
        TagOrderingCriterion::Alphabetic => counts.sort_by(|a, b| a.0.cmp(&b.0)),
    }

    let mut s = counts
        .iter()
        .map(|c| format!("{:<20} {:>10}\n", c.0, c.1,))
        .collect::<String>();

    s.insert_str(0, &format!("{:<20} {:>10}\n", "Tag", "Count"));
    s
}
