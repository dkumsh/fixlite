//! Compares the bundled custom `f64` encoder against the stdlib `Display` path
//! (`write!(&mut Vec<u8>, "{}", value)` via `std::io::Write`).
//!
//! The two produce *different bytes* for some inputs — the custom encoder is
//! fixed-precision 15-significant-digit with banker's rounding, while stdlib
//! emits the shortest decimal that round-trips back to the same `f64`. This
//! bench measures cost, not output equivalence.

#![allow(dead_code)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use fixlite::builder::TryFixValue;
use std::io::Write;

fn inputs() -> Vec<f64> {
    vec![
        // Exactly-representable prices and quantities — the common FIX case.
        0.0,
        1.0,
        42.0,
        1000.0,
        100_000.0,
        0.5,
        0.25,
        0.125,
        1.5,
        1.25,
        150.25,
        0.0001,
        9999.9999,
        // Computed / "dirty" arithmetic results.
        0.1 + 0.2,
        1.0 / 3.0,
        std::f64::consts::PI,
        std::f64::consts::E,
        // Edge magnitudes.
        1e-10,
        1e10,
        1e15,
        1e-15,
        // Negatives.
        -0.5,
        -1.25,
        -1000.0,
    ]
}

fn bench_custom(c: &mut Criterion) {
    let xs = inputs();
    c.bench_function("f64 encode: custom (current)", |b| {
        let mut buf = Vec::with_capacity(64);
        b.iter(|| {
            for &v in &xs {
                buf.clear();
                <f64 as TryFixValue>::try_encode(&v, &mut buf).unwrap();
                black_box(&buf);
            }
        });
    });
}

fn bench_stdlib(c: &mut Criterion) {
    let xs = inputs();
    c.bench_function("f64 encode: stdlib write!", |b| {
        let mut buf = Vec::with_capacity(64);
        b.iter(|| {
            for &v in &xs {
                buf.clear();
                let _ = write!(&mut buf, "{}", v);
                black_box(&buf);
            }
        });
    });
}

criterion_group!(benches, bench_custom, bench_stdlib);
criterion_main!(benches);
