#![no_main]
use base45;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
	// 1. Test Alloc path
	let encoded = base45::encode(data);
	let decoded = base45::decode(&encoded).expect("Round-trip failed in alloc path");
	assert_eq!(data, decoded);

	// 2. Test No-Alloc path
	// Required length for base45 encoding: n + ceil(n/2)
	let required_enc_len = data.len() + data.len().div_ceil(2);

	// Limit size for fuzzing to avoid excessive stack usage
	if required_enc_len < 4096 {
		let mut enc_buf = [0u8; 4096];
		let mut dec_buf = [0u8; 4096];

		// Encode to buffer
		let enc_len = base45::encode_to_buffer(data, &mut enc_buf[..required_enc_len])
			.expect("encode_to_buffer failed");

		assert_eq!(enc_len, required_enc_len);
		assert_eq!(encoded.as_bytes(), &enc_buf[..enc_len]);

		// Decode to buffer
		let dec_len = base45::decode_to_buffer(&enc_buf[..enc_len], &mut dec_buf)
			.expect("decode_to_buffer failed");

		assert_eq!(data, &dec_buf[..dec_len]);
	}
});
