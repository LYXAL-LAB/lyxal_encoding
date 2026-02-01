use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lyxal_base256emoji::{encode, decode};

fn bench_encode(c: &mut Criterion) {
    let data = vec![0u8; 1024]; // 1KB of data
    c.bench_function("emoji_encode_1kb", |b| {
        b.iter(|| encode(black_box(&data)))
    });
}

fn bench_decode(c: &mut Criterion) {
    let data = vec![0u8; 1024];
    let encoded = encode(&data);
    c.bench_function("emoji_decode_1kb", |b| {
        b.iter(|| decode(black_box(&encoded)))
    });
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
