pub mod builder;
pub mod enums;
pub mod error;
pub mod fix;
mod scanner;
pub mod tag;
pub mod tags;
pub use crate::builder::{
    AsFixStr, FixBuilder, FixSendingTime, FixSeqNum, FixTaggedValue, FixValue, SOH,
};
pub use crate::error::{FixError, MalformedFix};
#[doc(hidden)]
pub mod __private {
    pub use crate::scanner::TagCursor;
}
extern crate self as fixlite;
#[cfg(feature = "derive")]
pub use fixlite_derive::FixDeserialize;

/// Decode a FIX message into a type that implements `FixDeserialize`.
///
/// Validates that all bytes are ASCII before parsing. FIX 4.x is by spec an
/// ASCII protocol; non-ASCII input returns `MalformedFix::NonAsciiByte`. The
/// validation guarantees the parser can safely treat field values as `&str`
/// throughout. Use [`decode_unchecked`] to skip the check when the caller has
/// already verified the input.
#[inline]
pub fn decode<'fix, T: FixDeserialize<'fix>>(fix_message: &'fix [u8]) -> Result<T, FixError> {
    T::from_fix(fix_message)
}

/// Decode a FIX message without validating that bytes are ASCII.
///
/// # Safety
/// The caller must guarantee that `fix_message` contains only valid UTF-8
/// bytes (ASCII is sufficient and is the FIX 4.x spec). Passing bytes that
/// violate this contract is undefined behavior because the parser uses
/// unchecked UTF-8 conversion internally.
#[inline]
pub unsafe fn decode_unchecked<'fix, T: FixDeserialize<'fix>>(
    fix_message: &'fix [u8],
) -> Result<T, FixError> {
    // SAFETY: forwarded to the caller.
    unsafe { T::from_fix_unchecked(fix_message) }
}

/// Trait for types that can be deserialized from FIX messages.
///
/// Prefer using `#[derive(FixDeserialize)]` (feature `derive`) to implement this.
pub trait FixDeserialize<'fix>: Sized {
    /// Decode a FIX message after validating that all bytes are ASCII.
    ///
    /// Returns `MalformedFix::NonAsciiByte` if any byte is `>= 128`. Use
    /// [`Self::from_fix_unchecked`] to skip this validation.
    fn from_fix(fix_message: &'fix [u8]) -> Result<Self, FixError> {
        if !fix_message.is_ascii() {
            return Err(crate::MalformedFix::NonAsciiByte.into());
        }
        // SAFETY: just validated that every byte is ASCII (< 128), which is a
        // subset of valid UTF-8, so the unchecked UTF-8 conversions performed
        // by the parser are sound.
        unsafe { Self::from_fix_unchecked(fix_message) }
    }

    /// Decode a FIX message without validating UTF-8.
    ///
    /// # Safety
    /// The caller must guarantee that `fix_message` contains only valid UTF-8
    /// bytes (ASCII is sufficient). Violating this contract is undefined
    /// behavior because field values are converted to `&str` without checks.
    unsafe fn from_fix_unchecked(fix_message: &'fix [u8]) -> Result<Self, FixError> {
        let mut cur = crate::__private::TagCursor::new(fix_message, b'\x01');
        let parsed = Self::deserialize_fields(&mut cur, |_| false)?;
        #[cfg(feature = "checksum")]
        cur.validate_checksum()?;
        Ok(parsed)
    }

    /// Parse fields from the tag cursor, deciding which tags belong to this type.
    fn deserialize_fields<F>(
        cur: &mut crate::__private::TagCursor<'fix>,
        is_a_parent_tag: F,
    ) -> Result<Self, FixError>
    where
        F: Fn(u32) -> bool;

    /// Return whether a tag is known to this type.
    fn is_known_tag(tag: u32) -> bool;
}

#[cfg(test)]
mod ascii_validation_tests {
    use super::FixError;
    use crate::FixBuilder;
    use crate::MalformedFix;
    use crate::enums::MsgType;
    use chrono::{TimeZone, Utc};

    #[derive(Debug, fixlite_derive::FixDeserialize)]
    struct SimpleMessage {
        #[fix(tag = 35)]
        msg_type: MsgType,
    }

    fn build_valid_message() -> Vec<u8> {
        let mut builder = FixBuilder::new("FIX.4.2", "S", "T");
        let dt = Utc.with_ymd_and_hms(2025, 1, 2, 3, 4, 5).unwrap();
        let seq = 1u32;
        builder
            .begin_with(&seq, &dt, &MsgType::NewOrderSingle)
            .finish()
            .to_vec()
    }

    #[test]
    fn decode_rejects_non_ascii_bytes() {
        // Take a valid FIX message and corrupt one byte to non-ASCII.
        let mut msg = build_valid_message();
        // Replace a known-ASCII byte mid-message (the 'T' in SenderCompID "T").
        let pos = msg.iter().position(|&b| b == b'T').unwrap();
        msg[pos] = 0xFF;

        let err = crate::decode::<SimpleMessage>(&msg).unwrap_err();
        assert!(
            matches!(err, FixError::Malformed(MalformedFix::NonAsciiByte)),
            "expected NonAsciiByte, got {err:?}"
        );
    }

    #[test]
    fn decode_accepts_pure_ascii() {
        let msg = build_valid_message();
        let parsed: SimpleMessage = crate::decode(&msg).unwrap();
        assert_eq!(parsed.msg_type, MsgType::NewOrderSingle);
    }

    #[test]
    fn decode_unchecked_skips_validation() {
        let msg = build_valid_message();
        // SAFETY: FixBuilder output is pure ASCII.
        let parsed: SimpleMessage = unsafe { crate::decode_unchecked(&msg) }.unwrap();
        assert_eq!(parsed.msg_type, MsgType::NewOrderSingle);
    }

    #[derive(Debug, fixlite_derive::FixDeserialize)]
    struct MessageWithRequiredField<'a> {
        #[fix(tag = 35)]
        #[allow(dead_code)]
        msg_type: MsgType,
        #[fix(tag = 11)]
        #[allow(dead_code)]
        cl_ord_id: &'a str,
    }

    #[test]
    fn missing_field_error_carries_fix_tag() {
        // Build a message that's missing tag 11.
        let msg = build_valid_message();
        let err = crate::decode::<MessageWithRequiredField>(&msg).unwrap_err();
        match err {
            FixError::MissingField { name, tag } => {
                assert_eq!(name, "cl_ord_id");
                assert_eq!(tag, 11);
            }
            other => panic!("expected MissingField {{ tag: 11 }}, got {other:?}"),
        }
    }
}

#[cfg(all(test, feature = "checksum"))]
mod checksum_tests {
    use super::{FixDeserialize, FixError};
    use crate::FixBuilder;
    use crate::MalformedFix;
    use crate::enums::MsgType;
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
        let parsed: ChecksumMessage = fixlite::decode(&msg).unwrap();
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
