use crate::alphabet::{self, SIZE, SIZE_SIZE};

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use core::fmt::{Display, Formatter};

/// Errors that can occur during decoding.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum DecodeError {
	/// The input string has an invalid length (e.g., remainder of 1 character).
	InvalidLength,
	/// The input string contains a character not in the Base45 alphabet.
	InvalidCharacter,
	/// The decoded value exceeds the allowed range (triplet > 65535 or pair > 255).
	OutOfRange,
	/// The provided output buffer is too small to hold the result.
	BufferTooSmall,
}

impl Display for DecodeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		match self {
			DecodeError::InvalidLength => f.write_str("Invalid base45 string length"),
			DecodeError::InvalidCharacter => f.write_str("Invalid character in base45 string"),
			DecodeError::OutOfRange => f.write_str("Decoded value out of range"),
			DecodeError::BufferTooSmall => f.write_str("Output buffer is too small"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

/// Decodes a base45 encoded string into an existing buffer.
/// Returns the number of bytes written to the output.
///
/// # Errors
/// - Returns `DecodeError::InvalidLength` if the input length is not a multiple of 3 or has a remainder of 1.
/// - Returns `DecodeError::InvalidCharacter` if a character is not in the Base45 alphabet.
/// - Returns `DecodeError::OutOfRange` if a sequence decodes to a value exceeding the allowed range.
/// - Returns `DecodeError::BufferTooSmall` if the output buffer is too small.
pub fn decode_to_buffer(input: &[u8], output: &mut [u8]) -> Result<usize, DecodeError> {
	if input.is_empty() {
		return Ok(0);
	}

	if input.len() % 3 == 1 {
		return Err(DecodeError::InvalidLength);
	}

	let required_len = (input.len() / 3) * 2
		+ if input.len() % 3 == 2 {
			1
		} else {
			0
		};
	if output.len() < required_len {
		return Err(DecodeError::BufferTooSmall);
	}

	let mut out_idx = 0;
	let (chunks, remainder) = input.as_chunks::<3>();

	for chunk in chunks {
		let c = alphabet::decode(chunk[0]).ok_or(DecodeError::InvalidCharacter)?;
		let d = alphabet::decode(chunk[1]).ok_or(DecodeError::InvalidCharacter)?;
		let e = alphabet::decode(chunk[2]).ok_or(DecodeError::InvalidCharacter)?;

		let v = u32::from(c) + u32::from(d) * SIZE + u32::from(e) * SIZE_SIZE;
		if v > u32::from(u16::MAX) {
			return Err(DecodeError::OutOfRange);
		}

		output[out_idx] = (v >> 8) as u8;
		output[out_idx + 1] = (v & 0xFF) as u8;
		out_idx += 2;
	}

	if let &[first, second] = remainder {
		let c = alphabet::decode(first).ok_or(DecodeError::InvalidCharacter)?;
		let d = alphabet::decode(second).ok_or(DecodeError::InvalidCharacter)?;
		let v = u32::from(c) + u32::from(d) * SIZE;

		if v > 255 {
			return Err(DecodeError::OutOfRange);
		}
		output[out_idx] = v as u8;
		out_idx += 1;
	}

	Ok(out_idx)
}

/// Decodes a base45 encoded string into a `Vec<u8>`.
///
/// # Errors
/// Returns a `DecodeError` if the input is not a valid base45 string.
pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
	let input = input.as_ref();
	if input.is_empty() {
		return Ok(Vec::new());
	}

	if input.len() % 3 == 1 {
		return Err(DecodeError::InvalidLength);
	}

	let required_len = (input.len() / 3) * 2
		+ if input.len() % 3 == 2 {
			1
		} else {
			0
		};
	let mut output = vec![0u8; required_len];
	let len = decode_to_buffer(input, &mut output)?;
	output.truncate(len);
	Ok(output)
}
