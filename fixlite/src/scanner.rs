use memchr::memchr;

pub struct TagCursor<'a> {
    s: &'a [u8],
    sep: u8,
    position: Option<(usize, usize, usize)>,
}

impl<'a> TagCursor<'a> {
    #[inline]
    pub fn new(s: &'a [u8], sep: u8) -> Self {
        let start = 0;
        let position = if let Some(eq) = memchr(b'=', s) {
            let end = memchr(sep, &s[eq + 1..])
                .map(|p| eq + 1 + p)
                .unwrap_or(s.len());
            Some((start, eq, end))
        } else {
            None // EOS
        };
        Self { s, sep, position }
    }

    pub fn skip(&mut self) -> bool {
        self.next().is_some()
    }

    #[inline]
    pub fn peek_tag_u32(&self) -> Option<u32> {
        self.position
            .map(|(start, eq, _)| parse_u32_ascii(&self.s[start..eq]))
    }
}

impl<'a> Iterator for TagCursor<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let s = self.s;
        let sep = self.sep;
        if let Some((_, eq, end)) = self.position {
            let ret = Some(unsafe { std::str::from_utf8_unchecked(&s[eq + 1..end]) });
            let start = end + 1;
            self.position = if start < s.len() {
                if let Some(eq) = memchr(b'=', &s[start..]).map(|p| start + p) {
                    let end = memchr(sep, &s[eq + 1..])
                        .map(|p| eq + 1 + p)
                        .unwrap_or(s.len());
                    Some((start, eq, end))
                } else {
                    None // EOS
                }
            } else {
                None
            };
            ret
        } else {
            None
        }
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
