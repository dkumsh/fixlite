// fix/src/lib.rs

use std::iter::Peekable;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FixError {
    #[error("Invalid FIX message")]
    InvalidMessage,
    #[error("Invalid value for tag {0}")]
    InvalidValue(String),
    #[error("Missing required field {0}")]
    MissingField(&'static str),
    #[error("UTF-8 parsing error")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("DateTime parsing error")]
    DateTimeError(#[from] chrono::ParseError),
}

pub trait FixDeserialize: Sized {
    fn from_fix_message(fix_message: &[u8]) -> Result<Self, FixError>;

    fn from_fix_message_iter<'a, I, F>(
        fields: &mut Peekable<I>,
        is_known_tag: F,
    ) -> Result<Self, FixError>
    where
        I: Iterator<Item = &'a str>,
        F: Fn(&str) -> bool;
    fn is_known_tag(tag: &str) -> bool;
}
