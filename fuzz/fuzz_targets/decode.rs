//! Fuzz target for `fixlite::decode`.
//!
//! Run with:
//!   cargo +nightly fuzz run decode
//!
//! The struct intentionally declares every field `Option<T>` so the parser
//! runs to completion on most inputs — we are testing that the parse path
//! does not panic, hang, overflow, or trip a sanitizer, not whether the
//! input is semantically valid FIX.

#![no_main]

use libfuzzer_sys::fuzz_target;
use chrono::{DateTime, Utc};
use fixlite::FixDeserialize;
use fixlite::enums::{MsgType, OrdType, Side};

#[derive(FixDeserialize, Debug)]
struct FuzzMessage<'a> {
    #[fix(tag = 8)]
    _begin_string: Option<&'a str>,
    #[fix(tag = 9)]
    _body_length: Option<u32>,
    #[fix(tag = 35)]
    _msg_type: Option<MsgType>,
    #[fix(tag = 49)]
    _sender_comp_id: Option<&'a str>,
    #[fix(tag = 56)]
    _target_comp_id: Option<&'a str>,
    #[fix(tag = 34)]
    _msg_seq_num: Option<u64>,
    #[fix(tag = 52)]
    _sending_time: Option<DateTime<Utc>>,
    #[fix(tag = 11)]
    _cl_ord_id: Option<&'a str>,
    #[fix(tag = 55)]
    _symbol: Option<&'a str>,
    #[fix(tag = 54)]
    _side: Option<Side>,
    #[fix(tag = 40)]
    _ord_type: Option<OrdType>,
    #[fix(tag = 10)]
    _checksum: Option<u8>,
}

fuzz_target!(|data: &[u8]| {
    let _ = fixlite::decode::<FuzzMessage>(data);
});
