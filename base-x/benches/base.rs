use criterion::{black_box, criterion_group, criterion_main, Criterion};
use base_x::{decode, decode_to_buffer, encode, encode_to_buffer};

fn random_input(size: usize) -> Vec<u8> {
    let mut v = vec![0; size];
    for x in v.iter_mut() {
        *x = rand::random()
    }
    v
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Encode");
    let input = random_input(100);
    const ALPH_58: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    group.bench_function("Base58_Alloc", |b| {
        b.iter(|| encode(black_box(ALPH_58), black_box(&input)).unwrap())
    });

    group.bench_function("Base58_NoAlloc", |b| {
        let mut out = [0u8; 1024];
        b.iter(|| encode_to_buffer(black_box(ALPH_58.as_bytes()), black_box(&input), black_box(&mut out)).unwrap())
    });

    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decode");
    let input = random_input(100);
    const ALPH_58: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let encoded = encode(ALPH_58, &input).unwrap();

    group.bench_function("Base58_Alloc", |b| {
        b.iter(|| decode(black_box(ALPH_58), black_box(&encoded)).unwrap())
    });

    group.bench_function("Base58_NoAlloc", |b| {
        let mut out = [0u8; 128];
        b.iter(|| decode_to_buffer(black_box(ALPH_58.as_bytes()), black_box(&encoded), black_box(&mut out)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
