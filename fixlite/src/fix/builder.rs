use super::{DayOfMonth, price::FixedPrice};
use chrono::{DateTime, Datelike, Timelike, Utc};

pub const SOH: u8 = 0x01;
const HEADER_SPACE: usize = 32; // bytes reserved for 8 & 9

#[macro_export]
macro_rules! build_fix {
    ($builder:expr, $seq_out:expr, $dt:expr, $msg_type:expr $(, $tag:expr, $val:expr )* $(,)?) => {{
        let builder = &mut $builder;
        builder.begin_with(&$seq_out, &$dt, &$msg_type);
        $(
            builder.field($tag as u32, &$val);
        )*
        builder.finish()
    }};
}

/// Values that can be encoded as a FIX field value (no heap allocation required).
pub trait FixValue {
    fn encode(&self, out: &mut Vec<u8>);
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
    fn encode_sending_time(&self, out: &mut Vec<u8>);
}

// --- Common FixSendingTime impls ---
impl FixSendingTime for DateTime<Utc> {
    #[inline]
    fn encode_sending_time(&self, out: &mut Vec<u8>) {
        encode_timestamp_utc(self, out);
    }
}

/// FixSendingTime wrapper for a preformatted str
pub struct SendingTimeStr<'a>(pub &'a str);

impl FixSendingTime for SendingTimeStr<'_> {
    #[inline]
    fn encode_sending_time(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.0.as_bytes());
    }
}

pub struct SendingTimeBytes<'a>(pub &'a [u8]);

impl FixSendingTime for SendingTimeBytes<'_> {
    #[inline]
    fn encode_sending_time(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.0);
    }
}

/// Convenience trait for enums that map to a static FIX code ("D", "1", "2", ...).
pub trait AsFixStr {
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
        if *self < 0 {
            out.push(b'-');
            // NOTE: i64::MIN will wrap here; if that's a concern, handle it explicitly.
            push_u64_ascii(out, self.wrapping_neg() as u64);
        } else {
            push_u64_ascii(out, *self as u64);
        }
    }
}
impl FixValue for i32 {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        (*self as i64).encode(out)
    }
}

