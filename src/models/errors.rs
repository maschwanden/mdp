use std::{error::Error, fmt, fmt::Display, path::PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum MDPError {
    MarkdownParseError { msg: String, line_number: usize },
    MDPSyntaxError(String),
    IOReadError {
        path: PathBuf,
        details: String,
    },
    IOWriteError(PathBuf),
    IOError(String),
    ConfigError(ConfigError),

    MultiError(Vec<MDPError>),
}

impl Display for MDPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::MarkdownParseError { msg, line_number } => format!(
                "The following error occured during tokenization on line {}: {}",
                line_number,
                msg.to_owned()
            ),
            Self::MDPSyntaxError(s) => s.to_owned(),
            Self::IOReadError{ path, details } => match path.to_str() {
                Some(f) => format!("An error occured while reading the file {}: {}", f, details),
                None => format!("An error occured while reading a file: {}", details),
            },
            Self::IOWriteError(f) => match f.to_str() {
                Some(ff) => format!("An error occured while writing the following file: {}", ff),
                None => "An error occured while writing a file".to_string(),
            },
            Self::IOError(s) => s.to_string(),
            Self::ConfigError(e) => e.to_string(),
            Self::MultiError(errors) => format!(
                "Multiple errors occured:\n{}",
                errors
                    .iter()
                    .map(|e| format!("- {}", e))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        };
        write!(f, "{}", msg)
    }
}

impl Error for MDPError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    IOError,
    InvalidSearchTermError,
    IncompatibleConfigError,
    UnkownError,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::IOError => "An IO error occured while processing the configuration",
            Self::InvalidSearchTermError => "One of the provided search terms is invalid",
            Self::IncompatibleConfigError => {
                "The provided configuration is incompatible with the command"
            }
            Self::UnkownError => "An unknown error occured",
        };
        write!(f, "{}", msg)
    }
}

impl Error for ConfigError {}
