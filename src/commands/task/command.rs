use anyhow::Result;
use chrono::{NaiveDate, Utc};

use super::config::{TaskConfig, TaskFilterType, TaskOrderingCriterion};
use crate::{
    commands::io::{FileReader, OutputWriter},
    models::{
        Token, MarkdownTokenizer, TaskStatus,
    },
};

pub fn run<T, R>(
    config: TaskConfig,
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

    let tasks = tasks_from_tokens(tokens);
    let tasks = filter_tasks(tasks, config.filter);
    let tasks = order_tasks(tasks, config.ordering);
    let task_strings = tasks_as_strings(tasks);

    let output_string = task_strings.join("\n");
    for writer in writers {
        writer.write_output(&output_string)?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct Task<'a> {
    content: Vec<Token<'a>>,
    status: TaskStatus,
}

impl<'a> Task<'a> {
    fn is_finished(&self) -> bool {
        match self.status {
            TaskStatus::Done => true,
            _ => false,
        }
    }

    fn is_unfinished(&self) -> bool {
        !self.is_finished()
    }

    fn urgency(&self) -> usize {
        match self.status {
            TaskStatus::Done => 0,
            TaskStatus::Review => 10,
            TaskStatus::Doing => 20,
            TaskStatus::Todo => 30,
            TaskStatus::TodoUntil(d) => {
                let today: NaiveDate = Utc::now().naive_utc().into();
                let days_until = (d - today).num_days();
                let urgency = if days_until > 0 {
                    days_until * 10
                } else {
                    days_until.abs() * 100
                };
                30 + urgency as usize
            },
        }
    }
}

impl<'a> From<&Task<'a>> for Token<'a> {
    fn from(value: &Task<'a>) -> Self {
        Token::Task { content: value.content.clone(), status: value.status.clone() }
    }
}

fn tasks_from_tokens(tokens: Vec<Token>) -> Vec<Task> {
    tokens.iter()
        .filter_map(|t| match t {
        Token::Task { content, status } => Some(Task{content: content.to_owned(), status: status.to_owned()}),
        _ => None,
        })
        .collect()
}

fn filter_tasks(tasks: Vec<Task>, filter: TaskFilterType) -> Vec<Task> {
    match filter {
        TaskFilterType::All => tasks,
        TaskFilterType::Finished => tasks.iter().filter(|t| t.is_finished()).cloned().collect(),
        TaskFilterType::Unfinished => tasks.iter().filter(|t| t.is_unfinished()).cloned().collect(),
    }
}

fn order_tasks(tasks: Vec<Task>, ordering: TaskOrderingCriterion) -> Vec<Task> {
    match ordering {
        TaskOrderingCriterion::Occurence => tasks,
        TaskOrderingCriterion::Urgency => {
            let mut ordered_tasks = tasks.clone();
            ordered_tasks.sort_by_key(|t| t.urgency()); 
            ordered_tasks
        },
    }
}

fn tasks_as_strings(tasks:  Vec<Task>) -> Vec<String> {
    tasks.iter().map(|t| Token::from(t).to_markdown_string()).collect()
}