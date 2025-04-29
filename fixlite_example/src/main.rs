// src/main.rs

use chrono::{DateTime, Utc};
use fixlite::fix::MDEntryType;
use fixlite::FixDeserialize;
use fixlite_derive::FixDeserialize;

#[allow(dead_code)]
#[derive(FixDeserialize, Debug)]
struct MarketDataMessage<'a> {
    #[fix(tag = "8", type = "String")]
    pub begin_string: &'a str,
    #[fix(tag = "9", type = "u32")]
    pub body_length: u32,
    #[fix(tag = "35", type = "String")]
    pub msg_type: &'a str,
    #[fix(tag = "49", type = "& 'a str")]
    pub sender_comp_id: &'a str,
    #[fix(tag = "56", type = "& 'a str")]
    pub target_comp_id: &'a str,
    #[fix(tag = "34", type = "u32")]
    pub msg_seq_num: u32,
    #[fix(tag = "52", type = "UTC_TIMESTAMP")]
    pub sending_time: DateTime<Utc>,
    #[fix(tag = "55", type = "& 'a str")]
    pub symbol: Option<&'a str>,
    #[fix(tag = "262", type = "& 'a str")]
    pub md_req_id: &'a str,
    #[fix_group(tag = "268")]
    pub md_entries: Vec<MDEntry>,
    #[fix(tag = "10", type = "u8")]
    pub checksum: u8,
}

#[allow(dead_code)]
#[derive(FixDeserialize, Debug)]
struct MDEntry {
    #[fix(tag = "269", type = "u8")]
    pub md_entry_type: MDEntryType,
    #[fix(tag = "270", type = "f64")]
    pub md_entry_px: f64,
    #[fix(tag = "271", type = "f64")]
    pub md_entry_size: f64,
    #[fix(tag = "272", type = "UTC_TIMESTAMP")]
    pub md_entry_date: DateTime<Utc>,
}

fn main() {
    let fix_message = b"8=FIX.4.4|9=31226|35=W|49=DERIBITSERVER|56=gsr01|34=2|52=20240918-12:11:46.594|55=BTC-PERPETUAL|\
231=10.0|100087=26105623|100090=59806.74|746=843249447.0|100092=0.0|100093=0.00066246|262=1|268=2|\
269=0|270=59765.5|271=1.0|272=20240918-12:11:46.529|269=1|270=150000.0|271=1810000.0|272=20240918-12:11:46.529|10=163|";

    let parsed: MarketDataMessage =
        MarketDataMessage::from_fix_message(fix_message, Some('|')).unwrap();
    println!("{:#?}", parsed);
}
