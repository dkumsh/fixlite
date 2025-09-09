// src/main.rs

use fixlite::fix_tag_registry;
use fixlite::FixDeserialize;
use fixlite_derive::FixDeserialize;

fix_tag_registry! {
    MyRegistry {
        38   => [u64],
        10   => [u32],
        9   => [u32],
        44   => [f64],
    }
}

#[allow(dead_code)]
#[derive(FixDeserialize, Debug)]
#[fix_registry(MyRegistry)]
struct MarketDataMessage<'a> {
    #[fix(tag = 8)]
    pub begin_string: Option<&'a str>,
    #[fix(tag = 9)]
    pub body_length: Option<u32>,
    #[fix(tag = 35)]
    pub msg_type: Option<&'a str>,
    #[fix(tag = 49)]
    pub sender_comp_id: Option<&'a str>,
    #[fix(tag = 56)]
    pub target_comp_id: Option<&'a str>,
    #[fix(tag = 11)]
    pub cl_ord_id: Option<&'a str>,
    #[fix(tag = 55)]
    pub symbol: Option<&'a str>,
    #[fix(tag = 54)]
    pub side: Option<&'a str>,
    #[fix(tag = 38)]
    pub order_qty: Option<u64>,
    #[fix(tag = 44)]
    pub price: Option<&'a str>,
    #[fix(tag = 52)]
    pub sending_time: Option<&'a str>,
    #[fix(tag = 10)]
    pub checksum: Option<u32>,
}

fn main() {
    let s = b"8=FIX.4.4|9=112|35=D|49=SENDER|56=TARGET|11=ABC123|55=EUR/USD|54=1|38=1000|44=1.2345|52=20250907-12:34:56.789|10=128|";
    let s = s
        .iter()
        .map(|&b| if b == b'|' { 0x01 } else { b })
        .collect::<Vec<u8>>();

    let mut buf = Vec::with_capacity(100000);
    const M: usize = 1_000_0;
    const N: usize = 100;
    for _ in 0..N {
        buf.extend_from_slice(&s);
    }

    let start = std::time::Instant::now();
    let len = s.len();
    for _ in 0..M {
        let mut start = 0;
        let mut end = len;
        for _ in 0..N {
            let m: MarketDataMessage = MarketDataMessage::from_fix(&buf[start..end]).unwrap();
            if m.body_length != Some(112) {
                panic!("bad");
            }
            start = end;
            end += len;
        }
    }
    let elapsed = start.elapsed();
    println!("elapsed = {:?}", elapsed);
}
