use crate::fix::{DayOfMonth, FixedPrice};
use chrono::{DateTime, Datelike, Timelike, Utc};

/// FIX field delimiter (SOH / 0x01).
pub const SOH: u8 = 0x01;
/// Maximum decimal digits required to encode a `usize` body length.
/// `log10(usize::MAX) + 1 = 20` on 64-bit; an over-estimate on 32-bit is harmless.
const MAX_BODY_LEN_DIGITS: usize = 20;

/// Build a FIX message with a single macro invocation.
///
/// This expands to `begin_with(...).field(...).finish()` using `FixBuilder`.
/// Supports `tag => value` pairs, tagged values (`@value`), and legacy `tag, value` pairs.
#[macro_export]
macro_rules! build_fix {
    ($builder:expr, $seq_out:expr, $dt:expr, $msg_type:expr $(,)?) => {{
        $builder
            .begin_with(&$seq_out, &$dt, &$msg_type)
            .finish()
    }};
    ($builder:expr, $seq_out:expr, $dt:expr, $msg_type:expr $(, $($rest:tt)+)?) => {{
        let msg = $builder.begin_with(&$seq_out, &$dt, &$msg_type);
        $crate::build_fix!(@fields msg $(, $($rest)+)?).finish()
    }};
    (@fields $msg:expr) => { $msg };
    (@fields $msg:expr, ) => { $msg };
    (@fields $msg:expr, $tag:expr => $val:expr ,) => {
        $msg.field_ref($tag as u32, &$val)
    };
    (@fields $msg:expr, @$val:expr ,) => {
        $msg.field_tagged_ref(&$val)
    };
    (@fields $msg:expr, $tag:expr, $val:expr ,) => {
        $msg.field_ref($tag as u32, &$val)
    };
    (@fields $msg:expr, $tag:expr => $val:expr $(, $($rest:tt)+)?) => {
        $crate::build_fix!(@fields $msg.field_ref($tag as u32, &$val) $(, $($rest)+)?)
    };
    (@fields $msg:expr, @$val:expr $(, $($rest:tt)+)?) => {
        $crate::build_fix!(@fields $msg.field_tagged_ref(&$val) $(, $($rest)+)?)
    };
    (@fields $msg:expr, $tag:expr, $val:expr $(, $($rest:tt)+)?) => {
        $crate::build_fix!(@fields $msg.field_ref($tag as u32, &$val) $(, $($rest)+)?)
    };
}

/// Values that can be encoded as a FIX field value (no heap allocation required).
pub trait FixValue {
    /// Encode the value into `out` without allocating.
    fn encode(&self, out: &mut Vec<u8>);
}

/// Values that map to a single, fixed FIX tag.
pub trait FixTaggedValue: FixValue {
    /// The FIX tag associated with this type.
    const TAG: u32;
}

/// Fallible FIX value encoding for builders that need validation.
///
/// Note: in this crate, only `f64` is fallible (NaN/inf); other impls are infallible.
pub trait TryFixValue {
    /// Error returned when encoding fails.
    type Error;
    /// Encode the value into `out`, returning an error if the value is invalid.
    fn try_encode(&self, out: &mut Vec<u8>) -> Result<(), Self::Error>;
}

impl TryFixValue for f64 {
    type Error = FixError;

    #[inline]
    fn try_encode(&self, out: &mut Vec<u8>) -> Result<(), Self::Error> {
        if encode_f64_checked(*self, out) {
            Ok(())
        } else {
            // tag=0 placeholder; try_kv() patches it to the actual tag
            Err(FixError::invalid_value(0))
        }
    }
}

/// Marker trait: values suitable for FIX tag 34 (MsgSeqNum).
pub trait FixSeqNum: FixValue {}

// --- Common FixSeqNum impls ---
impl FixSeqNum for u32 {}
impl FixSeqNum for u64 {}
impl FixSeqNum for usize {}
impl FixSeqNum for i64 {}

/// Marker trait: values suitable for FIX tag 52 (SendingTime, UTCTimestamp format).
pub trait FixSendingTime {
    /// Encode the sending time into `out` (YYYYMMDD-HH:MM:SS.mmm).
    fn encode_sending_time(&self, out: &mut Vec<u8>);
}

// --- Common FixSendingTime impls ---
impl FixSendingTime for DateTime<Utc> {
    #[inline]
    fn encode_sending_time(&self, out: &mut Vec<u8>) {
        encode_timestamp_utc(self, out);
    }
}

/// FixSendingTime wrapper for a preformatted str.
pub struct SendingTimeStr<'a>(pub &'a str);

impl FixSendingTime for SendingTimeStr<'_> {
    #[inline]
    fn encode_sending_time(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.0.as_bytes());
    }
}

/// FixSendingTime wrapper for preformatted bytes.
pub struct SendingTimeBytes<'a>(pub &'a [u8]);

impl FixSendingTime for SendingTimeBytes<'_> {
    #[inline]
    fn encode_sending_time(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.0);
    }
}

/// Convenience trait for enums that map to a static FIX code ("D", "1", "2", ...).
pub trait AsFixStr {
    /// Return the static FIX code for this value.
    fn as_fix_str(&self) -> &'static str;
}

impl<T: AsFixStr> FixValue for T {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.as_fix_str().as_bytes());
    }
}

// --- Common FixValue impls ---

impl FixValue for str {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.as_bytes());
    }
}

impl FixValue for String {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.as_bytes());
    }
}

impl FixValue for [u8] {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }
}

impl FixValue for bool {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        out.push(if *self { b'Y' } else { b'N' });
    }
}

impl FixValue for u8 {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        push_u64_ascii(out, *self as u64);
    }
}

