use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TaskConfig {
    pub input_path: PathBuf,
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
