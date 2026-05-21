#[cfg(feature = "checksum")]
use crate::{FixError, MalformedFix};

pub struct TagCursor<'a> {
    s: &'a [u8],
    sep: u8,
    position: Option<(usize, usize, usize)>,
    #[cfg(feature = "checksum")]
    checksum_sum: u32,
    #[cfg(feature = "checksum")]
    body_start: Option<usize>,
    #[cfg(feature = "checksum")]
    body_length: Option<u32>,
    #[cfg(feature = "checksum")]
    checksum_tag_start: Option<usize>,
    #[cfg(feature = "checksum")]
    checksum_expected: Option<u32>,
    #[cfg(feature = "checksum")]
    checksum_done: bool,
}

impl<'a> TagCursor<'a> {
    #[inline]
    pub fn new(s: &'a [u8], sep: u8) -> Self {
        let mut cursor = TagCursor {
            s,
            sep,
            position: None,
            #[cfg(feature = "checksum")]
            checksum_sum: 0,
            #[cfg(feature = "checksum")]
            body_start: None,
            #[cfg(feature = "checksum")]
            body_length: None,
            #[cfg(feature = "checksum")]
            checksum_tag_start: None,
            #[cfg(feature = "checksum")]
            checksum_expected: None,
            #[cfg(feature = "checksum")]
            checksum_done: false,
        };
        cursor.advance(0);
        cursor
    }

    /// Scans from `start` to locate '=' and, after that, the separator.
    /// Uses sentinel values instead of Option to minimize branching.
    #[inline]
    fn advance(&mut self, start: usize) {
        let bytes = self.s;
        let len = bytes.len();
        if start >= len {
            self.position = None;
            return;
        }

        let mut eq = len; // sentinel if '=' isn't found
        let mut i = start;
        #[cfg(feature = "checksum")]
        let mut field_sum: u32 = 0;

        // First pass: find '='
        while i < len {
            if bytes[i] == b'=' {
                eq = i;
                i += 1; // begin scanning for the separator after '='
                #[cfg(feature = "checksum")]
                {
                    field_sum = field_sum.wrapping_add(b'=' as u32);
                }
                break;
            }
            #[cfg(feature = "checksum")]
            {
                field_sum = field_sum.wrapping_add(bytes[i] as u32);
            }
            i += 1;
        }
        if eq == len {
            self.position = None;
            return;
        }

        // Second pass: find separator after '='
        let mut end = len;
        while i < len {
            if bytes[i] == self.sep {
                end = i;
                #[cfg(feature = "checksum")]
                {
                    field_sum = field_sum.wrapping_add(self.sep as u32);
                }
                break;
            }
            #[cfg(feature = "checksum")]
            {
                field_sum = field_sum.wrapping_add(bytes[i] as u32);
            }
            i += 1;
        }
        // Record (start, '=', end)
        self.position = Some((start, eq, end));

        #[cfg(feature = "checksum")]
        self.update_checksum(start, eq, end, field_sum);
    }

    #[inline]
    pub fn skip(&mut self) {
        if let Some((_, _, end)) = self.position {
            self.advance(end + 1);
        }
    }

    #[inline]
    pub fn next_value(&mut self) -> Option<&'a str> {
        if let Some((_, eq, end)) = self.position {
            let value = unsafe { std::str::from_utf8_unchecked(&self.s[eq + 1..end]) };
            self.advance(end + 1);
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn peek_tag_u32(&self) -> Option<u32> {
        self.position
            .and_then(|(start, eq, _)| parse_u32_ascii(&self.s[start..eq]))
    }

    #[cfg(feature = "checksum")]
    fn update_checksum(&mut self, start: usize, eq: usize, end: usize, field_sum: u32) {
        if self.checksum_done {
            return;
        }
        let Some(tag) = parse_u32_ascii(&self.s[start..eq]) else {
            // Leave body_length / checksum_expected unset; validate_checksum
            // surfaces this as MalformedFix::InvalidFormat.
            return;
        };
        if tag == 10 {
            self.checksum_tag_start = Some(start);
            self.checksum_expected = parse_u32_ascii(&self.s[eq + 1..end]);
            self.checksum_done = true;
            return;
        }
        if tag == 9 {
            self.body_length = parse_u32_ascii(&self.s[eq + 1..end]);
            self.body_start = Some(end + 1);
        }
        self.checksum_sum = self.checksum_sum.wrapping_add(field_sum);
    }

    #[cfg(feature = "checksum")]
    pub fn validate_checksum(&self) -> Result<(), FixError> {
        let body_length = self.body_length.ok_or(MalformedFix::InvalidFormat)?; // missing 9 in a FIX stream

        let body_start = self.body_start.ok_or(MalformedFix::InvalidFormat)?;

        let checksum_tag_start = self.checksum_tag_start.ok_or(MalformedFix::InvalidFormat)?; // missing 10

        let expected = self.checksum_expected.ok_or(MalformedFix::InvalidFormat)?;

        let actual_len = checksum_tag_start
            .checked_sub(body_start)
            .ok_or(MalformedFix::InvalidFormat)?;

        if actual_len != body_length as usize {
            return Err(MalformedFix::BodyLengthMismatch.into());
        }

        let actual_checksum = self.checksum_sum % 256;
        if expected != actual_checksum {
            return Err(MalformedFix::ChecksumMismatch.into());
        }

        Ok(())
    }
}

/// Parse an ASCII-digit byte slice into `u32`.
///
/// Returns `None` for empty input, any non-digit byte, or arithmetic overflow.
#[inline]
pub fn parse_u32_ascii(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    let mut n: u32 = 0;
    for &b in bytes {
        let d = b.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        n = n.checked_mul(10)?.checked_add(d as u32)?;
    }
    Some(n)
}

#[cfg(test)]
mod tests {
    use super::parse_u32_ascii;

    #[test]
    fn empty_input_returns_none() {
        assert_eq!(parse_u32_ascii(b""), None);
    }

    #[test]
    fn non_digit_byte_returns_none() {
        assert_eq!(parse_u32_ascii(b"12a"), None);
        assert_eq!(parse_u32_ascii(b"a12"), None);
        assert_eq!(parse_u32_ascii(b"1 2"), None);
        assert_eq!(parse_u32_ascii(b"-1"), None);
    }

    #[test]
    fn overflow_returns_none() {
        // u32::MAX = 4294967295 (10 digits).
        assert_eq!(parse_u32_ascii(b"4294967296"), None);
        assert_eq!(parse_u32_ascii(b"99999999999"), None);
    }

    #[test]
    fn valid_input_parses() {
        assert_eq!(parse_u32_ascii(b"0"), Some(0));
        assert_eq!(parse_u32_ascii(b"123"), Some(123));
        assert_eq!(parse_u32_ascii(b"4294967295"), Some(u32::MAX));
    }
}
