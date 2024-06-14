use std::fs;

use anyhow::Context;
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

fn main() -> Result<(), anyhow::Error> {
    let filename = "foo.txt";
    let _fd = fs::File::open(filename).context(format!("filename: {:?}, not found", filename))?;
    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
