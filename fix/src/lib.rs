pub mod fix;
pub mod type_check;

use std::iter::Peekable;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FixError {
    #[error("Invalid FIX message")]
    InvalidMessage,
    #[error("Invalid value for tag {0}")]
    InvalidValue(&'static str),
    #[error("Missing required field {0}")]
    MissingField(&'static str),
    #[error("UTF-8 parsing error")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("DateTime parsing error")]
    DateTimeError(#[from] chrono::ParseError),
}

pub trait FixDeserialize<'fix>: Sized {
    fn from_fix_message(
        fix_message: &'fix [u8],
        delimiter: Option<char>,
    ) -> Result<Self, FixError> {
        let delimiter = delimiter.unwrap_or('\x01');
        let fix_message_str = std::str::from_utf8(fix_message)?;
        let mut fields = fix_message_str.split(delimiter).peekable();
        Self::from_fix_message_inner(&mut fields, |_| false)
    }
    fn from_fix_message_inner<I, F>(
        fields: &mut Peekable<I>,
        is_a_parent_tag: F,
    ) -> Result<Self, FixError>
        where
            I: Iterator<Item = &'fix str>,
            F: Fn(&str) -> bool;
    fn is_known_tag(tag: &str) -> bool;
}
