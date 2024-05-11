use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TasksConfig {
    pub input_path: Vec<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub ordering: TaskOrderingCriterion,
    pub filter: TaskFilterType,
}

#[derive(Clone, Debug)]
pub enum TaskOrderingCriterion {
    Urgency,
    Occurence,
}

#[derive(Clone, Debug)]
pub enum TaskFilterType {
    All,
    Unfinished,
    Finished,
}