impl FixValue for f64 {
    #[inline]
    fn encode(&self, out: &mut Vec<u8>) {
        let mut buf = ryu::Buffer::new();
        out.extend_from_slice(buf.format(*self).as_bytes());
    }
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
fn digits_len(mut n: usize) -> usize {
    let mut d = 1;
    while n >= 10 {
        n /= 10;
        d += 1;
    }
    d
}

#[inline]
fn push_2(out: &mut Vec<u8>, n: u8) {
    out.push(b'0' + (n / 10));
    out.push(b'0' + (n % 10));
}

#[inline]
fn push_3(out: &mut Vec<u8>, n: u16) {
    out.push(b'0' + ((n / 100) as u8));
    out.push(b'0' + (((n / 10) % 10) as u8));
    out.push(b'0' + ((n % 10) as u8));
}

#[inline]
fn push_4(out: &mut Vec<u8>, n: u32) {
    out.push(b'0' + (((n / 1000) % 10) as u8));
    out.push(b'0' + (((n / 100) % 10) as u8));
    out.push(b'0' + (((n / 10) % 10) as u8));
    out.push(b'0' + ((n % 10) as u8));
}

#[inline]
fn push_u64_ascii(out: &mut Vec<u8>, mut n: u64) {
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
    out.extend_from_slice(&tmp[i..]);
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

pub struct FixBuilder {
    sender: Vec<u8>,
    target: Vec<u8>,
    buf: Vec<u8>,
    fix_version: Vec<u8>,
}

impl FixBuilder {
    pub fn new(
        fix_version: impl Into<String>,
        sender: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::with_capacity(fix_version, sender, target, 1024)
    }

    pub fn with_capacity(
        fix_version: impl Into<String>,
        sender: impl Into<String>,
        target: impl Into<String>,
        capacity: usize,
    ) -> Self {
        Self {
            fix_version: fix_version.into().into_bytes(),
            sender: sender.into().into_bytes(),
            target: target.into().into_bytes(),
            buf: Vec::with_capacity(capacity.max(HEADER_SPACE + 64)),
        }
    }

    /// Begin a message with explicit seq/time (session layer supplies seq + dt).
    pub fn begin_with<MT, SEQ, TS>(&mut self, seq_out: &SEQ, dt: &TS, msg_type: &MT)
    where
        MT: FixValue + ?Sized,
        SEQ: FixSeqNum + ?Sized,
        TS: FixSendingTime + ?Sized,
    {
        self.buf.clear();
        self.buf.resize(HEADER_SPACE, 0);

        let buf = &mut self.buf;
        kv(buf, 35, msg_type);
        kv(buf, 34, seq_out);
        kv_bytes(buf, 49, &self.sender);
        kv_bytes(buf, 56, &self.target);

        push_u64_ascii(buf, 52);
        buf.push(b'=');
        dt.encode_sending_time(buf);
        buf.push(SOH);
    }

    #[inline]
    pub fn field<V: FixValue + ?Sized>(&mut self, tag: u32, value: &V) {
        kv(&mut self.buf, tag, value);
    }

    /// Finalize: patch 8/9, compute checksum, append 10, return the message bytes.
    pub fn finish(&mut self) -> &[u8] {
        let body_start = HEADER_SPACE;
        let body_end = self.buf.len();
        debug_assert!(body_end >= body_start);

        let body_len = body_end - body_start;

        // header: "8=<fixver><SOH>9=<len><SOH>"
        let header_len = 2 + self.fix_version.len() + 1 + 2 + digits_len(body_len) + 1;
        debug_assert!(header_len <= HEADER_SPACE);

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
fn kv_bytes(buf: &mut Vec<u8>, tag: u32, bytes: &[u8]) {
    push_u64_ascii(buf, tag as u64);
    buf.push(b'=');
    buf.extend_from_slice(bytes);
    buf.push(SOH);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::{
        DayOfMonth as FixDayOfMonth, HandlInst as FixHandlInst, MsgType as FixMsgType,
        OrdType as FixOrdType, Price as FixPrice,
    };
    use chrono::{TimeZone, Timelike};

    // ---- Test-only FIX types ----

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
    fn begin_with_finish_produces_valid_header_length_and_checksum() {
        let mut b = FixBuilder::new("FIX.4.2", "S", "T");

        let dt = fixed_dt();
        let seq = 7u32;
        let mt = TestMsgType::NewOrderSingle;

        b.begin_with(&seq, &dt, &mt);
        b.field(11, &ClientOrderId(123));
        b.field(21, &HandlInst::Automated);
        b.field(40, &OrdType::Limit);

        let msg = b.finish();

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

        b.begin_with(&seq, &dt, &mt);
        b.field(11, &cl);
        b.field(21, &HandlInst::Automated);
        b.field(40, &OrdType::Limit);
        b.field(44, &px);

        let msg = b.finish();

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
        b.begin_with(&seq1, &dt, &mt);
        b.field(9999, "LEAKME");
        let msg1 = b.finish();
        assert!(find_field(msg1, 9999).is_some());

        let seq2 = 2u32;
        b.begin_with(&seq2, &dt, &mt);
        b.field(11, &ClientOrderId(1));
        let msg2 = b.finish();
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
            11,
            cl,
            21,
            HandlInst::Automated,
            40,
            OrdType::Limit,
        );

        assert!(fix_message.starts_with(b"8=FIX.4.2\x01"));
        assert_eq!(find_field(fix_message, 35).unwrap(), b"D");
        assert_eq!(find_field(fix_message, 34).unwrap(), b"42");
        assert_eq!(find_field(fix_message, 11).unwrap(), b"77");

        verify_body_length(fix_message);
        verify_checksum(fix_message);
    }

    #[test]
    fn builder_round_trip_with_fix_types() {
        let mut b = FixBuilder::new("FIX.4.2", "SENDER", "TARGET");

        let dt = fixed_dt();
        let seq = 42u64;
        let mt = FixMsgType::NewOrderSingle;
        let price: FixPrice = "123.4500".parse().unwrap();
        let day = FixDayOfMonth(7);

        b.begin_with(&seq, &dt, &mt);
        b.field(21, &FixHandlInst::Automated);
        b.field(40, &FixOrdType::Limit);
        b.field(44, &price);
        b.field(205, &day);

        let msg = b.finish();

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
}