impl FixValue for u32 {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        push_u64_ascii(out, *self as u64);
    }
}
impl FixValue for u64 {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        push_u64_ascii(out, *self);
    }
}
impl FixValue for usize {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        push_u64_ascii(out, *self as u64);
    }
}
impl FixValue for i64 {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        push_i64_ascii(out, *self);
    }
}
impl FixValue for i32 {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        (*self as i64).encode(out)
    }
}

const DIGITS_U16: [u16; 100] = digits_00_99_u16();
impl FixValue for f64 {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        // infallible path: ("write nothing" == skip)
        let _ = encode_f64_checked(*self, out);
    }
}

#[inline]
fn encode_f64_checked(mut value: f64, out: &mut Vec<u8>) -> bool {
    let start = out.len();
    if !value.is_finite() {
        return false;
    }
    if value < 0.0 {
        out.push(b'-');
        value = -value;
    }
    let whole = value as u64;
    let len = out.len();
    push_u64_ascii(out, whole);
    let wd = if whole != 0 {
        (out.len() - len) as u32
    } else {
        0
    };

    if wd < 15 {
        let mut factor = 10u64.pow(15 - wd);
        let fraction = value - whole as f64;

        // IEEE-754: round to nearest, ties to even (banker's rounding)
        let scaled = fraction * (factor as f64);
        let mut fraction = scaled.round();
        if fraction <= 0.0 {
            fraction = 0.0;
        }
        let mut fraction = fraction as u64;

        if fraction == factor {
            if let Some(next) = whole.checked_add(1) {
                out.truncate(len); // rewind to before writing whole
                push_u64_ascii(out, next);
            }
            // else: whole == u64::MAX, cannot carry; keep already-written whole
            return true;
        }

        if fraction > 0 {
            out.push(b'.');
            while fraction > 0 {
                let n: usize; // 0..100 (index into DIGITS_U16)

                if factor >= 100 {
                    factor /= 100;
                    n = (fraction / factor) as usize;
                    fraction %= factor;
                } else if factor == 10 {
                    n = (fraction as usize) * 10;
                    fraction = 0;
                } else {
                    n = fraction as usize;
                    fraction = 0;
                }
                let pair = DIGITS_U16[n];
                let [tens, ones] = pair.to_ne_bytes();
                if fraction > 0 || ones != b'0' {
                    push_2_u16(out, pair);
                } else {
                    out.push(tens);
                }
            }
        }
    }

    // written if we appended anything at all
    out.len() != start
}

/// FIX timestamp format: YYYYMMDD-HH:MM:SS.mmm
impl FixValue for DateTime<Utc> {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        encode_timestamp_utc(self, out);
    }
}

impl FixValue for DayOfMonth {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        self.0.encode(out);
    }
}

impl<const W: u32, const F: u32> FixValue for FixedPrice<W, F> {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        let mut raw = self.raw();
        if raw < 0 {
            out.push(b'-');
            raw = raw.wrapping_neg();
        }

        let scale_digits = F as usize;
        if scale_digits == 0 {
            push_u64_ascii(out, raw as u64);
            return;
        }
        if scale_digits > 18 {
            out.extend_from_slice(self.to_string().as_bytes());
            return;
        }

        let scale = 10u64.pow(F);
        let abs = raw as u64;
        let int_part = abs / scale;
        let frac_part = abs % scale;

        push_u64_ascii(out, int_part);
        if frac_part == 0 {
            return;
        }

        out.push(b'.');
        let mut frac_buf = [b'0'; 18];
        let mut x = frac_part;
        for i in (0..scale_digits).rev() {
            frac_buf[i] = b'0' + (x % 10) as u8;
            x /= 10;
        }
        let mut end = scale_digits;
        while end > 0 && frac_buf[end - 1] == b'0' {
            end -= 1;
        }
        out.extend_from_slice(&frac_buf[..end]);
    }
}

// --- small helpers (no fmt, no allocation) ---

#[inline]
fn push_2_u16(out: &mut Vec<u8>, pair: u16) {
    out.reserve(2);
    let start = out.len();

    // Get 2 bytes of spare capacity
    let spare = out.spare_capacity_mut();
    let dst = &mut spare[..2];

    let bytes = pair.to_ne_bytes();
    dst[0].write(bytes[0]);
    dst[1].write(bytes[1]);

    unsafe { out.set_len(start + 2) };
}

#[inline]
fn push_2(out: &mut Vec<u8>, n: u8) {
    push_2_u16(out, DIGITS_U16[n as usize]);
}

#[inline]
fn push_3(out: &mut Vec<u8>, n: u16) {
    out.push(b'0' + (n / 100) as u8);
    push_2_u16(out, DIGITS_U16[(n % 100) as usize]);
}

#[inline]
fn push_4(out: &mut Vec<u8>, n: u32) {
    debug_assert!(n < 10_000);
    push_2_u16(out, DIGITS_U16[(n / 100) as usize]);
    push_2_u16(out, DIGITS_U16[(n % 100) as usize]);
}

#[inline]
fn push_i64_ascii(out: &mut Vec<u8>, value: i64) {
    if value < 0 {
        out.push(b'-');
        // avoids UB for i64::MIN
        push_u64_ascii(out, value.wrapping_neg() as u64);
    } else {
        push_u64_ascii(out, value as u64);
    }
}

