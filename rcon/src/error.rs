use std::io::ErrorKind;

use nom::error::Error;
use thiserror::Error;
use tokio::io;

#[derive(PartialEq, Error, Debug)]
pub enum RconError {
    #[error("The data received is invalid and could not be converted to UTF-8.")]
    InvalidData,

    #[error("Error occurred while parsing with nom.")]
    ParsingError(nom::Err<nom::error::Error<String>>),

    #[error("An unhandled kind of io error occured: {0}")]
    IoError(ErrorKind),

    #[error("The server rejected the authentication attempt.")]
    InvalidPassword,
}

impl From<io::Error> for RconError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            e => Self::IoError(e),
        }
    }
}

impl From<nom::Err<Error<&str>>> for RconError {
    fn from(value: nom::Err<Error<&str>>) -> Self {
        let value = value.map_input(|e| e.to_string());
        RconError::ParsingError(value)
    }
}
