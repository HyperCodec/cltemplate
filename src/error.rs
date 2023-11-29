use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Exception: {0}")]
    IO(#[from] std::io::Error),

    #[error("Thread Join Exception: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Dialoguer Exception: {0}")]
    Dialoguer(#[from] dialoguer::Error),
}
