use clap::ValueEnum;

use mdp::commands::{tags, search, tasks};

#[derive(Clone, Debug, ValueEnum)]
pub enum TagOrderingCriterion {
    Count,
    Alphabetic,
}

impl From<TagOrderingCriterion> for tags::config::TagOrderingCriterion {
    fn from(criterion: TagOrderingCriterion) -> Self {
        match criterion {
            TagOrderingCriterion::Count => Self::Count,
            TagOrderingCriterion::Alphabetic => Self::Alphabetic,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum TagSearchMode {
    And,
    Or,
}

impl From<TagSearchMode> for search::config::TagSearchMode {
    fn from(mode: TagSearchMode) -> Self {
        match mode {
            TagSearchMode::And => Self::And,
            TagSearchMode::Or => Self::Or,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SectionOrderingCriterion {
    Relevance,
    Date,
}

impl From<SectionOrderingCriterion> for search::config::SectionOrderingCriterion {
    fn from(mode: SectionOrderingCriterion) -> Self {
        match mode {
            SectionOrderingCriterion::Relevance => Self::Relevance,
            SectionOrderingCriterion::Date => Self::Date,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum TaskOrderingCriterion {
    // Type,
    Urgency,
    Occurence,
}

impl From<TaskOrderingCriterion> for tasks::config::TaskOrderingCriterion {
    fn from(value: TaskOrderingCriterion) -> Self {
        match value {
            // TaskOrderingCriterion::Type => Self::Type,
            TaskOrderingCriterion::Urgency => Self::Urgency,
            TaskOrderingCriterion::Occurence => Self::Occurence,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum TaskFilterType {
    All,
    Unfinished,
    Finished,
}

impl From<TaskFilterType> for tasks::config::TaskFilterType {
    fn from(mode: TaskFilterType) -> Self {
        match mode {
            TaskFilterType::All => Self::All,
            TaskFilterType::Unfinished => Self::Unfinished,
            TaskFilterType::Finished => Self::Finished,
        }
    }
}