fn push_u64_ascii(out: &mut Vec<u8>, mut value: u64) {
    let len = num_digits(value);
    let start = out.len();

    out.reserve(len);
    let spare = out.spare_capacity_mut();
    debug_assert!(spare.len() >= len);

    let mut i = len;

    while value >= 100 {
        let idx = (value % 100) as usize;
        i -= 2;
        let bytes = DIGITS_U16[idx].to_ne_bytes();
        spare[i].write(bytes[0]);
        spare[i + 1].write(bytes[1]);
        value /= 100;
    }

    if value < 10 {
        i -= 1;
        spare[i].write(b'0' + value as u8);
    } else {
        let idx = value as usize;
        i -= 2;
        let bytes = DIGITS_U16[idx].to_ne_bytes();
        spare[i].write(bytes[0]);
        spare[i + 1].write(bytes[1]);
    }

    debug_assert_eq!(i, 0);
    unsafe { out.set_len(start + len) };
}

#[inline]
fn write_usize_ascii(dst: &mut [u8], mut n: usize) -> usize {
    let mut tmp = [0u8; 20];
    let mut i = tmp.len();
    loop {
        let digit = (n % 10) as u8;
        i -= 1;
        tmp[i] = b'0' + digit;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    let len = tmp.len() - i;
    dst[..len].copy_from_slice(&tmp[i..]);
    len
}

#[inline]
fn encode_timestamp_utc(dt: &DateTime<Utc>, out: &mut Vec<u8>) {
    push_4(out, dt.year() as u32);
    push_2(out, dt.month() as u8);
    push_2(out, dt.day() as u8);
    out.push(b'-');
    push_2(out, dt.hour() as u8);
    out.push(b':');
    push_2(out, dt.minute() as u8);
    out.push(b':');
    push_2(out, dt.second() as u8);
    out.push(b'.');
    push_3(out, dt.timestamp_subsec_millis() as u16);
}

#[inline]
fn push_checksum_3(out: &mut Vec<u8>, cksum: u8) {
    out.push(b'0' + (cksum / 100));
    out.push(b'0' + ((cksum / 10) % 10));
    out.push(b'0' + (cksum % 10));
}

/// Chainable message builder returned by `FixBuilder::begin_with`.
#[must_use = "Call .finish() to finalize the message (writes 8/9 and 10= checksum)"]
pub struct FixMsg<'a> {
    b: &'a mut FixBuilder,
}

impl<'a> FixMsg<'a> {
    /// Append a field using an owned value.
    #[inline]
    pub fn field<V: FixValue>(self, tag: u32, value: V) -> Self {
        kv(&mut self.b.buf, tag, &value);
        self
    }

    /// Append a field using a borrowed value.
    #[inline]
    pub fn field_ref<V: FixValue + ?Sized>(self, tag: u32, value: &V) -> Self {
        kv(&mut self.b.buf, tag, value);
        self
    }

    /// Append a field using a value with a compile-time tag.
    #[inline]
    pub fn field_tagged<V: FixTaggedValue>(self, value: V) -> Self {
        kv(&mut self.b.buf, V::TAG, &value);
        self
    }

    /// Append a field using a borrowed value with a compile-time tag.
    #[inline]
    pub fn field_tagged_ref<V: FixTaggedValue + ?Sized>(self, value: &V) -> Self {
        kv(&mut self.b.buf, V::TAG, value);
        self
    }

    /// Append a string field.
    #[inline]
    pub fn str(self, tag: u32, s: &str) -> Self {
        kv(&mut self.b.buf, tag, s);
        self
    }

    /// Append a raw byte field.
    #[inline]
    pub fn bytes(self, tag: u32, b: &[u8]) -> Self {
        kv(&mut self.b.buf, tag, b);
        self
    }

    /// Append a field using fallible encoding (currently only `f64` can fail).
    #[inline]
    pub fn try_field<V>(self, tag: u32, value: V) -> Result<Self, FixError>
    where
        V: TryFixValue<Error = FixError>,
    {
        try_kv(&mut self.b.buf, tag, &value)?;
        Ok(self)
    }

    /// Append a field using fallible encoding (currently only `f64` can fail).
    #[inline]
    pub fn try_field_ref<V>(self, tag: u32, value: &V) -> Result<Self, FixError>
    where
        V: TryFixValue<Error = FixError> + ?Sized,
    {
        try_kv(&mut self.b.buf, tag, value)?;
        Ok(self)
    }

    /// Append a field using fallible encoding and a compile-time tag.
    #[inline]
    pub fn try_field_tagged<V>(self, value: V) -> Result<Self, FixError>
    where
        V: FixTaggedValue + TryFixValue<Error = FixError>,
    {
        try_kv(&mut self.b.buf, V::TAG, &value)?;
        Ok(self)
    }

    /// Append a field using fallible encoding and a borrowed value with a compile-time tag.
    #[inline]
    pub fn try_field_tagged_ref<V>(self, value: &V) -> Result<Self, FixError>
    where
        V: FixTaggedValue + TryFixValue<Error = FixError> + ?Sized,
    {
        try_kv(&mut self.b.buf, V::TAG, value)?;
        Ok(self)
    }

    /// Add multiple fields using a writer (useful for optional/looped tags).
    #[inline]
    pub fn fields(self, f: impl FnOnce(&mut FixMsgWriter<'_>)) -> Self {
        let mut w = FixMsgWriter {
            buf: &mut self.b.buf,
        };
        f(&mut w);
        self
    }

    /// Add multiple fields using a writer that can fail.
    #[inline]
    pub fn try_fields<F>(self, f: F) -> Result<Self, FixError>
    where
        F: FnOnce(&mut FixMsgWriter<'_>) -> Result<(), FixError>,
    {
        let mut w = FixMsgWriter {
            buf: &mut self.b.buf,
        };
        f(&mut w)?;
        Ok(self)
    }

    /// Finalize the message (writes header/body length/checksum) and return bytes.
    #[inline]
    pub fn finish(self) -> &'a [u8] {
        self.b.finish()
    }
}

/// Helper used by `fields`/`try_fields` to append fields to a message.
pub struct FixMsgWriter<'a> {
    buf: &'a mut Vec<u8>,
}

