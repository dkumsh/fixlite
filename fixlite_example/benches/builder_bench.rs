#![allow(dead_code)]

use chrono::{TimeZone, Timelike, Utc};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use fixlite::FixBuilder;
use fixlite::enums::{HandlInst, MsgType, OrdType, SecurityType, Side, TimeInForce};
use fixlite::fix::Price;

fn bench_build_fix(c: &mut Criterion) {
    let mut builder = FixBuilder::new("FIX.4.2", "SENDER", "TARGET");

    let dt = Utc
        .with_ymd_and_hms(2025, 1, 2, 3, 4, 5)
        .unwrap()
        .with_nanosecond(678_000_000)
        .unwrap();
    let seq = 42u64;
    let msg_type = MsgType::NewOrderSingle;

    let cl_ord_id = String::from("ABC123");
    let symbol = String::from("EUR/USD");
    let account = String::from("ACC-42");
    let text = String::from("bench order");
    let order_qty = 1_000.0f64;
    let price: Price = "123.45".parse().unwrap();

    c.bench_function("build_fix NewOrderSingle", |b| {
        b.iter(|| {
            let msg = builder
                .begin_with(&seq, &dt, &msg_type)
                .field_ref(1, &account)
                .field_ref(11, &cl_ord_id)
                .field(21, HandlInst::Automated)
                .field_ref(55, &symbol)
                .field(54, Side::Buy)
                .field(38, order_qty)
                .field(40, OrdType::Limit)
                .field(44, price)
                .field(59, TimeInForce::Day)
                .field(60, dt)
                .str(100, "XNAS")
                .str(15, "USD")
                .field(167, SecurityType::ForeignExchangeContract)
                .field(114, true)
                .field_ref(58, &text)
                // Parties repeating group (453
                .field(453, 2)
                .str(448, "PARTY1")
                .str(447, "D")
                .field(452, 1)
                .str(448, "PARTY2")
                .str(447, "D")
                .field(452, 12)
                .finish();
            black_box(msg.len());
        });
    });
}

criterion_group!(benches, bench_build_fix);
criterion_main!(benches);
