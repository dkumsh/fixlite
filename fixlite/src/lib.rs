pub mod fix;
pub mod type_check;

use std::iter::Peekable;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FixError {
    #[error("Invalid FIX message")]
    InvalidMessage,
    #[error("Invalid value")]
    InvalidValue(u32),
    #[error("Missing required field {0}")]
    MissingField(&'static str),
    #[error("UTF-8 parsing error")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("DateTime parsing error")]
    DateTimeError(#[from] chrono::ParseError),
}

pub trait FixDeserialize<'fix>: Sized {
    fn from_fix(fix_message: &'fix [u8]) -> Result<Self, FixError> {
        let fix_message_str = std::str::from_utf8(fix_message)?;
        let mut fields = fix_message_str.split('\x01').peekable();
        Self::deserialize_fields(&mut fields, |_| false)
    }

    fn from_fix_with_separator(fix_message: &'fix [u8], separator: char) -> Result<Self, FixError> {
        let fix_message_str = std::str::from_utf8(fix_message)?;
        let mut fields = fix_message_str.split(separator).peekable();
        Self::deserialize_fields(&mut fields, |_| false)
    }

    fn deserialize_fields<I, F>(
        fields: &mut Peekable<I>,
        is_a_parent_tag: F,
    ) -> Result<Self, FixError>
    where
        I: Iterator<Item = &'fix str>,
        F: Fn(u32) -> bool;

    fn is_known_tag(tag: u32) -> bool;
}
