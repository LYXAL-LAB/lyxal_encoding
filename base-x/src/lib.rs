//! # base_x
//!
//! Encode and decode any base alphabet.
//!
//! ## Installation
//!
//! Add this to `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! base-x = "0.2.0"
//! ```
//!
//! ## Usage
//!
//! ```rust
//! extern crate base_x;
//!
//! fn main() {
//!   let decoded = base_x::decode("01", "11111111000000001111111100000000").unwrap();
//!   let encoded = base_x::encode("01", &decoded).unwrap();
//!  assert_eq!(encoded, "11111111000000001111111100000000");
//! }
//! ```

#![cfg_attr(not(feature = "alloc"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(feature = "alloc")]
mod alloc_types {
    #[cfg(feature = "std")]
    pub use std::{string::String, vec::Vec};
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::{string::String, vec::Vec};
}

pub mod alphabet;
mod bigint;
pub mod decoder;
pub mod encoder;

#[cfg(feature = "alloc")]
pub(crate) use crate::alloc_types::{String, Vec};

pub use crate::alphabet::Alphabet;

use core::fmt;

#[derive(Debug)]
pub struct DecodeError;

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to decode the given data")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

#[derive(Debug)]
pub enum EncodeError {
    BufferTooSmall,
    InputTooLarge,
    InvalidAlphabet,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncodeError::BufferTooSmall => write!(f, "Output buffer is too small"),
            EncodeError::InputTooLarge => write!(f, "Input data is too large for the fixed buffer"),
            EncodeError::InvalidAlphabet => write!(f, "Provided alphabet is invalid for this operation"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {}

/// Encode an input vector using the given alphabet into a provided buffer.
pub fn encode_to_buffer<A: Alphabet>(
    alphabet: A,
    input: &[u8],
    output: &mut [u8],
) -> Result<usize, EncodeError> {
    alphabet.encode_to_buffer(input, output)
}

/// Decode an input string using the given alphabet into a provided buffer.
pub fn decode_to_buffer<A: Alphabet>(
    alphabet: A,
    input: &str,
    output: &mut [u8],
) -> Result<usize, DecodeError> {
    alphabet.decode_to_buffer(input, output)
}

/// Encode an input vector using the given alphabet.
#[cfg(feature = "alloc")]
pub fn encode<A: Alphabet>(alphabet: A, input: &[u8]) -> Result<String, EncodeError> {
    alphabet.encode(input)
}

/// Decode an input vector using the given alphabet.
#[cfg(feature = "alloc")]
pub fn decode<A: Alphabet>(alphabet: A, input: &str) -> Result<Vec<u8>, DecodeError> {
    alphabet.decode(input)
}

#[cfg(all(test, feature = "alloc"))]
mod test {
    use super::decode;
    use super::encode;
    extern crate json;
    use self::json::parse;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn works() {
        let mut file = File::open("./fixtures/fixtures.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let json = parse(&data).unwrap();
        let alphabets = &json["alphabets"];

        for value in json["valid"].members() {
            let alphabet_name = value["alphabet"].as_str().unwrap();
            let input = value["string"].as_str().unwrap();
            let alphabet = alphabets[alphabet_name].as_str().unwrap();

            // Alphabet works as unicode
            let decoded = decode(alphabet, input).unwrap();
            let encoded = encode(alphabet, &decoded).unwrap();
            assert_eq!(encoded, input);

            // Alphabet works as ASCII
            let decoded = decode(alphabet.as_bytes(), input).unwrap();
            let encoded = encode(alphabet.as_bytes(), &decoded).unwrap();
            assert_eq!(encoded, input);
        }
    }

    #[test]
    fn is_unicode_sound() {
        // binary, kinda...
        let alphabet = "😐😀";

        let encoded = encode(alphabet, &[0xff, 0x00, 0xff, 0x00]).unwrap();
        let decoded = decode(alphabet, &encoded).unwrap();

        assert_eq!(
            encoded,
            "😀😀😀😀😀😀😀😀😐😐😐😐😐😐😐😐😀😀😀😀😀😀😀😀😐😐😐😐😐😐😐😐"
        );
        assert_eq!(decoded, &[0xff, 0x00, 0xff, 0x00]);
    }

    #[test]
    fn compare_no_alloc_to_standard() {
        use crate::decode_to_buffer;
        use crate::encode_to_buffer;

        let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        let input = [0xde, 0xad, 0xbe, 0xef, 0x00, 0x01];

        // 1. Test Encode
        let std_encoded = encode(alphabet, &input).unwrap();
        let mut buf_encoded = [0u8; 128];
        let len_encoded = encode_to_buffer(alphabet, &input, &mut buf_encoded).unwrap();
        let no_alloc_encoded = std::str::from_utf8(&buf_encoded[..len_encoded]).unwrap();

        assert_eq!(
            std_encoded, no_alloc_encoded,
            "Encode produce different results!"
        );

        // 2. Test Decode
        let std_decoded = decode(alphabet, &std_encoded).unwrap();
        let mut buf_decoded = [0u8; 128];
        let len_decoded = decode_to_buffer(alphabet, &std_encoded, &mut buf_decoded).unwrap();
        let no_alloc_decoded = &buf_decoded[..len_decoded];

        assert_eq!(
            std_decoded, no_alloc_decoded,
            "Decode produce different results!"
        );
    }

    #[cfg(feature = "alloc")]
    mod property_tests {
        use super::super::{decode, encode, decode_to_buffer, encode_to_buffer};
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn roundtrip_standard(
                ref input in any::<Vec<u8>>(),
                ref alphabet_raw in "[a-zA-Z0-9]{2,64}"
            ) {
                // Ensure alphabet has unique characters
                let mut chars: Vec<char> = alphabet_raw.chars().collect();
                chars.sort_unstable();
                chars.dedup();
                if chars.len() < 2 { return Ok(()); }
                let alphabet: String = chars.into_iter().collect();

                // 1. Standard Roundtrip
                if let Ok(encoded) = encode(alphabet.as_str(), input.as_slice()) {
                    let decoded = decode(alphabet.as_str(), &encoded).expect("Decode failed for valid encoding");
                    prop_assert_eq!(input, &decoded, "Standard roundtrip failed");

                    // 2. No-Alloc Roundtrip comparison
                    let mut enc_buf = vec![0u8; encoded.len() + 10];
                    let enc_len = encode_to_buffer(alphabet.as_str(), input.as_slice(), &mut enc_buf).expect("encode_to_buffer failed");
                    prop_assert_eq!(encoded.as_bytes(), &enc_buf[..enc_len], "Buffer encoding mismatch");

                    let mut dec_buf = vec![0u8; input.len() + 10];
                    let dec_len = decode_to_buffer(alphabet.as_str(), &encoded, &mut dec_buf).expect("decode_to_buffer failed");
                    prop_assert_eq!(input, &dec_buf[..dec_len], "Buffer decoding mismatch");
                }
            }
        }
    }
}
