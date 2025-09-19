pub struct TagCursor<'a> {
    s: &'a [u8],
    sep: u8,
    position: Option<(usize, usize, usize)>,
}

impl<'a> TagCursor<'a> {
    #[inline]
    pub fn new(s: &'a [u8], sep: u8) -> Self {
        let mut cursor = TagCursor {
            s,
            sep,
            position: None,
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
        // First pass: find '='
        while i < len {
            if bytes[i] == b'=' {
                eq = i;
                i += 1; // begin scanning for the separator after '='
                break;
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
                break;
            }
            i += 1;
        }
        // Record (start, '=', end)
        self.position = Some((start, eq, end));
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
            .map(|(start, eq, _)| parse_u32_ascii(&self.s[start..eq]))
    }
}

#[inline]
pub fn parse_u32_ascii(bytes: &[u8]) -> u32 {
    let mut n: u32 = 0;
    for &b in bytes {
        debug_assert!(b.is_ascii_digit());
        n = n * 10 + (b - b'0') as u32;
    }
    n
}
