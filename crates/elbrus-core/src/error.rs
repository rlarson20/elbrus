use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}
