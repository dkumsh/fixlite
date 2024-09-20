// src/main.rs

use chrono::{DateTime, Utc};
use fix::FixDeserialize;
use fix_derive::FixDeserialize;

#[derive(Default, Debug)]
struct MarketDataMessage {
    // #[fix(tag = "8", type = "String")]
    begin_string: String,
    // #[fix(tag = "9", type = "u32")]
    body_length: u32,
    // #[fix(tag = "35", type = "String")]
    msg_type: String,
    // #[fix(tag = "49", type = "String")]
    sender_comp_id: String,
    // #[fix(tag = "56", type = "String")]
    target_comp_id: String,
    // #[fix(tag = "34", type = "u32")]
    msg_seq_num: u32,
    // #[fix(tag = "52", type = "UTC_TIMESTAMP")]
    sending_time: DateTime<Utc>,
    // #[fix(tag = "55", type = "String")]
    symbol: String,
    // #[fix(tag = "262", type = "String")]
    md_req_id: String,
    // #[fix_group(tag = "268")]
    md_entries: Vec<MDEntry>,
    // #[fix(tag = "10", type = "u8")]
    checksum: u8,
}
impl ::fix::FixDeserialize for MarketDataMessage {
    fn from_fix_message(fix_message: &[u8]) -> Result<Self, ::fix::FixError> {
        let fix_message_str = std::str::from_utf8(fix_message)?;
        let mut fields = fix_message_str.split('|').peekable();
        Self::from_fix_message_iter(&mut fields)
    }
    fn from_fix_message_iter<'a, I>(fields: &mut std::iter::Peekable<I>) -> Result<Self, ::fix::FixError> where I: Iterator<Item=&'a str>, {
        use chrono::{NaiveDateTime, DateTime, Utc};
        let mut first_tag = None;
        let mut begin_string_tmp: Option<String> = None;
        let mut body_length_tmp: Option<u32> = None;
        let mut msg_type_tmp: Option<String> = None;
        let mut sender_comp_id_tmp: Option<String> = None;
        let mut target_comp_id_tmp: Option<String> = None;
        let mut msg_seq_num_tmp: Option<u32> = None;
        let mut sending_time_tmp: Option<DateTime<Utc>> = None;
        let mut symbol_tmp: Option<String> = None;
        let mut md_req_id_tmp: Option<String> = None;
        let mut md_entries_tmp: Option<Vec<MDEntry>> = None;
        let mut checksum_tmp: Option<u8> = None;
        while let Some(field) = fields.peek().map(|x| *x) {
            if field.is_empty() {
                fields.next();
                continue;
            }
            let mut parts = field.splitn(2, '=');
            let tag = parts.next().unwrap();
            println!("FIELD parts: {:?}={:?}", tag, parts.next().unwrap());
            if first_tag.is_none() {
                first_tag = Some(tag);
            } else if tag == first_tag.unwrap() {
                break;
            }
            match tag {
                "8" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    begin_string_tmp = Some(value.to_string());
                }
                "9" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    body_length_tmp = Some(value.parse::<u32>().map_err(|_| ::fix::FixError::InvalidValue("9".to_string()))?);
                }
                "35" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    msg_type_tmp = Some(value.to_string());
                }
                "49" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    sender_comp_id_tmp = Some(value.to_string());
                }
                "56" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    target_comp_id_tmp = Some(value.to_string());
                }
                "34" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    msg_seq_num_tmp = Some(value.parse::<u32>().map_err(|_| ::fix::FixError::InvalidValue("34".to_string()))?);
                }
                "52" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    sending_time_tmp = Some({
                        let dt = NaiveDateTime::parse_from_str(value, "%Y%m%d-%H:%M:%S%.f")?;
                        DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)
                    });
                }
                "55" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    symbol_tmp = Some(value.to_string());
                }
                "262" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    md_req_id_tmp = Some(value.to_string());
                }
                "268" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    let tag = parts.next();
                    let value = parts.next().unwrap();
                    let group_count = value.parse::<usize>().map_err(|_| ::fix::FixError::InvalidValue("268".to_string()))?;
                    let mut entries = Vec::with_capacity(group_count);
                    for i in 0..group_count {
                        let entry = <MDEntry as ::fix::FixDeserialize>::from_fix_message_iter(fields)?;
                        entries.push(entry);
                    }
                    md_entries_tmp = Some(entries);
                }
                "10" => {
                    println!("GOT tag 10");
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    checksum_tmp = Some(value.parse::<u8>().map_err(|_| ::fix::FixError::InvalidValue("10".to_string()))?);
                }
                _ => { fields.next(); }
            }
        }
        let begin_string = begin_string_tmp.ok_or(::fix::FixError::MissingField(stringify!(begin_string)))?;
        let body_length = body_length_tmp.ok_or(::fix::FixError::MissingField(stringify!(body_length)))?;
        let msg_type = msg_type_tmp.ok_or(::fix::FixError::MissingField(stringify!(msg_type)))?;
        let sender_comp_id = sender_comp_id_tmp.ok_or(::fix::FixError::MissingField(stringify!(sender_comp_id)))?;
        let target_comp_id = target_comp_id_tmp.ok_or(::fix::FixError::MissingField(stringify!(target_comp_id)))?;
        let msg_seq_num = msg_seq_num_tmp.ok_or(::fix::FixError::MissingField(stringify!(msg_seq_num)))?;
        let sending_time = sending_time_tmp.ok_or(::fix::FixError::MissingField(stringify!(sending_time)))?;
        let symbol = symbol_tmp.ok_or(::fix::FixError::MissingField(stringify!(symbol)))?;
        let md_req_id = md_req_id_tmp.ok_or(::fix::FixError::MissingField(stringify!(md_req_id)))?;
        let md_entries = md_entries_tmp.ok_or(::fix::FixError::MissingField(stringify!(md_entries)))?;
        let checksum = checksum_tmp.ok_or(::fix::FixError::MissingField(stringify!(checksum)))?;
        Ok(Self { begin_string, body_length, msg_type, sender_comp_id, target_comp_id, msg_seq_num, sending_time, symbol, md_req_id, md_entries, checksum })
    }
}
#[derive(Default, Debug)]
struct MDEntry {
    // #[fix(tag = "269", type = "u8")]
    md_entry_type: u8,
    // #[fix(tag = "270", type = "f64")]
    md_entry_px: f64,
    // #[fix(tag = "271", type = "f64")]
    md_entry_size: f64,
    // #[fix(tag = "272", type = "UTC_TIMESTAMP")]
    md_entry_date: DateTime<Utc>,
}
impl ::fix::FixDeserialize for MDEntry {
    fn from_fix_message(fix_message: &[u8]) -> Result<Self, ::fix::FixError> {
        let fix_message_str = std::str::from_utf8(fix_message)?;
        let mut fields = fix_message_str.split('|').peekable();
        Self::from_fix_message_iter(&mut fields)
    }
    fn from_fix_message_iter<'a, I>(fields: &mut std::iter::Peekable<I>) -> Result<Self, ::fix::FixError> where I: Iterator<Item=&'a str>, {
        println!("MDEntry::from_fix_message_iter() - enter" );
        use chrono::{NaiveDateTime, DateTime, Utc};
        let mut first_tag = None;
        let mut md_entry_type_tmp: Option<u8> = None;
        let mut md_entry_px_tmp: Option<f64> = None;
        let mut md_entry_size_tmp: Option<f64> = None;
        let mut md_entry_date_tmp: Option<DateTime<Utc>> = None;
        while let Some(field) = fields.peek().map(|x| *x) {
            if field.is_empty() {
                fields.next();
                continue;
            }
            let mut parts = field.splitn(2, '=');
            let tag = parts.next().unwrap();
            println!("FIELD parts: {:?}={:?}", tag, parts.next().unwrap());
            if first_tag.is_none() {
                first_tag = Some(tag);
            } else if tag == first_tag.unwrap() {
                break;
            }
            match tag {
                "269" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    md_entry_type_tmp = Some(value.parse::<u8>().map_err(|_| ::fix::FixError::InvalidValue("269".to_string()))?);
                }
                "270" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    md_entry_px_tmp = Some(value.parse::<f64>().map_err(|_| ::fix::FixError::InvalidValue("270".to_string()))?);
                }
                "271" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    md_entry_size_tmp = Some(value.parse::<f64>().map_err(|_| ::fix::FixError::InvalidValue("271".to_string()))?);
                }
                "272" => {
                    let field = fields.next().unwrap();
                    let mut parts = field.splitn(2, '=');
                    parts.next();
                    let value = parts.next().unwrap();
                    md_entry_date_tmp = Some({
                        let dt = NaiveDateTime::parse_from_str(value, "%Y%m%d-%H:%M:%S%.f")?;
                        DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)
                    });
                }
                _ => { fields.next(); }
            }
        }
        let md_entry_type = md_entry_type_tmp.ok_or(::fix::FixError::MissingField(stringify!(md_entry_type)))?;
        let md_entry_px = md_entry_px_tmp.ok_or(::fix::FixError::MissingField(stringify!(md_entry_px)))?;
        let md_entry_size = md_entry_size_tmp.ok_or(::fix::FixError::MissingField(stringify!(md_entry_size)))?;
        let md_entry_date = md_entry_date_tmp.ok_or(::fix::FixError::MissingField(stringify!(md_entry_date)))?;
        println!("MDEntry::from_fix_message_iter() - exit" );
        Ok(Self { md_entry_type, md_entry_px, md_entry_size, md_entry_date })
    }
}
fn main() {
    let fix_message = b"8=FIX.4.4|9=31226|35=W|49=DERIBITSERVER|56=gsr01|34=2|52=20240918-12:11:46.594|55=BTC-PERPETUAL|\
231=10.0|100087=26105623|100090=59806.74|746=843249447.0|100092=0.0|100093=0.00066246|262=1|268=2|\
269=0|270=59765.5|271=1.0|272=20240918-12:11:46.529|269=1|270=150000.0|271=1810000.0|272=20240918-12:11:46.529|10=163|";

    let parsed: MarketDataMessage = MarketDataMessage::from_fix_message(fix_message).unwrap();
    println!("{:#?}", parsed);
}
