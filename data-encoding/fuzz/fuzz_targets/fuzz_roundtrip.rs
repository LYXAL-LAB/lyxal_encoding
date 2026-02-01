#![no_main]
use data_encoding::BASE64;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
	// 1. Test Alloc path
	let encoded = BASE64.encode(data);
	let decoded = BASE64.decode(encoded.as_bytes()).expect("Round-trip failed");
	assert_eq!(data, decoded);

	// 2. Test No-Alloc path
	// Calculate required output size
	let enc_len = BASE64.encode_len(data.len());
	// Use a large enough buffer on stack, or skip if too large for stack safety in fuzzing
	if enc_len < 4096 {
		let mut enc_buf = [0u8; 4096];
		let mut dec_buf = [0u8; 4096];

		// Encode
		BASE64.encode_mut(data, &mut enc_buf[..enc_len]);

		// Decode
		let dec_len = BASE64.decode_len(enc_len).unwrap();
		let actual_len = BASE64
			.decode_mut(&enc_buf[..enc_len], &mut dec_buf[..dec_len])
			.expect("Round-trip failed in no_alloc");

		assert_eq!(data, &dec_buf[..actual_len]);
	}
});
