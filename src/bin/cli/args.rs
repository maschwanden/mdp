use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};

use super::helpers::*;
use mdp::{
    commands::{
        list::config::TagListConfig,
        search::config::{SearchTerm, TagSearchConfig},
        tree::config::TreeConfig, task::config::TaskConfig,
    },
    models::ConfigError,
};

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Search(TagSearchCommandArgs),
    Tags(TagsCommandArgs),
    TokenTree(TokenTreeCommandArgs),
    Tasks(TasksCommandArgs),
}

/// List tags
#[derive(Args, Debug, Clone)]
pub struct TagsCommandArgs {
    /// The path to the markdown file
    #[arg()]
    pub input_path: PathBuf,

    /// Export list to file
    #[arg(short = 'o', long = "output", default_value = None)]
    pub output_path: Option<PathBuf>,

    /// Ordering of tags
    #[arg(
        long = "ordering",
        value_enum,
        rename_all = "UPPER",
        default_value = "alphabetic"
    )]
    pub ordering: TagOrderingCriterion,
}

impl TryFrom<TagsCommandArgs> for TagListConfig {
    type Error = ConfigError;

    fn try_from(args: TagsCommandArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            input_path: args.input_path,
            ordering: args.ordering.into(),
            output_path: args.output_path,
        })
    }
}

/// Search for tags
#[derive(Args, Debug, Clone)]
pub struct TagSearchCommandArgs {
    /// The path to the markdown file
    #[arg()]
    pub input_path: PathBuf,

    /// Export list to file
    #[arg(short = 'o', long = "output", default_value = "./search.md")]
    pub output_path: PathBuf,

    /// The tag(s) to look for (comma-separated)
    #[arg(name = "TERM")]
    pub search_string: String,

    /// Defines how multiple search terms are logically combined
    #[arg(long = "mode", rename_all = "UPPER", default_value = "or")]
    pub search_mode: TagSearchMode,

    /// Defines the ordering of search results
    #[arg(
        long = "order",
        value_enum,
        rename_all = "UPPER",
        default_value = "date"
    )]
    pub ordering: SectionOrderingCriterion,

    /// Write matched sections also to stdout
    #[clap(long = "stdout", global = true)]
    pub stdout: bool,

    /// Only consider sections after this date
    #[clap(long = "from")]
    pub from: Option<NaiveDate>,

    /// Only consider sections before this date
    #[clap(long = "until")]
    pub until: Option<NaiveDate>,
}

impl TryFrom<TagSearchCommandArgs> for TagSearchConfig {
    type Error = ConfigError;

    fn try_from(args: TagSearchCommandArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            input_path: args.input_path,
            output_path: args.output_path,
            ordering: args.ordering.into(),
            search_terms: args
                .search_string
                .split(',')
                .collect::<Vec<&str>>()
                .iter()
                .map(|s| {
                    s.trim()
                        .to_string()
                        .try_into()
                        .map_err(|_| ConfigError::InvalidSearchTermError)
                })
                .collect::<Result<Vec<SearchTerm>, Self::Error>>()?,
            search_mode: args.search_mode.into(),
            from: args.from,
            until: args.until,
        })
    }
}

/// Show tree of Markdown content/tokens
#[derive(Args, Debug, Clone)]
pub struct TokenTreeCommandArgs {
    /// The path to the markdown file
    #[arg()]
    pub input_path: PathBuf,

    /// Activate debug mode: Print everything using debug representation
    #[clap(long = "debug", global = false)]
    pub debug: bool,
}

impl TryFrom<TokenTreeCommandArgs> for TreeConfig {
    type Error = ConfigError;

    fn try_from(args: TokenTreeCommandArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            input_path: args.input_path,
            debug: args.debug,
        })
    }
}


/// Show all tasks (TODO, TODO UNTIL <DATE>, DOING, REVIEW, DONE)
#[derive(Args, Debug, Clone)]
pub struct TasksCommandArgs {
    /// The path to the markdown file
    #[arg()]
    pub input_path: PathBuf,

    /// Only show tasks of the chosen kind
    #[arg(long = "show", rename_all = "UPPER", default_value = "unfinished")]
    pub filter: TaskFilterType,

    /// Order tasks according to the provided order criterion
    #[arg(
        long = "order",
        value_enum,
        rename_all = "UPPER",
        default_value = "occurence"
    )]
    pub ordering: TaskOrderingCriterion,
}

impl TryFrom<TasksCommandArgs> for TaskConfig {
    type Error = ConfigError;

    fn try_from(args: TasksCommandArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            input_path: args.input_path,
            ordering: args.ordering.into(),
            filter: args.filter.into(),
        })
    }
}