impl<'a> FixMsgWriter<'a> {
    /// Append a field using an owned value.
    #[inline]
    pub fn field<V: FixValue>(&mut self, tag: u32, value: V) {
        kv(self.buf, tag, &value);
    }

    /// Append a field using a borrowed value.
    #[inline]
    pub fn field_ref<V: FixValue + ?Sized>(&mut self, tag: u32, v: &V) {
        kv(self.buf, tag, v);
    }

    /// Append a field using a value with a compile-time tag.
    #[inline]
    pub fn field_tagged<V: FixTaggedValue>(&mut self, value: V) {
        kv(self.buf, V::TAG, &value);
    }

    /// Append a field using a borrowed value with a compile-time tag.
    #[inline]
    pub fn field_tagged_ref<V: FixTaggedValue + ?Sized>(&mut self, value: &V) {
        kv(self.buf, V::TAG, value);
    }

    /// Append a field by value using fallible encoding (currently only `f64` can fail).
    #[inline]
    pub fn try_field<V>(&mut self, tag: u32, v: V) -> Result<(), FixError>
    where
        V: TryFixValue<Error = FixError>,
    {
        try_kv(self.buf, tag, &v)
    }

    /// Append a field by reference using fallible encoding (currently only `f64` can fail).
    #[inline]
    pub fn try_field_ref<V>(&mut self, tag: u32, v: &V) -> Result<(), FixError>
    where
        V: TryFixValue<Error = FixError> + ?Sized,
    {
        try_kv(self.buf, tag, v)
    }

    /// Append a field using fallible encoding and a compile-time tag.
    #[inline]
    pub fn try_field_tagged<V>(&mut self, value: V) -> Result<(), FixError>
    where
        V: FixTaggedValue + TryFixValue<Error = FixError>,
    {
        try_kv(self.buf, V::TAG, &value)
    }

    /// Append a field using fallible encoding and a borrowed value with a compile-time tag.
    #[inline]
    pub fn try_field_tagged_ref<V>(&mut self, value: &V) -> Result<(), FixError>
    where
        V: FixTaggedValue + TryFixValue<Error = FixError> + ?Sized,
    {
        try_kv(self.buf, V::TAG, value)
    }

    /// Append a string field.
    #[inline]
    pub fn str(&mut self, tag: u32, s: &str) {
        kv(self.buf, tag, s);
    }
    /// Append a raw byte field.
    #[inline]
    pub fn bytes(&mut self, tag: u32, b: &[u8]) {
        kv(self.buf, tag, b);
    }
}

/// Builder that encodes FIX messages into an internal buffer.
pub struct FixBuilder {
    sender: Vec<u8>,
    target: Vec<u8>,
    buf: Vec<u8>,
    fix_version: Vec<u8>,
    /// Reserved leading space sized for the worst-case `8=<ver><SOH>9=<len><SOH>` prefix.
    prefix_space: usize,
}

impl FixBuilder {
    /// Create a builder with a default buffer capacity.
    pub fn new(
        fix_version: impl Into<String>,
        sender: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::with_capacity(fix_version, sender, target, 1024)
    }

    /// Create a builder with a specific buffer capacity.
    pub fn with_capacity(
        fix_version: impl Into<String>,
        sender: impl Into<String>,
        target: impl Into<String>,
        capacity: usize,
    ) -> Self {
        let fix_version = fix_version.into().into_bytes();
        // Worst-case "8=<ver><SOH>9=<body_len><SOH>" prefix.
        let prefix_space = 2 + fix_version.len() + 1 + 2 + MAX_BODY_LEN_DIGITS + 1;
        Self {
            fix_version,
            sender: sender.into().into_bytes(),
            target: target.into().into_bytes(),
            buf: Vec::with_capacity(capacity.max(prefix_space + 64)),
            prefix_space,
        }
    }

    /// Begin a message with explicit seq/time (session layer supplies seq + dt).
    pub fn begin_with<MT, SEQ, TS>(&mut self, seq_out: &SEQ, dt: &TS, msg_type: &MT) -> FixMsg<'_>
    where
        MT: FixValue + ?Sized,
        SEQ: FixSeqNum + ?Sized,
        TS: FixSendingTime + ?Sized,
    {
        self.buf.clear();
        self.buf.resize(self.prefix_space, 0);

        let buf = &mut self.buf;
        kv(buf, 35, msg_type);
        kv(buf, 34, seq_out);
        kv_bytes(buf, 49, &self.sender);
        kv_bytes(buf, 56, &self.target);

        push_u64_ascii(buf, 52);
        buf.push(b'=');
        dt.encode_sending_time(buf);
        buf.push(SOH);
        FixMsg { b: self }
    }

