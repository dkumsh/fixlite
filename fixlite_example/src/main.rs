// src/main.rs

use chrono::{DateTime, Utc};
use fixlite::fix::DayOfMonth;
use fixlite_derive::FixDeserialize;

#[allow(dead_code)]
#[derive(FixDeserialize, Debug)]
struct MarketDataMessage<'a> {
    #[fix(tag = 8)]
    pub begin_string: &'a str,
    #[fix(tag = 9)]
    pub body_length: u32,
    #[fix(tag = 35)]
    pub msg_type: &'a str,
    #[fix(tag = 49)]
    pub sender_comp_id: &'a str,
    #[fix(tag = 56)]
    pub target_comp_id: &'a str,
    #[fix(tag = 34)]
    pub msg_seq_num: u64,
    #[fix(tag = 52)]
    pub sending_time: DateTime<Utc>,
    #[fix(tag = 55)]
    pub symbol: Option<&'a str>,
    #[fix(tag = 205)]
    day: DayOfMonth,
    #[fix(tag = 262)]
    pub md_req_id: &'a str,
    #[fix_group(tag = 268)]
    pub md_entries: Vec<MDEntry<'a>>,
    #[fix(tag = 10)]
    pub checksum: u8,
}

#[allow(dead_code)]
#[derive(FixDeserialize, Debug)]
struct MDEntry<'a> {
    #[fix(tag = 269)]
    pub md_entry_type: &'a str,
    #[fix(tag = 270)]
    pub md_entry_px: f64,
    #[fix(tag = 271)]
    pub md_entry_size: f64,
    #[fix(tag = 272)]
    pub md_entry_date: DateTime<Utc>,
}
#[allow(dead_code)]
#[derive(FixDeserialize, Debug)]
struct Day {
    #[fix(tag = 205)]
    day: DayOfMonth,
}

#[derive(Debug, FixDeserialize)]
pub struct Component<'c> {
    #[fix(tag = 123)]
    pub a1: &'c str,
    #[fix(tag = 124)]
    pub a2: Option<&'c str>,
}

#[derive(Debug, FixDeserialize)]
pub struct Container<'a> {
    #[fix(tag = 125)]
    pub b1: &'a str,
    #[fix(component)]
    pub c: Option<Component<'a>>, // <- optional component
}

fn main() {}

#[cfg(test)]
mod tests {
    use crate::{Container, MarketDataMessage};
    use fixlite::FixDeserialize;

    #[test]
    fn repeating_group_test() {
        let fix_message = b"8=FIX.4.4|9=31226|35=W|49=DERIBITSERVER|56=gsr01|34=2|52=20240918-12:11:46.594|55=BTC-PERPETUAL|\
231=10.0|100087=26105623|100090=59806.74|205=7|746=843249447.0|100092=0.0|100093=0.00066246|262=1|268=2|\
269=0|270=59765.5|271=1.0|272=20240918-12:11:46.529|269=1|270=150000.0|271=1810000.0|272=20240918-12:11:46.529|10=163|";

        let parsed: MarketDataMessage =
            MarketDataMessage::from_fix_message(fix_message, Some('|')).unwrap();
        println!("{:#?}", parsed);
    }
    #[test]
    fn component_test() {
        let fix_message = b"8=FIX.4.4|9=31226|35=W|125=bar|123=a1|124=a2|10=100|";
        let parsed: Container = Container::from_fix_message(fix_message, Some('|')).unwrap();
        println!("{:#?}", parsed);
    }
}
