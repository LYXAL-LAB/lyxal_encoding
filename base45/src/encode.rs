use crate::alphabet::{self, SIZE, SIZE_SIZE};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{string::String, vec};
#[cfg(feature = "std")]
use std::{string::String, vec};

use core::fmt;

/// Errors that can occur during encoding.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EncodeError {
	/// The provided output buffer is too small to hold the result.
	BufferTooSmall,
}

impl fmt::Display for EncodeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			EncodeError::BufferTooSmall => f.write_str("Output buffer is too small"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {}

#[inline(always)]
fn ae(b: u8) -> u8 {
	match alphabet::encode(b) {
		Some(ch) => ch,
		#[cfg(feature = "unsafe")]
		// SAFETY: encode for this is highly unlikely to ever reach this point.
		None => unsafe { core::hint::unreachable_unchecked() },
		#[cfg(not(feature = "unsafe"))]
		None => unreachable!(),
	}
}

/// Encodes a byte slice into an existing buffer.
/// Returns the number of bytes written to the output.
///
/// # Errors
/// Returns `EncodeError::BufferTooSmall` if the output buffer cannot hold the encoded string.
pub fn encode_to_buffer(input: &[u8], output: &mut [u8]) -> Result<usize, EncodeError> {
	let required_len = input.len() + input.len().div_ceil(2);
	if output.len() < required_len {
		return Err(EncodeError::BufferTooSmall);
	}

	let (chunks, remainder) = input.as_chunks::<2>();
	let mut out_idx = 0;

	for chunk in chunks {
		let v = (u32::from(chunk[0]) << 8) | u32::from(chunk[1]);
		let e = v / SIZE_SIZE;
		let rest = v % SIZE_SIZE;
		let d = rest / SIZE;
		let c = rest % SIZE;

		output[out_idx] = ae(c as u8);
		output[out_idx + 1] = ae(d as u8);
		output[out_idx + 2] = ae(e as u8);
		out_idx += 3;
	}

	if let &[last] = remainder {
		let v = u32::from(last);
		let d = v / SIZE;
		let c = v % SIZE;

		output[out_idx] = ae(c as u8);
		output[out_idx + 1] = ae(d as u8);
		out_idx += 2;
	}

	Ok(out_idx)
}

/// Encode a byte slice into a `String`.
///
/// ```rust
/// use base45;
/// let encoded = base45::encode("Hello!!");
/// assert_eq!(encoded, "%69 VD92EX0");
/// ```
#[cfg(feature = "alloc")]
pub fn encode(input: impl AsRef<[u8]>) -> String {
	let input = input.as_ref();
	let len = input.len() + input.len().div_ceil(2);
	let mut buffer = vec![0u8; len];
	// Safety: we calculated the exact size needed
	let _ = encode_to_buffer(input, &mut buffer);

	#[cfg(feature = "unsafe")]
	// SAFETY: we control all bytes that enter this vector and they are from the base45 alphabet.
	unsafe {
		String::from_utf8_unchecked(buffer)
	}
	#[cfg(not(feature = "unsafe"))]
	String::from_utf8(buffer).expect("All bytes encoded must be ascii")
}