    /// Finalize: patch 8/9, compute checksum, append 10, return the message bytes.
    pub fn finish(&mut self) -> &[u8] {
        let body_start = self.prefix_space;
        let body_end = self.buf.len();
        debug_assert!(body_end >= body_start);

        let body_len = body_end - body_start;

        // header: "8=<fixver><SOH>9=<len><SOH>"
        let header_len = 2 + self.fix_version.len() + 1 + 2 + num_digits(body_len) + 1;
        // By construction: prefix_space was sized for MAX_BODY_LEN_DIGITS,
        // and num_digits(body_len) <= MAX_BODY_LEN_DIGITS for any usize.
        debug_assert!(header_len <= self.prefix_space);

        let header_start = body_start - header_len;

        // Write header in-place into reserved space.
        {
            let header = &mut self.buf[header_start..body_start];
            let mut i = 0;

            header[i] = b'8';
            header[i + 1] = b'=';
            i += 2;

            header[i..i + self.fix_version.len()].copy_from_slice(&self.fix_version);
            i += self.fix_version.len();

            header[i] = SOH;
            i += 1;

            header[i] = b'9';
            header[i + 1] = b'=';
            i += 2;

            i += write_usize_ascii(&mut header[i..], body_len);

            header[i] = SOH;
            i += 1;

            debug_assert_eq!(i, header_len);
        }

        // Compute checksum over header + body (everything up to, excluding tag 10).
        let mut sum: u32 = 0;
        for &b in &self.buf[header_start..body_end] {
            sum += b as u32;
        }
        let cksum = (sum % 256) as u8;

        // Append trailer
        self.buf.extend_from_slice(b"10=");
        push_checksum_3(&mut self.buf, cksum);
        self.buf.push(SOH);

        &self.buf[header_start..]
    }
}

// --- internal writers ---
#[inline]
fn kv<V: FixValue + ?Sized>(buf: &mut Vec<u8>, tag: u32, value: &V) {
    push_u64_ascii(buf, tag as u64);
    buf.push(b'=');
    value.encode(buf);
    buf.push(SOH);
}

#[inline]
fn try_kv<V: TryFixValue<Error = FixError> + ?Sized>(
    buf: &mut Vec<u8>,
    tag: u32,
    value: &V,
) -> Result<(), FixError> {
    let start = buf.len();
    push_u64_ascii(buf, tag as u64);
    buf.push(b'=');

    if let Err(e) = value.try_encode(buf) {
        buf.truncate(start);

        // If the value used "tag=0" as a placeholder, attach the real tag here.
        return Err(match e {
            FixError::InvalidValue { tag: 0, ctx } => FixError::InvalidValue { tag, ctx },
            other => other,
        });
    }

    buf.push(SOH);
    Ok(())
}

#[inline]
fn kv_bytes(buf: &mut Vec<u8>, tag: u32, bytes: &[u8]) {
    push_u64_ascii(buf, tag as u64);
    buf.push(b'=');
    buf.extend_from_slice(bytes);
    buf.push(SOH);
}

const fn digits_00_99_u16() -> [u16; 100] {
    let mut out = [0u16; 100];
    let mut n: usize = 0;
    while n < 100 {
        let tens = b'0' + (n as u8 / 10);
        let ones = b'0' + (n as u8 % 10);
        out[n] = u16::from_ne_bytes([tens, ones]);
        n += 1;
    }
    out
}

use crate::FixError;
use core::ops::DivAssign;

trait UnsignedDigits: Copy + PartialOrd + From<u8> + DivAssign<Self> {}

impl UnsignedDigits for u32 {}
impl UnsignedDigits for u64 {}
impl UnsignedDigits for usize {}

