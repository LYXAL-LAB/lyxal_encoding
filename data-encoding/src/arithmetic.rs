//! Arithmetic encoding and decoding for non-power-of-two bases.
//!
//! This module provides encoding and decoding functions for bases that
//! are not powers of two, such as Base58 or Base62, using big integer
//! arithmetic operations.

use crate::DecodeError;
use crate::DecodeKind;
use crate::EncodeError;
use crate::EncodeKind;
use crate::bigint::BigUintView;

/// Maximum buffer size for big integer operations (128 chunks = 512 bytes)
const MAX_BIGINT_BUFFER: usize = 128;

/// Encode input bytes using arithmetic encoding with the given alphabet.
///
/// This function writes the encoded result to the provided output buffer.
/// Returns the number of bytes written to the output buffer, or an error
/// if the buffer is too small or the input is too large.
pub(crate) fn encode_to_buffer(
	alphabet: &[u8],
	input: &[u8],
	output: &mut [u8],
) -> Result<usize, EncodeError> {
	if input.is_empty() {
		return Ok(0);
	}

	let base = alphabet.len() as u32;

	// Stack buffer for BigUint computation (512 bytes capacity)
	let mut chunks = [0u32; MAX_BIGINT_BUFFER];
	let mut big = BigUintView::new(&mut chunks);

	if !big.load_be_bytes(input) {
		return Err(EncodeError {
			kind: EncodeKind::Overflow,
		});
	}

	let mut out_idx = 0;
	let big_pow = 32 / (32 - base.leading_zeros());
	let big_base = base.pow(big_pow);

	'fast: loop {
		let mut big_rem = big.div_mod(big_base);

		if big.is_zero() {
			loop {
				let (result, remainder) = (big_rem / base, big_rem % base);
				if out_idx >= output.len() {
					return Err(EncodeError {
						kind: EncodeKind::BufferTooSmall,
					});
				}
				output[out_idx] = alphabet[remainder as usize];
				out_idx += 1;
				big_rem = result;

				if big_rem == 0 {
					break 'fast;
				}
			}
		} else {
			for _ in 0..big_pow {
				let (result, remainder) = (big_rem / base, big_rem % base);
				if out_idx >= output.len() {
					return Err(EncodeError {
						kind: EncodeKind::BufferTooSmall,
					});
				}
				output[out_idx] = alphabet[remainder as usize];
				out_idx += 1;
				big_rem = result;
			}
		}
	}

	// Add leaders (zeros at the beginning of input)
	let leader = alphabet[0];
	for &byte in input.iter().take(input.len() - 1) {
		if byte == 0 {
			if out_idx >= output.len() {
				return Err(EncodeError {
					kind: EncodeKind::BufferTooSmall,
				});
			}
			output[out_idx] = leader;
			out_idx += 1;
		} else {
			break;
		}
	}

	// Reverse the output in place
	output[..out_idx].reverse();

	Ok(out_idx)
}

/// Decode input string using arithmetic decoding with the given alphabet.
///
/// This function writes the decoded result to the provided output buffer.
/// Returns the number of bytes written to the output buffer, or an error
/// if the input contains invalid characters or the output buffer is too small.
pub(crate) fn decode_to_buffer(
	alphabet: &[u8],
	input: &str,
	output: &mut [u8],
) -> Result<usize, DecodeError> {
	if input.is_empty() {
		return Ok(0);
	}

	let base = alphabet.len() as u32;

	// Build lookup table for alphabet characters
	const INVALID_INDEX: u8 = 0xFF;
	let mut lookup = [INVALID_INDEX; 256];

	for (i, &byte) in alphabet.iter().enumerate() {
		lookup[byte as usize] = i as u8;
	}

	// On utilise un buffer fixe sur la pile pour le calcul intermédiaire.
	// 128 chunks = 512 octets de données binaires max.
	let mut chunks = [0u32; MAX_BIGINT_BUFFER];
	let mut big = BigUintView::new(&mut chunks);

	for byte in input.bytes() {
		match lookup[byte as usize] {
			INVALID_INDEX => {
				// Find the position of the invalid character
				let position = input
					.bytes()
					.enumerate()
					.find(|&(_, b)| b == byte)
					.map(|(pos, _)| pos)
					.unwrap_or(0);

				return Err(DecodeError {
					position,
					kind: DecodeKind::Symbol,
				});
			}
			index => {
				if let Err(()) = big.mul_add(base, index as u32) {
					return Err(DecodeError {
						position: 0, // Position isn't meaningful for overflow
						kind: DecodeKind::Overflow,
					});
				}
			}
		}
	}

	let written = big.copy_into_bytes_be(output).map_err(|_| DecodeError {
		position: 0, // Position isn't meaningful for buffer too small
		kind: DecodeKind::BufferTooSmall,
	})?;

	let leader = alphabet[0];
	let leaders = input.bytes().take_while(|&byte| byte == leader).count();

	if leaders > 0 {
		if output.len() < written + leaders {
			return Err(DecodeError {
				position: 0, // Position isn't meaningful for buffer too small
				kind: DecodeKind::BufferTooSmall,
			});
		}
		// On décale les données vers la droite pour insérer les zéros de tête.
		output.copy_within(0..written, leaders);
		output[..leaders].fill(0);
	}

	Ok(written + leaders)
}

