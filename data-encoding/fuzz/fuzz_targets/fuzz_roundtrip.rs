#![no_main]
use data_encoding::{BASE58, BASE64};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
	// 1. Test Base64 (Standard power-of-two path)
	{
		let encoded = BASE64.encode(data);
		let decoded = BASE64.decode(encoded.as_bytes()).expect("Base64 round-trip failed");
		assert_eq!(data, decoded.as_slice());

		let enc_len = BASE64.encode_len(data.len()).unwrap();
		if enc_len < 4096 {
			let mut enc_buf = [0u8; 4096];
			let mut dec_buf = [0u8; 4096];

			let written =
				BASE64.encode_mut(data, &mut enc_buf[..enc_len]).expect("Base64 encode_mut failed");
			assert_eq!(written, enc_len);

			let dec_len = BASE64.decode_len(enc_len).unwrap();
			let actual_len = BASE64
				.decode_mut(&enc_buf[..written], &mut dec_buf[..dec_len])
				.expect("Base64 decode_mut round-trip failed");

			assert_eq!(data, &dec_buf[..actual_len]);
		}
	}

	// 2. Test Base58 (Arithmetic non-power-of-two path)
	{
		// Skip extremely large inputs for arithmetic fuzzing to avoid timeout
		if data.len() < 1024 {
			let encoded = BASE58.encode(data);
			let decoded = BASE58.decode(encoded.as_bytes()).expect("Base58 round-trip failed");
			assert_eq!(data, decoded.as_slice());

			let enc_len_upper = BASE58.encode_len(data.len()).unwrap();
			if enc_len_upper < 4096 {
				let mut enc_buf = [0u8; 4096];
				let mut dec_buf = [0u8; 4096];

				let written = BASE58
					.encode_mut(data, &mut enc_buf[..enc_len_upper])
					.expect("Base58 encode_mut failed");

				let dec_len_upper = BASE58.decode_len(written).unwrap();
				let actual_len = BASE58
					.decode_mut(&enc_buf[..written], &mut dec_buf[..dec_len_upper])
					.expect("Base58 decode_mut round-trip failed");

				assert_eq!(data, &dec_buf[..actual_len]);
			}
		}
	}
});
