use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TreeConfig {
    pub input_path: Vec<PathBuf>,
    pub debug: bool,
}