#[inline]
fn num_digits<T>(mut value: T) -> usize
where
    T: UnsignedDigits + From<u16>,
{
    let ten: T = 10u16.into();
    let hundred: T = 100u16.into();
    let thousand: T = 1000u16.into();
    let ten_thousand: T = 10_000u16.into();

    let mut len = 0usize;

    while value >= ten_thousand {
        value /= ten_thousand;
        len += 4;
    }

    if value < hundred {
        len += if value < ten { 1 } else { 2 };
    } else {
        len += if value < thousand { 3 } else { 4 };
    }

    len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{
        HandlInst as FixHandlInst, MsgType as FixMsgType, OrdType as FixOrdType, Side as FixSide,
    };
    use crate::fix::{DayOfMonth as FixDayOfMonth, Price as FixPrice};
    use crate::tags;
    use chrono::{TimeZone, Timelike};

    // ---- Test-only FIX types ----

    fn encode_f64(value: f64) -> String {
        let mut out = Vec::new();
        value.encode(&mut out);
        String::from_utf8(out).expect("f64 encoding should be ASCII")
    }

    #[derive(Copy, Clone, Debug)]
    enum TestMsgType {
        NewOrderSingle,
    }
    impl AsFixStr for TestMsgType {
        fn as_fix_str(&self) -> &'static str {
            match self {
                TestMsgType::NewOrderSingle => "D",
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    enum HandlInst {
        Automated,
    }
    impl AsFixStr for HandlInst {
        fn as_fix_str(&self) -> &'static str {
            match self {
                HandlInst::Automated => "1",
            }
        }
    }
    impl FixTaggedValue for HandlInst {
        const TAG: u32 = tags::HANDL_INST;
    }

    #[derive(Copy, Clone, Debug)]
    enum OrdType {
        Limit,
    }
    impl AsFixStr for OrdType {
        fn as_fix_str(&self) -> &'static str {
            match self {
                OrdType::Limit => "2",
            }
        }
    }
    impl FixTaggedValue for OrdType {
        const TAG: u32 = tags::ORD_TYPE;
    }

    #[derive(Copy, Clone, Debug)]
    struct ClientOrderId(u64);
    impl FixValue for ClientOrderId {
        fn encode(&self, out: &mut Vec<u8>) {
            push_u64_ascii(out, self.0);
        }
    }

    // A minimal fixed-decimal-ish price type: mantissa + scale.
    #[derive(Copy, Clone, Debug)]
    struct Price {
        mantissa: i64,
        scale: u8,
    }
    impl FixValue for Price {
        fn encode(&self, out: &mut Vec<u8>) {
            let mut m = self.mantissa;
            if m < 0 {
                out.push(b'-');
                m = -m;
            }
            let scale = self.scale as usize;
            if scale == 0 {
                push_u64_ascii(out, m as u64);
                return;
            }

            const POW10: [u64; 19] = [
                1,
                10,
                100,
                1_000,
                10_000,
                100_000,
                1_000_000,
                10_000_000,
                100_000_000,
                1_000_000_000,
                10_000_000_000,
                100_000_000_000,
                1_000_000_000_000,
                10_000_000_000_000,
                100_000_000_000_000,
                1_000_000_000_000_000,
                10_000_000_000_000_000,
                100_000_000_000_000_000,
                1_000_000_000_000_000_000,
            ];

            let div = POW10[scale];
            let um = m as u64;
            let int_part = um / div;
            let frac_part = um % div;

            push_u64_ascii(out, int_part);
            out.push(b'.');

            let mut tmp = [b'0'; 18];
            let mut x = frac_part;
            for i in (0..scale).rev() {
                tmp[i] = b'0' + (x % 10) as u8;
                x /= 10;
            }
            out.extend_from_slice(&tmp[..scale]);
        }
    }

    // ---- Parsing helpers ----

    fn find_field(msg: &[u8], tag: u32) -> Option<&[u8]> {
        let tag_s = tag.to_string();
        let tag_b = tag_s.as_bytes();
        for part in msg.split(|&b| b == SOH) {
            if part.is_empty() {
                continue;
            }
            let Some(eq) = part.iter().position(|&b| b == b'=') else {
                continue;
            };
            if &part[..eq] == tag_b {
                return Some(&part[eq + 1..]);
            }
        }
        None
    }

    fn parse_u32_ascii(bytes: &[u8]) -> u32 {
        let mut v: u32 = 0;
        for &b in bytes {
            assert!(b.is_ascii_digit());
            v = v * 10 + (b - b'0') as u32;
        }
        v
    }

    fn locate_body_bounds(msg: &[u8]) -> (usize, usize) {
        // body starts after "9=<len><SOH>"
        let mut body_start = None;

        let mut idx = 0usize;
        for part in msg.split(|&b| b == SOH) {
            let part_len = part.len();
            if part_len == 0 {
                idx += 1;
                continue;
            }
            if let Some(eq) = part.iter().position(|&b| b == b'=')
                && &part[..eq] == b"9"
            {
                body_start = Some(idx + part_len + 1);
                break;
            }
            idx += part_len + 1;
        }
        let body_start = body_start.expect("tag 9 not found");

        // checksum field starts at "10="
        let mut checksum_tag_start = None;
        let mut idx2 = 0usize;
        for part in msg.split(|&b| b == SOH) {
            let part_len = part.len();
            if part_len == 0 {
                idx2 += 1;
                continue;
            }
            if part.starts_with(b"10=") {
                checksum_tag_start = Some(idx2);
            }
            idx2 += part_len + 1;
        }
        let checksum_tag_start = checksum_tag_start.expect("tag 10 not found");

        (body_start, checksum_tag_start)
    }

    fn verify_body_length(msg: &[u8]) {
        let body_len = parse_u32_ascii(find_field(msg, 9).expect("missing 9"));
        let (body_start, checksum_tag_start) = locate_body_bounds(msg);
        let actual = checksum_tag_start - body_start;
        assert_eq!(body_len as usize, actual, "BodyLength mismatch");
    }

    fn verify_checksum(msg: &[u8]) {
        let cksum = parse_u32_ascii(find_field(msg, 10).expect("missing 10"));
        let (_body_start, checksum_tag_start) = locate_body_bounds(msg);

        let sum: u32 = msg[..checksum_tag_start].iter().map(|&b| b as u32).sum();
        let expected = sum % 256;
        assert_eq!(cksum, expected, "CheckSum mismatch");
    }

    fn fixed_dt() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 1, 2, 3, 4, 5)
            .unwrap()
            .with_nanosecond(678_000_000)
            .unwrap()
    }

    #[derive(Debug, fixlite_derive::FixDeserialize)]
    struct RoundTripMessage<'a> {
        #[fix(tag = 8)]
        begin_string: &'a str,
        #[fix(tag = 9)]
        body_length: u32,
        #[fix(tag = 35)]
        msg_type: FixMsgType,
        #[fix(tag = 49)]
        sender_comp_id: &'a str,
        #[fix(tag = 56)]
        target_comp_id: &'a str,
        #[fix(tag = 34)]
        msg_seq_num: u64,
        #[fix(tag = 52)]
        sending_time: DateTime<Utc>,
        #[fix(tag = 21)]
        handl_inst: FixHandlInst,
        #[fix(tag = 40)]
        ord_type: FixOrdType,
        #[fix(tag = 44)]
        price: FixPrice,
        #[fix(tag = 205)]
        maturity_day: FixDayOfMonth,
        #[fix(tag = 10)]
        checksum: u8,
    }

    // ---- Tests ----

    #[test]
    fn f64_encode_handles_integers_and_fractions() {
        assert_eq!(encode_f64(0.0), "0");
        assert_eq!(encode_f64(42.0), "42");
        assert_eq!(encode_f64(1.5), "1.5");
        assert_eq!(encode_f64(1.25), "1.25");
        assert_eq!(encode_f64(1.234375), "1.234375");
    }

    #[test]
    fn f64_encode_handles_leading_fractional_zeros_and_signs() {
        assert_eq!(encode_f64(0.001953125), "0.001953125");
        assert_eq!(encode_f64(-0.5), "-0.5");
        assert_eq!(encode_f64(-1.25), "-1.25");
    }

    #[test]
    fn f64_encode_handles_simple_decimals() {
        assert_eq!(encode_f64(0.1), "0.1");
        assert_eq!(encode_f64(0.01), "0.01");
        assert_eq!(encode_f64(10.01), "10.01");
    }

    #[test]
    fn f64_encode_rounds_and_carries_whole() {
        assert_eq!(encode_f64(99_999_999_999_999.97), "100000000000000");
    }

    #[test]
    fn begin_with_finish_produces_valid_header_length_and_checksum() {
        let mut b = FixBuilder::new("FIX.4.2", "S", "T");

        let dt = fixed_dt();
        let seq = 7u32;
        let mt = TestMsgType::NewOrderSingle;

        let msg = b
            .begin_with(&seq, &dt, &mt)
            .fields(|m| {
                m.field(11, ClientOrderId(123));
                m.field(21, HandlInst::Automated);
                m.field(40, OrdType::Limit);
            })
            .finish();

        assert!(msg.starts_with(b"8=FIX.4.2\x01"), "Missing BeginString");
        assert_eq!(find_field(msg, 35).unwrap(), b"D");
        assert_eq!(find_field(msg, 34).unwrap(), b"7");
        assert_eq!(find_field(msg, 49).unwrap(), b"S");
        assert_eq!(find_field(msg, 56).unwrap(), b"T");

        verify_body_length(msg);
        verify_checksum(msg);
    }

    #[test]
    fn custom_types_are_encoded_correctly() {
        let mut b = FixBuilder::new("FIX.4.2", "SENDER", "TARGET");

        let dt = fixed_dt();
        let seq = 1u32;
        let mt = TestMsgType::NewOrderSingle;

        let cl = ClientOrderId(999_001);
        let px = Price {
            mantissa: 12345,
            scale: 2,
        };

        let msg = b
            .begin_with(&seq, &dt, &mt)
            .field(tags::CL_ORD_ID, cl)
            .field_tagged(HandlInst::Automated)
            .field_tagged(OrdType::Limit)
            .field(44, px)
            .finish();

        assert_eq!(find_field(msg, 11).unwrap(), b"999001");
        assert_eq!(find_field(msg, 21).unwrap(), b"1");
        assert_eq!(find_field(msg, 40).unwrap(), b"2");
        assert_eq!(find_field(msg, 44).unwrap(), b"123.45");

        verify_body_length(msg);
        verify_checksum(msg);
    }

    #[test]
    fn builder_reuse_does_not_leak_previous_fields() {
        let mut b = FixBuilder::new("FIX.4.2", "S", "T");

        let dt = fixed_dt();
        let mt = TestMsgType::NewOrderSingle;

        let seq1 = 1u32;
        let msg1 = b.begin_with(&seq1, &dt, &mt).str(9999, "LEAKME").finish();

        assert!(find_field(msg1, 9999).is_some());

        let seq2 = 2u32;
        let msg2 = b
            .begin_with(&seq2, &dt, &mt)
            .field(11, ClientOrderId(1))
            .finish();

        assert!(
            find_field(msg2, 9999).is_none(),
            "Field leaked across messages"
        );

        verify_body_length(msg2);
        verify_checksum(msg2);
    }

    #[test]
    fn macro_build_fix_builds_message_and_validates_checksum_and_length() {
        let mut builder = FixBuilder::new("FIX.4.2", "S", "T");

        let cl = ClientOrderId(77);
        let dt = fixed_dt();

        let fix_message = build_fix!(
            builder,
            42u32,
            dt,
            TestMsgType::NewOrderSingle,
            tags::CL_ORD_ID => cl,
            @HandlInst::Automated,
            @OrdType::Limit,
        );

        assert!(fix_message.starts_with(b"8=FIX.4.2\x01"));
        assert_eq!(find_field(fix_message, 35).unwrap(), b"D");
        assert_eq!(find_field(fix_message, 34).unwrap(), b"42");
        assert_eq!(find_field(fix_message, 11).unwrap(), b"77");

        verify_body_length(fix_message);
        verify_checksum(fix_message);
    }

    #[test]
    fn macro_build_fix_accepts_enum_references() {
        let mut builder = FixBuilder::new("FIX.4.2", "S", "T");

        let dt = fixed_dt();
        let seq = 7u32;
        let fix_message = build_fix!(
            builder,
            seq,
            dt,
            FixMsgType::NewOrderSingle,
            @FixSide::Buy,
            @FixOrdType::Limit,
        );

        assert_eq!(find_field(fix_message, 35).unwrap(), b"D");
        assert_eq!(find_field(fix_message, 34).unwrap(), b"7");
        assert_eq!(find_field(fix_message, 54).unwrap(), b"1");
        assert_eq!(find_field(fix_message, 40).unwrap(), b"2");

        verify_body_length(fix_message);
        verify_checksum(fix_message);
    }

    #[test]
    fn macro_build_fix_supports_arrow_and_tagged_values() {
        let mut builder = FixBuilder::new("FIX.4.2", "S", "T");

        let dt = fixed_dt();
        let fix_message = build_fix!(
            builder,
            3u32,
            dt,
            FixMsgType::NewOrderSingle,
            tags::SIDE => FixSide::Buy,
            tags::ORD_TYPE => FixOrdType::Limit,
            @FixHandlInst::Automated,
        );

        assert_eq!(find_field(fix_message, 35).unwrap(), b"D");
        assert_eq!(find_field(fix_message, 34).unwrap(), b"3");
        assert_eq!(find_field(fix_message, tags::SIDE).unwrap(), b"1");
        assert_eq!(find_field(fix_message, tags::ORD_TYPE).unwrap(), b"2");
        assert_eq!(find_field(fix_message, tags::HANDL_INST).unwrap(), b"1");

        verify_body_length(fix_message);
        verify_checksum(fix_message);
    }

    #[test]
    fn field_tagged_uses_enum_tag_constants() {
        let mut b = FixBuilder::new("FIX.4.2", "S", "T");

        let dt = fixed_dt();
        let seq = 9u32;
        let mt = FixMsgType::NewOrderSingle;
        let side = FixSide::Buy;
        let ord_type = FixOrdType::Limit;

        let msg = b
            .begin_with(&seq, &dt, &mt)
            .field_tagged_ref(&side)
            .field_tagged(ord_type)
            .finish();

        assert_eq!(find_field(msg, tags::SIDE).unwrap(), b"1");
        assert_eq!(find_field(msg, tags::ORD_TYPE).unwrap(), b"2");

        verify_body_length(msg);
        verify_checksum(msg);
    }

    #[test]
    fn builder_round_trip_with_fix_types() {
        let mut b = FixBuilder::new("FIX.4.2", "SENDER", "TARGET");

        let dt = fixed_dt();
        let seq = 42;
        let price: FixPrice = "123.4500".parse().unwrap();
        let day = FixDayOfMonth(7);

        let msg = b
            .begin_with(&seq, &dt, &FixMsgType::NewOrderSingle)
            .field(21, FixHandlInst::Automated)
            .field(40, FixOrdType::Limit)
            .field(44, price)
            .field(205, day)
            .finish();

        let parsed = <RoundTripMessage as crate::FixDeserialize>::from_fix(msg).unwrap();
        assert_eq!(parsed.begin_string, "FIX.4.2");
        assert_eq!(parsed.msg_type, FixMsgType::NewOrderSingle);
        assert_eq!(parsed.sender_comp_id, "SENDER");
        assert_eq!(parsed.target_comp_id, "TARGET");
        assert_eq!(parsed.msg_seq_num, seq);
        assert_eq!(parsed.sending_time, dt);
        assert_eq!(parsed.handl_inst, FixHandlInst::Automated);
        assert_eq!(parsed.ord_type, FixOrdType::Limit);
        assert_eq!(parsed.price, price);
        assert_eq!(parsed.maturity_day, day);
        assert_eq!(
            parsed.body_length,
            parse_u32_ascii(find_field(msg, 9).unwrap())
        );
        assert_eq!(
            parsed.checksum as u32,
            parse_u32_ascii(find_field(msg, 10).unwrap())
        );

        verify_body_length(msg);
        verify_checksum(msg);
    }

    #[test]
    fn fields_closure_is_nicer_for_conditionals_and_loops() {
        let mut b = FixBuilder::new("FIX.4.2", "S", "T");

        let dt = fixed_dt();
        let seq = 100u32;
        let mt = TestMsgType::NewOrderSingle;

        // pretend these come from a higher layer:
        let cl_ord_id = Some(ClientOrderId(777));
        let account: Option<&str> = None;

        // “extras” are dynamic (e.g. strategy tags / venue-specific tags)
        let extras: &[(u32, &str)] = &[
            (58, "hello"), // Text
            (100, "XNAS"), // ExDestination
            (110, "1"),    // MinQty
        ];

        let msg = b
            .begin_with(&seq, &dt, &mt)
            // fields() is most ergonomic when you want to:
            // - conditionally add fields (lots of if let / match)
            // - loop over collections (repeating groups / “optional extras”)
            // - avoid repeating .field(...) chains that get ugly with branching
            .fields(|m| {
                // required-ish fields
                m.field(21, HandlInst::Automated);
                m.field(40, OrdType::Limit);

                // conditional fields without breaking the flow
                if let Some(cl) = cl_ord_id {
                    m.field(11, cl);
                }
                if let Some(acct) = account {
                    m.str(1, acct); // Account(1)
                }

                // looped/dynamic fields (nice for repeating groups / “extras”)
                for &(tag, val) in extras {
                    m.str(tag, val);
                }
            })
            .finish();

        // spot-check a couple fields exist
        assert_eq!(find_field(msg, 34).unwrap(), b"100");
        assert_eq!(find_field(msg, 11).unwrap(), b"777");
        assert_eq!(find_field(msg, 58).unwrap(), b"hello");
        assert_eq!(find_field(msg, 100).unwrap(), b"XNAS");

        verify_body_length(msg);
        verify_checksum(msg);
    }

    #[test]
    fn long_fix_version_does_not_overflow_prefix() {
        // "FIXT.1.1" is 8 bytes; combined with a large body it would have
        // overflowed the old hard-coded HEADER_SPACE = 32. The prefix is now
        // sized at construction from fix_version.len() + MAX_BODY_LEN_DIGITS,
        // so this must produce a well-formed message.
        let mut b = FixBuilder::new("FIXT.1.1", "S", "T");
        let dt = fixed_dt();
        let seq = 1u32;

        // Build a message with a large body to exercise multi-digit body lengths.
        let big_text: String = "x".repeat(5000);
        let msg = b
            .begin_with(&seq, &dt, &FixMsgType::NewOrderSingle)
            .field_ref(58, &big_text)
            .finish();

        assert!(msg.starts_with(b"8=FIXT.1.1\x01"));
        verify_body_length(msg);
        verify_checksum(msg);
    }

    #[test]
    fn try_field_f64_rejects_nan() {
        let mut b = FixBuilder::new("FIX.4.2", "S", "T");
        let dt = fixed_dt();
        let seq = 1u32;
        let mt = TestMsgType::NewOrderSingle;

        let err = match b.begin_with(&seq, &dt, &mt).try_field_ref(44, &f64::NAN) {
            Ok(_) => panic!("expected error"),
            Err(e) => e,
        };

        assert!(matches!(err, FixError::InvalidValue { tag: 44, .. }));
    }
}
