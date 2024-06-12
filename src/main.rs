use std::fs;

use anyhow::Context;
use ecosystem::MyError;

fn main() -> Result<(), anyhow::Error> {
    let filename = "foo.txt";
    let _fd = fs::File::open(filename).context(format!("filename: {:?}, not found", filename))?;
    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
