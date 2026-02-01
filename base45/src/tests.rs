use crate::*;
#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

#[cfg(test)]
mod property_tests {
	use super::*;
	use proptest::prelude::*;

	proptest! {
		#![proptest_config(ProptestConfig::with_cases(10000))]
		#[test]
		fn roundtrip(data in any::<Vec<u8>>()) {
			// 1. Test Alloc path
			let encoded = encode(&data);
			let decoded = decode(&encoded).expect("Alloc decode failed");
			prop_assert_eq!(&data, &decoded);

			// 2. Test No-Alloc path
			let required_enc_len = data.len() + data.len().div_ceil(2);
			let mut enc_buf = vec![0u8; required_enc_len];
			let enc_len = encode_to_buffer(&data, &mut enc_buf).expect("encode_to_buffer failed");
			prop_assert_eq!(enc_len, required_enc_len);
			prop_assert_eq!(encoded.as_bytes(), &enc_buf);

			let required_dec_len = (enc_len / 3) * 2 + if enc_len % 3 == 2 { 1 } else { 0 };
			let mut dec_buf = vec![0u8; required_dec_len];
			let dec_len = decode_to_buffer(&enc_buf, &mut dec_buf).expect("decode_to_buffer failed");
			prop_assert_eq!(&data, &dec_buf[..dec_len]);
		}
	}
}

#[test]
fn encode_ab() {
	assert_eq!(encode("AB"), "BB8")
}

#[test]
fn decode_fail() {
	assert!(decode(":::").is_err());
}

#[test]
fn decode_fail_out_of_range() {
	assert!(decode(":::").is_err());
}

#[test]
fn encode_hello() {
	assert_eq!(encode("Hello!!"), "%69 VD92EX0")
}

#[test]
fn encode_base45() {
	assert_eq!(encode("base-45"), "UJCLQE7W581")
}

#[test]
fn encode_long_string() {
	assert_eq!(
		encode("The quick brown fox jumps over the lazy dog"),
		"8UADZCKFEOEDJOD2KC54EM-DX.CH8FSKDQ$D.OE44E5$CS44+8DK44OEC3EFGVCD2",
	)
}

#[test]
fn encode_unicode() {
	assert_eq!(encode("foo © bar 𝌆 baz"), "X.C82EIROA44GECH74C-J1/GUJCW2")
}

#[test]
fn encode_emoji() {
	assert_eq!(encode("I ❤️  Rust"), "0B98TSD%K.ENY244JA QE")
}

#[test]
fn encode_ietf() {
	assert_eq!(encode("ietf!"), "QED8WEX0")
}

#[test]
fn decode_ab() {
	assert_eq!(String::from_utf8(decode("BB8").unwrap()).unwrap(), "AB")
}

#[test]
fn decode_hello() {
	assert_eq!(String::from_utf8(decode("%69 VD92EX0").unwrap()).unwrap(), "Hello!!")
}

#[test]
fn decode_base45() {
	assert_eq!(String::from_utf8(decode("UJCLQE7W581").unwrap()).unwrap(), "base-45")
}

#[test]
fn decode_ietf() {
	assert_eq!(String::from_utf8(decode("QED8WEX0").unwrap()).unwrap(), "ietf!")
}

const QUICK_BROWN_FOX_ENC: &str =
	"8UADZCKFEOEDJOD2KC54EM-DX.CH8FSKDQ$D.OE44E5$CS44+8DK44OEC3EFGVCD2";
const QUICK_BROWN_FOX_DEC: &str = "The quick brown fox jumps over the lazy dog";
#[test]
fn decode_long_string() {
	assert_eq!(
		String::from_utf8(decode(QUICK_BROWN_FOX_ENC).unwrap()).unwrap(),
		QUICK_BROWN_FOX_DEC,
	)
}

#[test]
fn encode_hello_from_buffer() {
	assert_eq!(encode(vec![72, 101, 108, 108, 111, 33, 33]), "%69 VD92EX0")
}

#[test]
fn encode_full_bytes() {
	let s = encode(b"\xff\xff\xff\xff\xff\xff");
	assert_eq!(s, "FGWFGWFGW");
	let s = encode(b"\xff\xff\xff\xff\xff\xff\xff");
	assert_eq!(s, "FGWFGWFGWU5");
	let s = encode(b"\xff\xff\xff\xff\xff\xff\xff\xff");
	assert_eq!(s, "FGWFGWFGWFGW");
}
#[test]
fn decode_full_bytes() {
	let s = decode("FGWFGWFGW").unwrap();
	assert_eq!(s, b"\xff\xff\xff\xff\xff\xff");
	let s = decode("FGWFGWFGWU5").unwrap();
	assert_eq!(s, b"\xff\xff\xff\xff\xff\xff\xff");
	let s = decode("FGWFGWFGWFGW").unwrap();
	assert_eq!(s, b"\xff\xff\xff\xff\xff\xff\xff\xff");
}
