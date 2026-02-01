use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use data_encoding::Specification;

fn bench_base02(c: &mut Criterion) {
	let mut group = c.benchmark_group("base02");
	let mut spec = Specification::new();
	spec.symbols.push_str("01");
	let base = spec.encoding().unwrap();

	let input_enc = [0u8; 4096];
	let mut output_enc = [0u8; 32768];

	group.throughput(Throughput::Bytes(input_enc.len() as u64));
	group.bench_function("encode", |b| b.iter(|| base.encode_mut(&input_enc, &mut output_enc)));

	let input_dec = [b'0'; 4096];
	let mut output_dec = [0u8; 512];

	group.throughput(Throughput::Bytes(input_dec.len() as u64));
	group.bench_function("decode", |b| b.iter(|| base.decode_mut(&input_dec, &mut output_dec)));
	group.finish();
}

fn bench_base04(c: &mut Criterion) {
	let mut group = c.benchmark_group("base04");
	let mut spec = Specification::new();
	spec.symbols.push_str("0123");
	let base = spec.encoding().unwrap();

	let input_enc = [0u8; 4096];
	let mut output_enc = [0u8; 16384];

	group.throughput(Throughput::Bytes(input_enc.len() as u64));
	group.bench_function("encode", |b| b.iter(|| base.encode_mut(&input_enc, &mut output_enc)));

	let input_dec = [b'0'; 4096];
	let mut output_dec = [0u8; 1024];

	group.throughput(Throughput::Bytes(input_dec.len() as u64));
	group.bench_function("decode", |b| b.iter(|| base.decode_mut(&input_dec, &mut output_dec)));
	group.finish();
}

fn bench_base08(c: &mut Criterion) {
	let mut group = c.benchmark_group("base08");
	let mut spec = Specification::new();
	spec.symbols.push_str("01234567");
	let base = spec.encoding().unwrap();

	let input_enc = [0u8; 4096];
	let mut output_enc = [0u8; 10923];

	group.throughput(Throughput::Bytes(input_enc.len() as u64));
	group.bench_function("encode", |b| b.iter(|| base.encode_mut(&input_enc, &mut output_enc)));

	let input_dec = [b'0'; 4096];
	let mut output_dec = [0u8; 1536];

	group.throughput(Throughput::Bytes(input_dec.len() as u64));
	group.bench_function("decode", |b| b.iter(|| base.decode_mut(&input_dec, &mut output_dec)));
	group.finish();
}

fn bench_base16(c: &mut Criterion) {
	let mut group = c.benchmark_group("base16");
	let base = data_encoding::HEXLOWER;

	let input_enc = [0u8; 4096];
	let mut output_enc = [0u8; 8192];

	group.throughput(Throughput::Bytes(input_enc.len() as u64));
	group.bench_function("encode", |b| b.iter(|| base.encode_mut(&input_enc, &mut output_enc)));

	let input_dec = [b'0'; 4096];
	let mut output_dec = [0u8; 2048];

	group.throughput(Throughput::Bytes(input_dec.len() as u64));
	group.bench_function("decode", |b| b.iter(|| base.decode_mut(&input_dec, &mut output_dec)));
	group.finish();
}

fn bench_base32(c: &mut Criterion) {
	let mut group = c.benchmark_group("base32");
	let base = data_encoding::BASE32;

	let input_enc = [0u8; 4096];
	let mut output_enc = [0u8; 6560];

	group.throughput(Throughput::Bytes(input_enc.len() as u64));
	group.bench_function("encode", |b| b.iter(|| base.encode_mut(&input_enc, &mut output_enc)));

	let input_dec = [b'A'; 4096];
	let mut output_dec = [0u8; 2560];

	group.throughput(Throughput::Bytes(input_dec.len() as u64));
	group.bench_function("decode", |b| b.iter(|| base.decode_mut(&input_dec, &mut output_dec)));
	group.finish();
}

fn bench_base64(c: &mut Criterion) {
	let mut group = c.benchmark_group("base64");

	// NOPAD
	let input_enc = [0u8; 4096];
	let mut output_enc = [0u8; 5462];

	group.throughput(Throughput::Bytes(input_enc.len() as u64));
	group.bench_function("encode_nopad", |b| {
		b.iter(|| data_encoding::BASE64_NOPAD.encode_mut(&input_enc, &mut output_enc))
	});

	let input_dec = [b'A'; 4096];
	let mut output_dec = [0u8; 3072];

	group.throughput(Throughput::Bytes(input_dec.len() as u64));
	group.bench_function("decode_nopad", |b| {
		b.iter(|| data_encoding::BASE64_NOPAD.decode_mut(&input_dec, &mut output_dec))
	});

	// PAD
	let mut input_enc_pad = [b'A'; 4096];
	let mut output_enc_pad = [0u8; 5464];

	group.throughput(Throughput::Bytes(input_enc_pad.len() as u64));
	group.bench_function("encode_pad", |b| {
		b.iter(|| data_encoding::BASE64.encode_mut(&input_enc_pad, &mut output_enc_pad))
	});

	let mut input_dec_pad = [b'A'; 4096];
	for i in 0..20 {
		let x = 4096 * i / 20 / 4 * 4;
		input_dec_pad[x + 3] = b'=';
		if i % 2 == 0 {
			input_dec_pad[x + 2] = b'=';
		}
	}

	group.throughput(Throughput::Bytes(input_dec_pad.len() as u64));
	group.bench_function("decode_pad", |b| {
		b.iter(|| data_encoding::BASE64.decode_mut(&input_dec_pad, &mut output_dec).unwrap())
	});

	// WRAP
	let mut spec = data_encoding::BASE64.specification();
	spec.wrap.width = 76;
	spec.wrap.separator.push_str("\r\n");
	let base64_wrap = spec.encoding().unwrap();

	let input_enc_wrap = [0u8; 4096];
	let mut output_enc_wrap = [0u8; 5608];

	group.throughput(Throughput::Bytes(input_enc_wrap.len() as u64));
	group.bench_function("encode_wrap", |b| {
		b.iter(|| base64_wrap.encode_mut(&input_enc_wrap, &mut output_enc_wrap))
	});

	let mut input_dec_wrap = [b'A'; 4096];
	for i in 0..20 {
		let x = 4096 * i / 20 / 4 * 4;
		input_dec_wrap[x + 3] = b'\n';
	}

	group.throughput(Throughput::Bytes(input_dec_wrap.len() as u64));
	group.bench_function("decode_wrap", |b| {
		b.iter(|| base64_wrap.decode_mut(&input_dec_wrap, &mut output_dec).unwrap())
	});

	group.finish();
}

fn bench_dnscurve(c: &mut Criterion) {
	let mut group = c.benchmark_group("dnscurve");

	let input_enc = [0u8; 4096];
	let mut output_enc = [0u8; 6554];

	group.throughput(Throughput::Bytes(input_enc.len() as u64));
	group.bench_function("encode", |b| {
		b.iter(|| data_encoding::BASE32_DNSCURVE.encode_mut(&input_enc, &mut output_enc))
	});

	let input_dec = [b'0'; 4096];
	let mut output_dec = [0u8; 2560];

	group.throughput(Throughput::Bytes(input_dec.len() as u64));
	group.bench_function("decode", |b| {
		b.iter(|| data_encoding::BASE32_DNSCURVE.decode_mut(&input_dec, &mut output_dec))
	});
	group.finish();
}

criterion_group!(
	benches,
	bench_base02,
	bench_base04,
	bench_base08,
	bench_base16,
	bench_base32,
	bench_base64,
	bench_dnscurve
);
criterion_main!(benches);
