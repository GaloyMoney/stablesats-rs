use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("{0}")]
    PersistError(String),
}
