use serde_yaml;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Errors {
    #[error("Cannot cook concourse configuration:\n{0}")]
    CookError(String),
    #[error("Cannot cook concourse configuration due to serde error:\n{0}")]
    SerdeError(serde_yaml::Error),
}

impl Errors {
    pub fn from(detail: &str) -> Errors {
        Errors::CookError(String::from(detail))
    }
}
