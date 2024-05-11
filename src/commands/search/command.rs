use std::cmp::Ordering;

use chrono::NaiveDate;

use anyhow::Result;

use super::config::{SearchTerm, SectionOrderingCriterion, SearchConfig, TagSearchMode};
use crate::{
    commands::io::{FileReader, OutputWriter},
    models::{MarkdownTokenizer, Section, SectionBuilder, SectionType},
};

pub fn run<T, S, R>(
    config: SearchConfig,
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

    let results = search(
        sections,
        config.search_terms,
        config.search_mode,
        config.from,
        config.until,
    );

    let output_string = search_results_to_string(results, config.ordering);
    for writer in writers {
        writer.write_output(&output_string)?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct SearchResultSection<'a> {
    pub matched_tags: Vec<String>,
    pub section: Section<'a>,
}

fn search(
    sections: Vec<Section>,
    search_terms: Vec<SearchTerm>,
    mode: TagSearchMode,
    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
) -> Vec<SearchResultSection> {
    let mut results = vec![];
    for s in sections {
        if tags_match(&s.tags, &search_terms, &mode) && in_date_range(s.date, from, until) {
            results.push(SearchResultSection {
                section: s.clone(),
                matched_tags: matched_tags(&s.tags, &search_terms),
            });
        }
        results.append(&mut search(
            s.subsections,
            search_terms.clone(),
            mode.clone(),
            from,
            until,
        ))
    }
    results
}

fn tags_match(tags: &[String], tag_search_terms: &[SearchTerm], mode: &TagSearchMode) -> bool {
    match mode {
        TagSearchMode::Or => tag_search_terms.iter().any(|t| tags.contains(&t.inner())),
        TagSearchMode::And => tag_search_terms.iter().all(|t| tags.contains(&t.inner())),
    }
}

fn matched_tags(tags: &[String], tag_search_terms: &[SearchTerm]) -> Vec<String> {
    tag_search_terms
        .iter()
        .filter_map(|t| {
            if tags.contains(&t.inner()) {
                Some(t.inner())
            } else {
                None
            }
        })
        .collect()
}

fn in_date_range(date: NaiveDate, from: Option<NaiveDate>, until: Option<NaiveDate>) -> bool {
    if let Some(from) = from {
        if date < from {
            return false;
        }
    }
    if let Some(until) = until {
        if date > until {
            return false;
        }
    }
    true
}

fn search_results_to_string(
    results: Vec<SearchResultSection>,
    ordering: SectionOrderingCriterion,
) -> String {
    let ordered_results = ordered_search_result_sections(results, ordering);

    let mut section_strings = Vec::<String>::new();
    let mut previous_section_date: Option<NaiveDate> = None;

    for r in ordered_results.iter() {
        let mut s = String::new();

        if r.section.section_type != SectionType::H1 {
            if previous_section_date.is_none() || previous_section_date.unwrap() != r.section.date {
                s += &format!("# {}\n\n", r.section.date);
            } else {
                s += &format!("{}\n\n", section_strings.pop().unwrap().to_owned());
            }
        }
        s += r.section.to_string().trim();
        section_strings.push(s);

        previous_section_date = Some(r.section.date);
    }

    section_strings.join("\n\n---\n\n")
}

fn ordered_search_result_sections(
    results: Vec<SearchResultSection>,
    ordering: SectionOrderingCriterion,
) -> Vec<SearchResultSection> {
    let mut ordered_result = results.clone();
    match ordering {
        SectionOrderingCriterion::Relevance => ordered_result.sort_by(|a, b| {
            match a.matched_tags.len().cmp(&b.matched_tags.len()).reverse() {
                Ordering::Equal => a.section.date.cmp(&b.section.date),
                other => other,
            }
        }),
        SectionOrderingCriterion::Date => {
            ordered_result.sort_by(|a, b| match a.section.date.cmp(&b.section.date) {
                Ordering::Equal => a.matched_tags.len().cmp(&b.matched_tags.len()).reverse(),
                other => other,
            })
        }
    }
    ordered_result
}
