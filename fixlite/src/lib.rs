pub mod fix;
mod scanner;
pub mod type_check;
pub use scanner::TagCursor;
extern crate self as fixlite;

use thiserror::Error;

/// Wire-format / framing errors (checksum, body length, missing separators, etc.).
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MalformedFix {
    #[error("Invalid FIX message framing")]
    InvalidMessage,
    #[error("Invalid FIX format")]
    InvalidFormat,
    #[error("Missing separator")]
    MissingSeparator,
    #[error("BodyLength mismatch")]
    BodyLengthMismatch,
    #[error("Checksum mismatch")]
    ChecksumMismatch,
}

#[derive(Error, Debug)]
pub enum FixError {
    /// Wire-format/parsing failures (these mean the byte stream is malformed).
    #[error("{0}")]
    Malformed(#[from] MalformedFix),

    /// Semantic/typing failures (message is well-formed but doesn't match expected schema).
    #[error("Invalid value for tag {tag}{ctx}")]
    InvalidValue { tag: u32, ctx: String },

    #[error("Invalid tag {0}")]
    InvalidTag(u32),

    #[error("Invalid enum value for tag {tag} ({ty})")]
    InvalidEnumValue {
        tag: u32,
        /// Enum type name (or tag name) for debugging.
        ty: &'static str,
    },
    #[error("Missing required field {0}")]
    MissingField(&'static str),
    #[error("UTF-8 parsing error")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("DateTime parsing error")]
    DateTimeError(#[from] chrono::ParseError),
}

impl FixError {
    #[inline]
    pub fn invalid_value(tag: u32) -> Self {
        Self::InvalidValue {
            tag,
            ctx: String::new(),
        }
    }

    #[inline]
    pub fn invalid_value_ctx(tag: u32, bytes: &[u8]) -> Self {
        const MAX: usize = 48;
        let mut s = String::from_utf8_lossy(bytes).into_owned();
        if s.len() > MAX {
            s.truncate(MAX);
            s.push('…');
        }
        Self::InvalidValue {
            tag,
            ctx: format!(" (value: {s})"),
        }
    }

    #[inline]
    pub fn invalid_enum_value(tag: u32, ty: &'static str) -> Self {
        Self::InvalidEnumValue { tag, ty }
    }
}
pub trait FixDeserialize<'fix>: Sized {
    fn from_fix(fix_message: &'fix [u8]) -> Result<Self, FixError> {
        let mut cur = TagCursor::new(fix_message, b'\x01');
        let parsed = Self::deserialize_fields(&mut cur, |_| false)?;
        #[cfg(feature = "checksum")]
        cur.validate_checksum()?;
        Ok(parsed)
    }

    fn deserialize_fields<F>(
        cur: &mut TagCursor<'fix>,
        is_a_parent_tag: F,
    ) -> Result<Self, FixError>
    where
        F: Fn(u32) -> bool;

    fn is_known_tag(tag: u32) -> bool;
}

#[cfg(all(test, feature = "checksum"))]
mod checksum_tests {
    use super::{FixDeserialize, FixError};
    use crate::MalformedFix;
    use crate::fix::{FixBuilder, MsgType};
    use chrono::{TimeZone, Utc};

    #[derive(Debug, fixlite_derive::FixDeserialize)]
    struct ChecksumMessage {
        #[fix(tag = 35)]
        msg_type: MsgType,
    }

    fn build_message() -> Vec<u8> {
        let mut builder = FixBuilder::new("FIX.4.2", "S", "T");
        let dt = Utc.with_ymd_and_hms(2025, 1, 2, 3, 4, 5).unwrap();
        let seq = 1u32;

        builder
            .begin_with(&seq, &dt, &MsgType::NewOrderSingle)
            .str(11, "ABC")
            .finish()
            .to_vec()
    }

    fn find_tag_range(msg: &[u8], tag: &[u8]) -> Option<(usize, usize, usize)> {
        let mut idx = 0usize;
        for part in msg.split(|&b| b == b'\x01') {
            let part_len = part.len();
            if part_len == 0 {
                idx += 1;
                continue;
            }
            if part.starts_with(tag) && part.get(tag.len()) == Some(&b'=') {
                let value_start = idx + tag.len() + 1;
                let value_end = idx + part_len;
                return Some((idx, value_start, value_end));
            }
            idx += part_len + 1;
        }
        None
    }

    fn update_checksum(msg: &mut [u8]) {
        let (tag_start, value_start, value_end) =
            find_tag_range(msg, b"10").expect("missing checksum tag");
        debug_assert_eq!(value_end - value_start, 3, "checksum must be 3 digits");

        let sum: u32 = msg[..tag_start].iter().map(|&b| b as u32).sum();
        let checksum = (sum % 256) as u8;

        msg[value_start] = b'0' + (checksum / 100);
        msg[value_start + 1] = b'0' + ((checksum / 10) % 10);
        msg[value_start + 2] = b'0' + (checksum % 10);
    }

    #[test]
    fn checksum_valid_message_passes() {
        let msg = build_message();
        let parsed = ChecksumMessage::from_fix(&msg).unwrap();
        assert_eq!(parsed.msg_type, MsgType::NewOrderSingle);
    }

    #[test]
    fn checksum_mismatch_fails() {
        let mut msg = build_message();
        let (_tag_start, _value_start, value_end) =
            find_tag_range(&msg, b"10").expect("missing checksum tag");
        let last = value_end - 1;
        msg[last] = if msg[last] == b'0' { b'1' } else { b'0' };

        let err = ChecksumMessage::from_fix(&msg).unwrap_err();
        assert!(matches!(
            err,
            FixError::Malformed(MalformedFix::ChecksumMismatch)
                | FixError::Malformed(MalformedFix::InvalidFormat)
        ));
    }

    #[test]
    fn body_length_mismatch_fails() {
        let mut msg = build_message();
        let (_tag_start, _value_start, value_end) =
            find_tag_range(&msg, b"9").expect("missing body length tag");
        let last = value_end - 1;
        msg[last] = if msg[last] == b'0' { b'1' } else { b'0' };

        update_checksum(&mut msg);

        let err = ChecksumMessage::from_fix(&msg).unwrap_err();
        assert!(matches!(
            err,
            FixError::Malformed(MalformedFix::BodyLengthMismatch)
                | FixError::Malformed(MalformedFix::InvalidFormat)
        ));
    }
}
