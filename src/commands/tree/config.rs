use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TreeConfig {
    pub input_path: PathBuf,
    pub debug: bool,
}
