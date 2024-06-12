use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO Error `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Parse Int error `{0}`")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Serialize json error: `{0}`")]
    Serialize(#[from] serde_json::Error),
    #[error("Custom Error `{0}`")]
    Custom(String),
}
