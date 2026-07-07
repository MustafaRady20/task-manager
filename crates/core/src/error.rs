#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("task {0} not found")]
    TaskNotFound(i64),
}
