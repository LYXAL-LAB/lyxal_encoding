//! # multibase
//!
//! Implementation of [multibase](https://github.com/multiformats/multibase) in Rust.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

mod base;
mod encoding;
mod error;
mod impls;

pub use self::base::Base;
pub use self::error::{Error, Result};

/// Decode the base string.
///
/// # Examples
///
/// ```
/// use multibase::{Base, decode};
///
/// assert_eq!(
///     decode("zCn8eVZg").unwrap(),
///     (Base::Base58Btc, b"hello".to_vec())
/// );
/// ```
pub fn decode<T: AsRef<str>>(input: T) -> Result<(Base, Vec<u8>)> {
    let input = input.as_ref();
    let code = input.chars().next().ok_or(Error::InvalidBaseString)?;
    let base = Base::from_code(code)?;
    let decoded = base.decode(&input[code.len_utf8()..])?;
    Ok((base, decoded))
}

/// Encode with the given byte slice to base string.
///
/// # Examples
///
/// ```
/// use multibase::{Base, encode};
///
/// assert_eq!(encode(Base::Base58Btc, b"hello").unwrap(), "zCn8eVZg");
/// ```
pub fn encode<T: AsRef<[u8]>>(base: Base, input: T) -> Result<String> {
    let input = input.as_ref();
    let mut encoded = base.encode(input.as_ref())?;
    encoded.insert(0, base.code());
    Ok(encoded)
}
#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    fn debug_emoji() {
        let input = b"y";
        let encoded = encode(Base::Base256Emoji, input).unwrap();
        println!("Encoded 'y' ({}): {:?}", encoded, encoded.as_bytes());
        let (base, decoded) = decode(&encoded).unwrap();
        assert_eq!(base, Base::Base256Emoji);
        assert_eq!(decoded, input.to_vec());
    }
}
