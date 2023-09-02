use std::{fs, path::PathBuf};

use crate::models::MDPError;

pub trait FileReader {
    fn read_file(&self, path: PathBuf) -> Result<String, MDPError>;
}

pub struct MarkdownFileReader {}

impl FileReader for MarkdownFileReader {
    fn read_file(&self, path: PathBuf) -> Result<String, MDPError> {
        fs::read_to_string(path.as_path()).map_err(|_| MDPError::IOReadError(path))
    }
}

pub trait OutputWriter {
    fn write_output(&self, output: &str) -> Result<(), MDPError>;
}

pub struct StdoutWriter {}

impl OutputWriter for StdoutWriter {
    fn write_output(&self, output: &str) -> Result<(), MDPError> {
        println!("{}", output);
        Ok(())
    }
}

pub struct FileWriter {
    pub path: PathBuf,
}

impl OutputWriter for FileWriter {
    fn write_output(&self, output: &str) -> Result<(), MDPError> {
        fs::write(self.path.clone(), output)
            .map_err(|_| MDPError::IOReadError(self.path.clone()))?;
        Ok(())
    }
}
