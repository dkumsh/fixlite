#![allow(dead_code)]

use criterion::{Criterion, criterion_group, criterion_main};
use fixlite::FixDeserialize;
use fixlite::fix_tag_registry;

// Re‑declare the registry and message type for the benchmark
fix_tag_registry! {
    MyRegistry {
        38 => [u64],
        10 => [u32],
        9 => [u32],
        44 => [f64],
    }
}

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

/// Benchmark parsing a batch of FIX messages repeatedly.
fn bench_parse_fix(c: &mut Criterion) {
    // Prepare one FIX message in SOH‑separated form.
    let raw = b"8=FIX.4.4|9=112|35=D|49=SENDER|56=TARGET|11=ABC123|55=EUR/USD|54=1|38=1000|44=1.2345|52=20250907-12:34:56.789|10=128|";
    let message = raw
        .iter()
        .map(|&b| if b == b'|' { 0x01 } else { b })
        .collect::<Vec<u8>>();

    // Create a buffer with N copies of the message concatenated together.
    const N: usize = 100;
    let mut buf = Vec::with_capacity(message.len() * N);
    for _ in 0..N {
        buf.extend_from_slice(&message);
    }
    let len = message.len();

    c.bench_function("parse MarketDataMessage N times", |b| {
        b.iter(|| {
            // Walk through the concatenated buffer and parse each message.
            let mut offset = 0;
            for _ in 0..N {
                let slice = &buf[offset..offset + len];
                let m = MarketDataMessage::from_fix(slice).unwrap();
                // Check one field to ensure the optimizer doesn’t eliminate parsing.
                assert_eq!(m.body_length, Some(112));
                offset += len;
            }
        });
    });
}

criterion_group!(benches, bench_parse_fix);
criterion_main!(benches);
