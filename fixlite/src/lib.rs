pub mod fix;
pub mod scanner;
pub mod type_check;
pub use scanner::TagCursor;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FixError {
    #[error("Invalid FIX message")]
    InvalidMessage,
    #[error("Invalid value for tag {0}")]
    InvalidValue(u32),
    #[error("Invalid tag {0}")]
    InvalidTag(u32),
    #[error("Invalid fix format")]
    InvalidFixFormat,
    #[error("Invalid enum value for tag {0}")]
    InvalidEnumValue(&'static str),
    #[error("Missing required field {0}")]
    MissingField(&'static str),
    #[error("Missing separator")]
    MissingSeparator,
    #[error("UTF-8 parsing error")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("DateTime parsing error")]
    DateTimeError(#[from] chrono::ParseError),
}

pub trait FixDeserialize<'fix>: Sized {
    fn from_fix(fix_message: &'fix [u8]) -> Result<Self, FixError> {
        let mut cur = TagCursor::new(fix_message, b'\x01');
        Self::deserialize_fields(&mut cur, |_| false)
    }

    fn from_fix_with_separator(fix_message: &'fix [u8], separator: u8) -> Result<Self, FixError> {
        let mut cur = TagCursor::new(fix_message, separator);
        Self::deserialize_fields(&mut cur, |_| false)
    }

    fn deserialize_fields<F>(
        cur: &mut TagCursor<'fix>,
        is_a_parent_tag: F,
    ) -> Result<Self, FixError>
    where
        F: Fn(u32) -> bool;

    fn is_known_tag(tag: u32) -> bool;
}
