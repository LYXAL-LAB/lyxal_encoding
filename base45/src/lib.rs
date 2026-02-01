#![cfg_attr(not(feature = "std"), no_std)]
//! High-performance, Production-Grade Base45 encoder/decoder.
//!
//! This implementation is fully compatible with [RFC 9285](https://datatracker.ietf.org/doc/html/rfc9285).
//! It is designed for robustness, security, and optional zero-allocation operations.

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

/// The Base45 alphabet and its associated mapping functions.
pub mod alphabet;
mod decode;
mod encode;

pub use decode::{DecodeError, decode_to_buffer};
pub use encode::{EncodeError, encode_to_buffer};

#[cfg(feature = "alloc")]
pub use decode::decode;
#[cfg(feature = "alloc")]
pub use encode::encode;

#[cfg(test)]
mod tests;
