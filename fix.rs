// Минимальный, zero-alloc FIX-парсер для hot path.
// Без внешних зависимостей, только std.
// Идея: линейно сканируем buf, ищем '=' и SOH (0x01), парсим число тега,
// затем по match(tag) либо копим срез &value[..], либо конвертируем в числа.
// Валидация упрощена для скорости — предполагаем корректный поток.

// SOH-разделитель FIX
const SOH: u8 = 0x01;

#[derive(Debug, Default)]
pub struct FixMsg<'a> {
    pub begin_string: Option<&'a [u8]>,  // 8
    pub body_length: Option<u32>,        // 9
    pub msg_type: Option<&'a [u8]>,      // 35
    pub sender_comp: Option<&'a [u8]>,   // 49
    pub target_comp: Option<&'a [u8]>,   // 56
    pub cl_ord_id: Option<&'a [u8]>,     // 11
    pub symbol: Option<&'a [u8]>,        // 55
    pub side: Option<u8>,                // 54 (49=Buy, 50=Sell и т.п., здесь оставим как байт)
    pub order_qty: Option<u64>,          // 38
    pub price_fp: Option<i64>,           // 44 (fixed-point, масштаб ниже)
    pub price_scale: i32,                // во сколько знаков после запятой масштабирован price_fp
    pub transact_time: Option<&'a [u8]>, // 60 или 52 в разных диалектах
    pub checksum: Option<u32>,           // 10 (не проверяем для скорости)
}

// Быстрый парс int без аллокаций; не допускает знака/пробелов.
#[inline]
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

#[inline]
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

// Парсинг десятичного числа в фиксированную точку: возвращает mantissa и scale.
// Пример: b"123.45" -> (12345, 2). Никаких аллокаций.
#[inline]
fn parse_decimal_fp(bytes: &[u8]) -> Option<(i64, i32)> {
    if bytes.is_empty() {
        return None;
    }
    let mut neg = false;
    let mut i = 0usize;
    if bytes[0] == b'-' {
        neg = true;
        i = 1;
    }
    let mut mant: i64 = 0;
    let mut scale: i32 = 0;
    let mut seen_dot = false;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'.' {
            if seen_dot {
                return None;
            }
            seen_dot = true;
            i += 1;
            continue;
        }
        let d = b.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        mant = mant.checked_mul(10)?.checked_add(d as i64)?;
        if seen_dot {
            scale += 1;
        }
        i += 1;
    }
    if neg {
        mant = -mant;
    }
    Some((mant, scale))
}

// Основной парсер: идём по buf, ищем '=' и SOH, матчим теги.
pub fn parse_fix<'a>(buf: &'a [u8]) -> Option<FixMsg<'a>> {
    let mut msg = FixMsg {
        price_scale: 0,
        ..Default::default()
    };

    let mut i = 0usize;
    let n = buf.len();

    while i < n {
        // Найти '='
        let mut eq = i;
        while eq < n && buf[eq] != b'=' {
            eq += 1;
        }
        if eq >= n {
            break;
        }

        // Парсим номер тега [i..eq)
        let tag = parse_u32(&buf[i..eq])?;

        // Найти SOH после '='
        let mut j = eq + 1;
        while j < n && buf[j] != SOH {
            j += 1;
        }
        if j > n {
            break;
        } // нет SOH — некорректно

        let val = &buf[eq + 1..j];

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
                if val.len() == 1 {
                    msg.side = Some(val[0]);
                }
            } // '1','2','B','S' — зависит от диалекта
            38 => {
                msg.order_qty = parse_u64(val);
            }
            44 => {
                if let Some((mant, scale)) = parse_decimal_fp(val) {
                    msg.price_fp = Some(mant);
                    msg.price_scale = scale;
                }
            }
            52 | 60 => {
                msg.transact_time = Some(val);
            } // ISO8601/UTC
            10 => {
                msg.checksum = parse_u32(val); /* обычно завершающий тег */
            }
            _ => { /* пропускаем ненужные теги в hot path */ }
        }

        // Следующее поле начинается после SOH
        i = j + 1;
    }

    Some(msg)
}

fn main() {
    // Используем '|' для наглядности; в реальном потоке это 0x01.
    let s = b"8=FIX.4.4|9=112|35=D|49=SENDER|56=TARGET|11=ABC123|55=EUR/USD|54=1|38=1000|44=1.2345|52=20250907-12:34:56.789|10=128|";
    let buf = s
        .iter()
        .map(|&b| if b == b'|' { SOH } else { b })
        .collect::<Vec<u8>>();

    let m = parse_fix(&buf).expect("ok");
    assert_eq!(m.msg_type.unwrap(), b"D");
    assert_eq!(m.cl_ord_id.unwrap(), b"ABC123");
    assert_eq!(m.order_qty, Some(1000));
    assert_eq!(m.price_fp, Some(12345)); // 1.2345 -> mantissa 12345
    assert_eq!(m.price_scale, 4);
    dbg!(m);
}
