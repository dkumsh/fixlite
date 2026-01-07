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
