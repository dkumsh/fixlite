#![allow(dead_code)]

use chrono::{TimeZone, Timelike, Utc};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use fixlite::fix::{
    FixBuilder, HandlInst, MsgType, OrdType, Price, SecurityType, Side, TimeInForce,
};

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
            builder.begin_with(&seq, &dt, &msg_type);
            builder.field(1, &account);
            builder.field(11, &cl_ord_id);
            builder.field(21, &HandlInst::Automated);
            builder.field(55, &symbol);
            builder.field(54, &Side::Buy);
            builder.field(38, &order_qty);
            builder.field(40, &OrdType::Limit);
            builder.field(44, &price);
            builder.field(59, &TimeInForce::Day);
            builder.field(60, &dt);
            builder.field(100, "XNAS");
            builder.field(15, "USD");
            builder.field(167, &SecurityType::ForeignExchangeContract);
            builder.field(114, &true);
            builder.field(58, &text);
            // Parties repeating group (453)
            builder.field(453, &2u32);
            builder.field(448, "PARTY1");
            builder.field(447, "D");
            builder.field(452, &1u32);
            builder.field(448, "PARTY2");
            builder.field(447, "D");
            builder.field(452, &12u32);
            let msg = builder.finish();
            black_box(msg.len());
        });
    });
}

criterion_group!(benches, bench_build_fix);
criterion_main!(benches);
