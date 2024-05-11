use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TagsConfig {
    pub input_path: PathBuf,
    pub ordering: TagOrderingCriterion,
    pub output_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum TagOrderingCriterion {
    Count,
    Alphabetic,
}
