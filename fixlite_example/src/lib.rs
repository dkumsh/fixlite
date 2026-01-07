#![allow(dead_code)]
pub mod fixparser;

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use fixlite::enums::MsgType;
    use fixlite::fix::DayOfMonth;
    use fixlite::{FixDeserialize, fix_tag_registry};

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
        const SOH: u8 = 0x01;
        let bytes: Vec<u8> = message
            .iter()
            .map(|&b| if b == b'|' { SOH } else { b })
            .collect();

        let mut begin_string: Option<&[u8]> = None;
        let mut body_fields: Vec<&[u8]> = Vec::new();

        for field in bytes.split(|&b| b == SOH) {
            if field.is_empty() {
                continue;
            }
            let Some(eq) = field.iter().position(|&b| b == b'=') else {
                continue;
            };
            let tag = &field[..eq];
            let value = &field[eq + 1..];
            match tag {
                b"8" => begin_string = Some(value),
                b"9" | b"10" => {}
                _ => body_fields.push(field),
            }
        }

        let begin_string = begin_string.expect("missing BeginString (8)");
        let body_len: usize = body_fields.iter().map(|f| f.len() + 1).sum();

        let mut out = Vec::with_capacity(bytes.len() + 16);
        out.extend_from_slice(b"8=");
        out.extend_from_slice(begin_string);
        out.push(SOH);
        out.extend_from_slice(b"9=");
        out.extend_from_slice(body_len.to_string().as_bytes());
        out.push(SOH);

        for field in body_fields {
            out.extend_from_slice(field);
            out.push(SOH);
        }

        let sum: u32 = out.iter().map(|&b| b as u32).sum();
        let checksum = (sum % 256) as u8;
        out.extend_from_slice(b"10=");
        out.push(b'0' + (checksum / 100));
        out.push(b'0' + ((checksum / 10) % 10));
        out.push(b'0' + (checksum % 10));
        out.push(SOH);

        out
    }
    #[test]
    fn repeating_group_test() {
        let message = fix(b"8=FIX.4.4|9=31226|35=W|49=DERIBITSERVER|56=gsr01|34=2|52=20240918-12:11:46.594|55=BTC-PERPETUAL|\
231=10.0|100087=26105623|100090=59806.74|205=7|746=843249447.0|100092=0.0|100093=0.00066246|262=1|268=2|\
269=0|270=59765.5|271=1.0|272=20240918-12:11:46.529|269=1|270=150000.0|271=1810000.0|272=20240918-12:11:46.529|10=163|");

        let parsed: MarketDataMessage = fixlite::decode(&message).unwrap();
        println!("{:#?}", parsed);
    }
    #[test]
    fn component_test() {
        let message = fix(b"8=FIX.4.4|9=31226|35=W|125=bar|123=a1|124=a2|10=100|");
        let parsed: Container = fixlite::decode(&message).unwrap();
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
        let parsed: TestMessage = fixlite::decode(&message).unwrap();
        assert_eq!(parsed.custom_price, 62.2); // 8001=62.20
        assert_eq!(parsed.begin, "FIX.4.2"); // 8=FIX.4.2
        assert_eq!(parsed.msg_type, MsgType::MarketDataSnapshotFullRefresh); // "35=W"
        assert_eq!(parsed.last_px, None); // 31 is missing
    }
}