#[cfg(feature = "alloc")]
mod alloc_impl {
	use super::*;
	use crate::String;
	use crate::Vec;

	/// Encode input bytes using arithmetic encoding with the given alphabet.
	///
	/// This function returns the encoded result as a String.
	pub(crate) fn encode(alphabet: &[u8], input: &[u8]) -> Result<String, EncodeError> {
		if input.is_empty() {
			return Ok(String::new());
		}

		let base = alphabet.len() as u32;

		// Stack buffer for BigUint computation (512 bytes capacity)
		let mut chunks = [0u32; MAX_BIGINT_BUFFER];
		let mut big = BigUintView::new(&mut chunks);

		if !big.load_be_bytes(input) {
			return Err(EncodeError {
				kind: EncodeKind::Overflow,
			});
		}

		let mut out = Vec::new();
		let big_pow = 32 / (32 - base.leading_zeros());
		let big_base = base.pow(big_pow);

		'fast: loop {
			let mut big_rem = big.div_mod(big_base);

			if big.is_zero() {
				loop {
					let (result, remainder) = (big_rem / base, big_rem % base);
					out.push(alphabet[remainder as usize]);
					big_rem = result;

					if big_rem == 0 {
						break 'fast;
					}
				}
			} else {
				for _ in 0..big_pow {
					let (result, remainder) = (big_rem / base, big_rem % base);
					out.push(alphabet[remainder as usize]);
					big_rem = result;
				}
			}
		}

		let leaders =
			input.iter().take(input.len() - 1).take_while(|&&i| i == 0).map(|_| alphabet[0]);

		out.extend(leaders);
		out.reverse();

		// C'est sûr car l'alphabet est ASCII et les calculs bigint préservent les bytes.
		Ok(String::from_utf8(out)
			.expect("Alphabet checked ASCII; indices always within bounds; qed"))
	}

	/// Decode input string using arithmetic decoding with the given alphabet.
	///
	/// This function returns the decoded result as a Vec<u8>.
	pub(crate) fn decode(alphabet: &[u8], input: &str) -> Result<Vec<u8>, DecodeError> {
		if input.is_empty() {
			return Ok(Vec::new());
		}

		let base = alphabet.len() as u32;

		// Build lookup table for alphabet characters
		const INVALID_INDEX: u8 = 0xFF;
		let mut lookup = [INVALID_INDEX; 256];

		for (i, &byte) in alphabet.iter().enumerate() {
			lookup[byte as usize] = i as u8;
		}

		// Use heap-allocated BigUint for potentially larger computations
		let mut big = crate::bigint::BigUint::with_capacity(4);

		for (position, byte) in input.bytes().enumerate() {
			match lookup[byte as usize] {
				INVALID_INDEX => {
					return Err(DecodeError {
						position,
						kind: DecodeKind::Symbol,
					});
				}
				index => {
					big.mul_add(base, index as u32);
				}
			}
		}

		let bytes = big.into_bytes_be();

		let leader = alphabet[0];
		let leaders = input.bytes().take_while(|&byte| byte == leader).count();

		let mut res = Vec::with_capacity(bytes.len() + leaders);
		res.resize(leaders, 0);
		res.extend(bytes);

		Ok(res)
	}
}

#[cfg(feature = "alloc")]
pub(crate) use alloc_impl::{decode, encode};
