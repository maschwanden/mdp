pub mod args;
pub mod helpers;

use anyhow::Result;
use clap::Parser;
use simple_logger::SimpleLogger;

use crate::args::{CliArgs, Command};
use mdp::{
    commands::{
        io::{FileWriter, MarkdownFileReader, OutputWriter, StdoutWriter},
        list, search, tree, task,
    },
    markdown::{MDPMarkdownTokenizer, MDPSectionBuilder},
};

fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();
    let cli = CliArgs::parse();

    match &cli.command {
        Command::Search(cmd_args) => {
            let config = search::config::TagSearchConfig::try_from(cmd_args.to_owned())?;
            let output_path = config.output_path.to_owned();
            search::command::run(
                config,
                MDPMarkdownTokenizer {},
                MDPSectionBuilder {},
                MarkdownFileReader {},
                vec![
                    Box::new(StdoutWriter {}),
                    Box::new(FileWriter { path: output_path }),
                ],
            )?
        }

        Command::Tags(cmd_args) => {
            let config = list::config::TagListConfig::try_from(cmd_args.to_owned())?;

            let mut writers: Vec<Box<dyn OutputWriter>> = vec![Box::new(StdoutWriter {})];
            if let Some(output_path) = &config.output_path {
                writers.push(Box::new(FileWriter {
                    path: output_path.to_owned(),
                }));
            }

            list::command::run(
                config,
                MDPMarkdownTokenizer {},
                MarkdownFileReader {},
                writers,
            )?
        }

        Command::TokenTree(cmd_args) => {
            let config = tree::config::TreeConfig::try_from(cmd_args.to_owned())?;
            tree::command::run(
                config,
                MDPMarkdownTokenizer {},
                MDPSectionBuilder {},
                MarkdownFileReader {},
                vec![Box::new(StdoutWriter {})],
            )?
        }

        Command::Tasks(cmd_args) => {
            let config = task::config::TaskConfig::try_from(cmd_args.to_owned())?;
            task::command::run(
                config,
                MDPMarkdownTokenizer {},
                MarkdownFileReader {},
                vec![Box::new(StdoutWriter {})],
            )?
        }
    };

    Ok(())
}
