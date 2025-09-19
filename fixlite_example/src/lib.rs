#![allow(dead_code)]
pub mod fixparser;

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use fixlite::fix::DayOfMonth;
    use fixlite::fix::MsgType;
    use fixlite::{FixDeserialize, fix_tag_registry};
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

    fn fix(message: &[u8]) -> Vec<u8> {
        message
            .iter()
            .map(|&b| if b == b'|' { b'\x01' } else { b })
            .collect()
    }
    #[test]
    fn repeating_group_test() {
        let message = fix(b"8=FIX.4.4|9=31226|35=W|49=DERIBITSERVER|56=gsr01|34=2|52=20240918-12:11:46.594|55=BTC-PERPETUAL|\
231=10.0|100087=26105623|100090=59806.74|205=7|746=843249447.0|100092=0.0|100093=0.00066246|262=1|268=2|\
269=0|270=59765.5|271=1.0|272=20240918-12:11:46.529|269=1|270=150000.0|271=1810000.0|272=20240918-12:11:46.529|10=163|");

        let parsed: MarketDataMessage = MarketDataMessage::from_fix(&message).unwrap();
        println!("{:#?}", parsed);
    }
    #[test]
    fn component_test() {
        let message = fix(b"8=FIX.4.4|9=31226|35=W|125=bar|123=a1|124=a2|10=100|");
        let parsed: Container = Container::from_fix(&message).unwrap();
        println!("{:#?}", parsed);
    }
    #[test]
    fn test_custom_registry() {
        fix_tag_registry! {
            MyRegistry {
                35   => [u32,MsgType],
                31   => [f64], // LastPx
                8001   => [f64],
            }
        }

        #[derive(FixDeserialize, Debug)]
        #[fix_registry(MyRegistry)]
        struct TestMessage<'a> {
            #[fix(tag = 8001)]
            pub custom_price: f64,
            #[fix(tag = 8)]
            pub begin: &'a str,
            #[fix(tag = 35)]
            pub msg_type: MsgType,
            #[fix(tag = 31)]
            pub last_px: Option<f64>,
        }

        let message = fix(b"8=FIX.4.2|9=31226|35=W|8001=62.20|10=100|");
        let parsed: TestMessage = TestMessage::from_fix(&message).unwrap();
        assert_eq!(parsed.custom_price, 62.2); // 8001=62.20
        assert_eq!(parsed.begin, "FIX.4.2"); // 8=FIX.4.2
        assert_eq!(parsed.msg_type, MsgType::MarketDataSnapshotFullRefresh); // "35=W"
        assert_eq!(parsed.last_px, None); // 31 is missing
    }
}
