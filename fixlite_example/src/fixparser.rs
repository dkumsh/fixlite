#![allow(dead_code)]

#[derive(Debug)]
pub enum FixParseError {
    InvalidTag,
    MissingValue,
    InvalidFormat,
}

pub struct FixParser<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> FixParser<'a> {
    /// Creates a new FIX parser from raw bytes
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Tries to parse the next field, returning Ok(None) when no more fields are available
    #[inline(always)]
    pub fn try_next(&mut self) -> Result<Option<(u32, &'a [u8])>, FixParseError> {
        // Check if we've reached the end
        if self.position >= self.data.len() {
            return Ok(None);
        }

        let remaining = &self.data[self.position..];

        let mut tag = 0u32;
        let mut equals_pos = 0;

        for (pos, &b) in remaining.iter().enumerate() {
            if b == b'=' {
                equals_pos = pos;
                break;
            }
            let d = b.wrapping_sub(b'0');
            if d > 9 {
                return Err(FixParseError::InvalidTag);
            }
            tag = tag * 10 + d as u32;
        }

        if equals_pos == 0 {
            return Err(FixParseError::InvalidFormat);
        }

        // Find the SOH (0x01) delimiter that ends this field
        let value_start = equals_pos + 1;
        let soh_pos = remaining[value_start..]
            .iter()
            .position(|&b| b == 0x01)
            .ok_or(FixParseError::InvalidFormat)?;

        let value_end = value_start + soh_pos;
        let value = &remaining[value_start..value_end];

        // Update position for next call
        self.position += value_end + 1; // +1 to skip the SOH

        Ok(Some((tag, value)))
    }

    /// Returns the remaining unparsed data
    pub fn remaining(&self) -> &'a [u8] {
        &self.data[self.position..]
    }
}

#[derive(Debug, Default)]
pub struct FixMsg<'a> {
    pub begin_string: Option<&'a [u8]>,  // 8
    pub body_length: Option<u32>,        // 9
    pub msg_type: Option<&'a [u8]>,      // 35
    pub sender_comp: Option<&'a [u8]>,   // 49
    pub target_comp: Option<&'a [u8]>,   // 56
    pub cl_ord_id: Option<&'a [u8]>,     // 11
    pub symbol: Option<&'a [u8]>,        // 55
    pub side: Option<&'a [u8]>,          // 54 (49=Buy, 50=Sell и т.п., здесь оставим как байт)
    pub order_qty: Option<u64>,          // 38
    pub price_fp: Option<&'a [u8]>,      // 44 (fixed-point, масштаб ниже)
    pub transact_time: Option<&'a [u8]>, // 60 или 52 в разных диалектах
    pub checksum: Option<u32>,           // 10 (не проверяем для скорости)
}

// Быстрый парс int без аллокаций; не допускает знака/пробелов.
#[inline(always)]
fn parse_u32(bytes: &[u8]) -> Option<u32> {
    let mut x: u32 = 0;
    if bytes.is_empty() {
        return None;
    }
    for &b in bytes {
        let d = b.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        x = x.checked_mul(10)?.checked_add(d as u32)?;
    }
    Some(x)
}

#[inline(always)]
fn parse_u64(bytes: &[u8]) -> Option<u64> {
    let mut x: u64 = 0;
    if bytes.is_empty() {
        return None;
    }
    for &b in bytes {
        let d = b.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        x = x.checked_mul(10)?.checked_add(d as u64)?;
    }
    Some(x)
}

// Основной парсер: идём по buf, ищем '=' и SOH, матчим теги.
pub fn parse_fix<'a>(buf: &'a [u8]) -> Result<FixMsg<'a>, FixParseError> {
    let mut msg = FixMsg {
        ..Default::default()
    };

    let mut iter = FixParser::new(buf);
    while let Some((tag, val)) = iter.try_next()? {
        match tag {
            8 => {
                msg.begin_string = Some(val);
            }
            9 => {
                msg.body_length = parse_u32(val);
            }
            35 => {
                msg.msg_type = Some(val);
            }
            49 => {
                msg.sender_comp = Some(val);
            }
            56 => {
                msg.target_comp = Some(val);
            }
            11 => {
                msg.cl_ord_id = Some(val);
            }
            55 => {
                msg.symbol = Some(val);
            }
            54 => {
                msg.side = Some(val);
            } // '1','2','B','S' — зависит от диалекта
            38 => {
                msg.order_qty = parse_u64(val);
            }
            44 => {
                msg.price_fp = Some(val);
            }
            52 | 60 => {
                msg.transact_time = Some(val);
            } // ISO8601/UTC
            10 => {
                msg.checksum = parse_u32(val); /* обычно завершающий тег */
            }
            _ => { /* пропускаем ненужные теги в hot path */ }
        }
    }

    Ok(msg)
}
