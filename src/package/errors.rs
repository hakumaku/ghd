use std::io;

use thiserror::Error;
use ureq::{serde_json, Error};
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("Github API Error({0}): {1}")]
    GithubAPI(u16, String),
    #[error("Network Error: {0}")]
    Network(String),

    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Zip(#[from] ZipError),
    #[error("Pattern Not Found '{0}'")]
    NoMatchingPattern(String),
}

impl From<Error> for ErrorKind {
    fn from(err: Error) -> Self {
        match err {
            Error::Status(status_code, response) => {
                let message: serde_json::Value = response.into_json().unwrap();
                ErrorKind::GithubAPI(status_code, message["message"].to_string())
            }
            Error::Transport(transport) => ErrorKind::Network(format!("{}", transport)),
        }
    }
}
