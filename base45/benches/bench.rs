//! Benchmarks for Base45 encoding and decoding.

use core::hint::black_box;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};

const QUICK_BROWN_FOX_DEC: &str = "The quick brown fox jumps over the lazy dog";
const QUICK_BROWN_FOX_ENC: &str =
	"8UADZCKFEOEDJOD2KC54EM-DX.CH8FSKDQ$D.OE44E5$CS44+8DK44OEC3EFGVCD2";

fn bench_encode(c: &mut Criterion) {
	let mut group = c.benchmark_group("base45_encode");
	let input = QUICK_BROWN_FOX_DEC.as_bytes();

	let _ = group.throughput(Throughput::Bytes(input.len() as u64));
	let _ =
		group.bench_function("quick_brown_fox", |b| b.iter(|| base45::encode(black_box(input))));

	// Random data benches
	for size in [16, 256, 4096] {
		let data = vec![0u8; size];
		let _ = group.throughput(Throughput::Bytes(size as u64));
		let _ = group.bench_with_input(format!("random_{}", size), &data, |b, data| {
			b.iter(|| base45::encode(black_box(data)))
		});
	}
	group.finish();
}

fn bench_decode(c: &mut Criterion) {
	let mut group = c.benchmark_group("base45_decode");
	let input = QUICK_BROWN_FOX_ENC;

	let _ = group.throughput(Throughput::Bytes(input.len() as u64));
	let _ = group.bench_function("quick_brown_fox", |b| {
		b.iter(|| base45::decode(black_box(input)).unwrap())
	});

	group.finish();
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
