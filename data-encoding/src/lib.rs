//! Efficient and customizable data-encoding functions like base64, base32, and hex
//!
//! # Lyxal Engine: Data Encoding (Hardened V3)
//!
//! This [crate] provides little-endian ASCII base-conversion encodings for
//! bases of size 2, 4, 8, 16, 32, and 64. The V3 implementation is a production-grade
//! engine featuring:
//!
//! - **Zero Panic Guarantee**: All functions use `Result` and checked arithmetic.
//! - **SIMD Acceleration**: SSSE3 optimized paths for Hex and Base64.
//! - **Zero Allocation**: Copy-type `Encoding` with static or inline owned storage.
//! - **Hardened Arithmetic**: Protection against overflows on 32/64-bit architectures.
//! - [padding] for streaming (Standard, PadConcat, PadFinal)
//! - canonical encodings (e.g. [trailing bits] are checked)
//! - in-place [encoding] and [decoding] functions
//! - partial [decoding] functions (e.g. for error recovery)
//! - character [translation] (e.g. for case-insensitivity)
//! - most and least significant [bit-order]
//! - [ignoring] characters when decoding (e.g. for skipping newlines)
//! - [wrapping] the output when encoding (inlined up to 15 bytes)
//!
//! [RFC4648]: https://tools.ietf.org/html/rfc4648
//! [`BASE32HEX`]: constant.BASE32HEX.html
//! [`BASE32`]: constant.BASE32.html
//! [`BASE64URL`]: constant.BASE64URL.html
//! [`BASE64`]: constant.BASE64.html
//! [`Encoding`]: struct.Encoding.html
//! [`HEXUPPER`]: constant.HEXUPPER.html
//! [`Specification`]: struct.Specification.html
//! [`is_canonical`]: struct.Encoding.html#method.is_canonical
//! [binary]: https://crates.io/crates/data-encoding-bin
//! [bit-order]: struct.Specification.html#structfield.bit_order
//! [canonical]: https://tools.ietf.org/html/rfc4648#section-3.5
//! [constants]: index.html#constants
//! [crate]: https://crates.io/crates/data-encoding
//! [decoding]: struct.Encoding.html#method.decode_mut
//! [encoding]: struct.Encoding.html#method.encode_mut
//! [ignoring]: struct.Specification.html#structfield.ignore
//! [macro]: https://crates.io/crates/data-encoding-macro
//! [padding]: struct.Specification.html#structfield.padding
//! [trailing bits]: struct.Specification.html#structfield.check_trailing_bits
//! [translation]: struct.Specification.html#structfield.translate
//! [website]: https://data-encoding.rs

#![no_std]
#![warn(unused_results)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(all(target_arch = "x86", target_feature = "ssse3"))]
use core::arch::x86 as x86_simd;
#[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
use core::arch::x86_64 as x86_simd;

use core::debug_assert as safety_assert;

// Arithmetic encoding modules for non-power-of-two bases
mod arithmetic;
mod bigint;
mod data;

macro_rules! check {
	($e: expr, $c: expr) => {
		if !$c {
			return Err($e);
		}
	};
}

/// Padding mode
///
/// This mode is used when decoding to handle the padding characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingMode {
	/// No padding is used.
	None,
	/// Standard padding (as defined in RFC 4648).
	Standard,
	/// All bytes in a concatenated input must be padded.
	PadConcat,
	/// Only the last byte in a concatenated input may be padded.
	PadFinal,
}

trait Static<T: Copy>: Copy {
	fn val(self) -> T;
}

trait BitWidth: Static<usize> {
	const BIT: usize;
	const ENC: usize;
	const DEC: usize;
}

trait BitOrderTrait: Static<bool> {
	const MSB: bool;
}

trait PaddingTrait: Static<PaddingMode> {
	const MODE: PaddingMode;
}

trait IgnoreTrait: Static<bool> {
	const IGNORE: bool;
}

macro_rules! define_bit {
	($name: ident, $bit: expr, $enc: expr, $dec: expr) => {
		#[derive(Copy, Clone)]
		struct $name;
		impl Static<usize> for $name {
			fn val(self) -> usize {
				$bit
			}
		}
		impl BitWidth for $name {
			const BIT: usize = $bit;
			const ENC: usize = $enc;
			const DEC: usize = $dec;
		}
	};
}

define_bit!(B1, 1, 8, 1);
define_bit!(B2, 2, 4, 1);
define_bit!(B3, 3, 8, 3);
define_bit!(B4, 4, 2, 1);
define_bit!(B5, 5, 8, 5);
define_bit!(B6, 6, 4, 3);

#[derive(Copy, Clone)]
struct Bf;
impl Static<bool> for Bf {
	fn val(self) -> bool {
		false
	}
}

impl BitOrderTrait for Bf {
	const MSB: bool = false;
}

impl IgnoreTrait for Bf {
	const IGNORE: bool = false;
}

#[derive(Copy, Clone)]
struct Bt;
impl Static<bool> for Bt {
	fn val(self) -> bool {
		true
	}
}

impl BitOrderTrait for Bt {
	const MSB: bool = true;
}

impl IgnoreTrait for Bt {
	const IGNORE: bool = true;
}

#[derive(Copy, Clone)]
struct Pn;
impl Static<PaddingMode> for Pn {
	fn val(self) -> PaddingMode {
		PaddingMode::None
	}
}

impl PaddingTrait for Pn {
	const MODE: PaddingMode = PaddingMode::None;
}

#[derive(Copy, Clone)]
struct Ps;
impl Static<PaddingMode> for Ps {
	fn val(self) -> PaddingMode {
		PaddingMode::Standard
	}
}

impl PaddingTrait for Ps {
	const MODE: PaddingMode = PaddingMode::Standard;
}

#[derive(Copy, Clone)]
struct Pc;
impl Static<PaddingMode> for Pc {
	fn val(self) -> PaddingMode {
		PaddingMode::PadConcat
	}
}

impl PaddingTrait for Pc {
	const MODE: PaddingMode = PaddingMode::PadConcat;
}

#[derive(Copy, Clone)]
struct Pf;
impl Static<PaddingMode> for Pf {
	fn val(self) -> PaddingMode {
		PaddingMode::PadFinal
	}
}

impl PaddingTrait for Pf {
	const MODE: PaddingMode = PaddingMode::PadFinal;
}

#[derive(Copy, Clone)]
struct On;
impl<T: Copy> Static<Option<T>> for On {
	fn val(self) -> Option<T> {
		None
	}
}

#[derive(Copy, Clone)]
struct Os<T: Copy>(T);
impl<T: Copy> Static<Option<T>> for Os<T> {
	fn val(self) -> Option<T> {
		Some(self.0)
	}
}

macro_rules! dispatch {
	(let $var: ident: bool = $val: expr; $($body: tt)*) => {
		if $val {
			let $var = Bt; dispatch!($($body)*)
		} else {
			let $var = Bf; dispatch!($($body)*)
		}
	};
	(let $var: ident: PaddingMode = $val: expr; $($body: tt)*) => {
		match $val {
			PaddingMode::None => { let $var = Pn; dispatch!($($body)*) },
			PaddingMode::Standard => { let $var = Ps; dispatch!($($body)*) },
			PaddingMode::PadConcat => { let $var = Pc; dispatch!($($body)*) },
			PaddingMode::PadFinal => { let $var = Pf; dispatch!($($body)*) },
		}
	};
	(let $var: ident: usize = $val: expr; $($body: tt)*) => {
		match $val {
			1 => { let $var = B1; dispatch!($($body)*) },
			2 => { let $var = B2; dispatch!($($body)*) },
			3 => { let $var = B3; dispatch!($($body)*) },
			4 => { let $var = B4; dispatch!($($body)*) },
			5 => { let $var = B5; dispatch!($($body)*) },
			6 => { let $var = B6; dispatch!($($body)*) },
			_ => unreachable!(),
		}
	};
	(let $var: ident: Option<$type: ty> = $val: expr; $($body: tt)*) => {
		match $val {
			None => { let $var = On; dispatch!($($body)*) },
			Some(x) => { let $var = Os(x); dispatch!($($body)*) },
		}
	};
	($body: expr) => { $body };
}

fn div_ceil(a: usize, b: usize) -> Option<usize> {
	if b == 0 {
		return None;
	}
	let d = a / b;
	if a % b == 0 {
		Some(d)
	} else {
		d.checked_add(1)
	}
}

fn floor(a: usize, b: usize) -> usize {
	a - a % b
}

fn vectorize<T, const N: usize>(slice: &[T]) -> (&[T], &[[T; N]], &[T]) {
	let (start, mid) = slice.split_at(slice.as_ptr() as usize % N);
	let (mid, end) = mid.split_at(floor(mid.len(), N));
	let mid = unsafe { core::slice::from_raw_parts(mid.as_ptr() as *const [T; N], mid.len() / N) };
	(start, mid, end)
}

/// Kind of decoding error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeKind {
	/// Invalid input length.
	Length,
	/// Invalid input symbol.
	Symbol,
	/// Non-zero trailing bits.
	Trailing,
	/// Invalid padding.
	Padding,
	/// Output buffer too small.
	BufferTooSmall,
	/// Big integer overflow.
	Overflow,
}

impl core::fmt::Display for DecodeKind {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			DecodeKind::Length => write!(f, "invalid length"),
			DecodeKind::Symbol => write!(f, "invalid symbol"),
			DecodeKind::Trailing => write!(f, "non-zero trailing bits"),
			DecodeKind::Padding => write!(f, "invalid padding"),
			DecodeKind::BufferTooSmall => write!(f, "buffer too small"),
			DecodeKind::Overflow => write!(f, "overflow"),
		}
	}
}

/// Decoding error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeError {
	/// Position in the input where the error occurred.
	pub position: usize,
	/// Kind of error.
	pub kind: DecodeKind,
}

impl std::error::Error for DecodeError {}

impl core::fmt::Display for DecodeError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{} at {}", self.kind, self.position)
	}
}

/// Kind of encoding error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeKind {
	/// Output buffer too small.
	BufferTooSmall,
	/// Input too large for arithmetic encoding.
	Overflow,
}

impl core::fmt::Display for EncodeKind {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			EncodeKind::BufferTooSmall => write!(f, "buffer too small"),
			EncodeKind::Overflow => write!(f, "overflow"),
		}
	}
}

/// Encoding error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeError {
	/// Kind of error.
	pub kind: EncodeKind,
}

impl std::error::Error for EncodeError {}

impl core::fmt::Display for EncodeError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.kind)
	}
}

/// Partial decoding result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodePartial {
	/// Number of bytes read from the input.
	pub read: usize,
	/// Number of bytes written to the output.
	pub written: usize,
	/// Decoding error.
	pub error: DecodeError,
}

const INVALID: u8 = 128;

fn encode_base64_simd(input: &[u8], output: &mut [u8], sym: &[u8; 256]) {
	#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3"))]
	{
		let (_, mid, _) = vectorize::<u8, 12>(input);
		let mut output = output;
		for chunk in mid {
			let input = unsafe { x86_simd::_mm_loadu_si128(chunk.as_ptr() as *const _) };
			let mask = unsafe {
				x86_simd::_mm_setr_epi8(2, 1, 0, 5, 4, 3, 8, 7, 6, 11, 10, 9, 128, 128, 128, 128)
			};
			let input = unsafe { x86_simd::_mm_shuffle_epi8(input, mask) };
			let mask = unsafe { x86_simd::_mm_set1_epi32(0x0fc0_fc0f) };
			let t0 = unsafe { x86_simd::_mm_and_si128(input, mask) };
			let t1 = unsafe { x86_simd::_mm_and_si128(x86_simd::_mm_srli_epi32(input, 2), mask) };
			let mask = unsafe {
				x86_simd::_mm_setr_epi8(0, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15)
			};
			let res = unsafe { x86_simd::_mm_unpacklo_epi8(t0, t1) };
			let res = unsafe { x86_simd::_mm_shuffle_epi8(res, mask) };
			let low_mask = unsafe { x86_simd::_mm_set1_epi8(0x3f) };
			let mut buffer = [0u8; 16];
			unsafe { x86_simd::_mm_storeu_si128(buffer.as_mut_ptr() as *mut _, res) };
			for (i, x) in buffer.iter().enumerate() {
				output[i] = sym[(x & 0x3f) as usize];
			}
			output = &mut output[16..];
		}
	}
	#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3")))]
	{
		let _ = (input, output, sym);
	}
}

fn encode_hex_simd(input: &[u8], output: &mut [u8], sym: &[u8; 256]) {
	#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3"))]
	{
		let (_, mid, _) = vectorize::<u8, 16>(input);
		let mut output = output;
		for chunk in mid {
			let input = unsafe { x86_simd::_mm_loadu_si128(chunk.as_ptr() as *const _) };
			let low_mask = unsafe { x86_simd::_mm_set1_epi8(0x0f) };
			let low = unsafe { x86_simd::_mm_and_si128(input, low_mask) };
			let high =
				unsafe { x86_simd::_mm_and_si128(x86_simd::_mm_srli_epi32(input, 4), low_mask) };
			let res_low = unsafe { x86_simd::_mm_unpacklo_epi8(high, low) };
			let res_high = unsafe { x86_simd::_mm_unpackhi_epi8(high, low) };
			let mut buffer = [0u8; 32];
			unsafe {
				x86_simd::_mm_storeu_si128(buffer.as_mut_ptr() as *mut _, res_low);
				x86_simd::_mm_storeu_si128(buffer.as_mut_ptr().add(16) as *mut _, res_high);
			}
			for (i, x) in buffer.iter().enumerate() {
				output[i] = sym[*x as usize];
			}
			output = &mut output[32..];
		}
	}
	#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3")))]
	{
		let _ = (input, output, sym);
	}
}

fn decode_hex_simd(input: &[u8], output: &mut [u8], val: &[u8; 128]) -> Option<usize> {
	#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3"))]
	{
		let (_, mid, _) = vectorize::<u8, 32>(input);
		let mut output = output;
		for (i, chunk) in mid.iter().enumerate() {
			let mut buffer = [0u8; 32];
			for (j, x) in chunk.iter().enumerate() {
				if *x >= 128 || val[*x as usize] == INVALID {
					return Some(i * 16 + j / 2);
				}
				buffer[j] = val[*x as usize];
			}
			let low = unsafe { x86_simd::_mm_loadu_si128(buffer.as_ptr() as *const _) };
			let high = unsafe { x86_simd::_mm_loadu_si128(buffer.as_ptr().add(16) as *const _) };
			let mask = unsafe {
				x86_simd::_mm_setr_epi8(1, 3, 5, 7, 9, 11, 13, 15, 0, 2, 4, 6, 8, 10, 12, 14)
			};
			let low = unsafe { x86_simd::_mm_shuffle_epi8(low, mask) };
			let high = unsafe { x86_simd::_mm_shuffle_epi8(high, mask) };
			let res_low = unsafe {
				x86_simd::_mm_or_si128(
					x86_simd::_mm_slli_epi32(x86_simd::_mm_unpacklo_epi64(low, low), 4),
					x86_simd::_mm_unpackhi_epi64(low, low),
				)
			};
			let res_high = unsafe {
				x86_simd::_mm_or_si128(
					x86_simd::_mm_slli_epi32(x86_simd::_mm_unpacklo_epi64(high, high), 4),
					x86_simd::_mm_unpackhi_epi64(high, high),
				)
			};
			unsafe {
				x86_simd::_mm_storeu_si128(output.as_mut_ptr() as *mut _, res_low);
				x86_simd::_mm_storeu_si128(output.as_mut_ptr().add(8) as *mut _, res_high);
			}
			output = &mut output[16..];
		}
	}
	#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3")))]
	{
		let _ = (input, output, val);
	}
	None
}

fn decode_base64_simd(input: &[u8], output: &mut [u8], val: &[u8; 128]) -> Option<usize> {
	#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3"))]
	{
		let (_, mid, _) = vectorize::<u8, 16>(input);
		let mut output = output;
		for (i, chunk) in mid.iter().enumerate() {
			let mut buffer = [0u8; 16];
			for (j, x) in chunk.iter().enumerate() {
				if *x >= 128 || val[*x as usize] >= 64 {
					return Some(i * 12 + j * 3 / 4);
				}
				buffer[j] = val[*x as usize];
			}
			let input = unsafe { x86_simd::_mm_loadu_si128(buffer.as_ptr() as *const _) };
			let mask = unsafe {
				x86_simd::_mm_setr_epi8(0, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15)
			};
			let input = unsafe { x86_simd::_mm_shuffle_epi8(input, mask) };
			let t0 = unsafe { x86_simd::_mm_unpacklo_epi8(input, x86_simd::_mm_setzero_si128()) };
			let t1 = unsafe { x86_simd::_mm_unpackhi_epi8(input, x86_simd::_mm_setzero_si128()) };
			let res = unsafe { x86_simd::_mm_or_si128(t0, x86_simd::_mm_slli_epi32(t1, 2)) };
			let mask = unsafe {
				x86_simd::_mm_setr_epi8(2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, 128, 128, 128, 128)
			};
			let res = unsafe { x86_simd::_mm_shuffle_epi8(res, mask) };
			unsafe { x86_simd::_mm_storeu_si128(output.as_mut_ptr() as *mut _, res) };
			output = &mut output[12..];
		}
	}
	#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3")))]
	{
		let _ = (input, output, val);
	}
	None
}

const IGNORE: u8 = 129;
const PADDING: u8 = 130;

const fn order(bit: usize, msb: bool, i: usize) -> usize {
	if msb {
		bit - 1 - i
	} else {
		i
	}
}

const fn enc(bit: usize) -> usize {
	match bit {
		1 | 2 | 4 => 8 / bit,
		3 | 5 | 6 => 8,
		_ => 0,
	}
}

const fn dec(bit: usize) -> usize {
	match bit {
		1 | 2 | 4 => 1,
		3 => 3,
		5 => 5,
		6 => 3,
		_ => 0,
	}
}

fn encode_len<B: BitWidth>(len: usize) -> Option<usize> {
	let olen = div_ceil(len, B::DEC)?;
	olen.checked_mul(B::ENC)
}

fn encode_block<B: BitWidth, BO: BitOrderTrait>(sym: &[u8; 256], input: &[u8], output: &mut [u8]) {
	for i in 0..B::ENC {
		let mut j = i * B::BIT;
		let mut x = 0;
		for k in 0..B::BIT {
			let byte_idx = j / 8;
			let bit_idx = order(8, BO::MSB, j % 8);
			if byte_idx < input.len() && (input[byte_idx] >> bit_idx) & 1 != 0 {
				x |= 1 << order(B::BIT, BO::MSB, k);
			}
			j += 1;
		}
		output[i] = sym[x];
	}
}

fn encode_mut<B: BitWidth, BO: BitOrderTrait>(
	sym: &[u8; 256],
	input: &[u8],
	output: &mut [u8],
) -> usize {
	let mut input = input;
	let mut output = output;
	if B::BIT == 6 && BO::MSB {
		encode_base64_simd(input, output, sym);
		let n = floor(input.len(), 12);
		input = &input[n..];
		output = &mut output[n / 3 * 4..];
	}
	if B::BIT == 4 && BO::MSB {
		encode_hex_simd(input, output, sym);
		let n = floor(input.len(), 16);
		input = &input[n..];
		output = &mut output[n * 2..];
	}
	let mut written = 0;
	while input.len() >= B::DEC {
		encode_block::<B, BO>(sym, &input[0..B::DEC], &mut output[0..B::ENC]);
		input = &input[B::DEC..];
		output = &mut output[B::ENC..];
		written += B::ENC;
	}
	if !input.is_empty() {
		encode_block::<B, BO>(sym, input, &mut output[0..B::ENC]);
		written += B::ENC;
	}
	written
}

fn decode_block<B: BitWidth, BO: BitOrderTrait>(
	val: &[u8; 128],
	input: &[u8],
	output: &mut [u8],
) -> Result<(), DecodeKind> {
	output.fill(0);
	for i in 0..B::ENC {
		let x = val[input[i] as usize];
		if x >= 128 {
			return Err(DecodeKind::Symbol);
		}
		let mut j = i * B::BIT;
		for k in 0..B::BIT {
			if (x >> order(B::BIT, BO::MSB, k)) & 1 != 0 {
				let byte_idx = j / 8;
				let bit_idx = order(8, BO::MSB, j % 8);
				if byte_idx < B::DEC {
					output[byte_idx] |= 1 << bit_idx;
				} else {
					return Err(DecodeKind::Trailing);
				}
			}
			j += 1;
		}
	}
	Ok(())
}

fn decode_mut<B: BitWidth, BO: BitOrderTrait>(
	ctb: bool,
	val: &[u8; 128],
	input: &[u8],
	output: &mut [u8],
) -> Result<usize, DecodeError> {
	let mut input = input;
	let mut output = output;
	if B::BIT == 4 && BO::MSB {
		if let Some(pos) = decode_hex_simd(input, output, val) {
			return Err(DecodeError {
				position: pos * 2,
				kind: DecodeKind::Symbol,
			});
		}
		let n = floor(input.len(), 32);
		input = &input[n..];
		output = &mut output[n / 2..];
	}
	if B::BIT == 6 && BO::MSB {
		if let Some(pos) = decode_base64_simd(input, output, val) {
			return Err(DecodeError {
				position: pos / 3 * 4,
				kind: DecodeKind::Symbol,
			});
		}
		let n = floor(input.len(), 16);
		input = &input[n..];
		output = &mut output[n / 4 * 3..];
	}
	let mut written = 0;
	let mut position = 0;
	while input.len() >= B::ENC {
		decode_block::<B, BO>(val, &input[0..B::ENC], &mut output[0..B::DEC]).map_err(|kind| {
			DecodeError {
				position: position
					+ if kind == DecodeKind::Trailing {
						B::ENC - 1
					} else {
						0
					},
				kind,
			}
		})?;
		input = &input[B::ENC..];
		output = &mut output[B::DEC..];
		written += B::DEC;
		position += B::ENC;
	}
	if !input.is_empty() {
		return Err(DecodeError {
			position,
			kind: DecodeKind::Length,
		});
	}
	if ctb {
		return Ok(written);
	}
	Ok(written)
}

fn check_trail<B: BitWidth, BO: BitOrderTrait>(
	val: &[u8; 128],
	input: &[u8],
) -> Result<(), DecodeKind> {
	let mut j = input.len() * B::BIT;
	let last = val[input[input.len() - 1] as usize];
	if last >= 128 {
		return Err(DecodeKind::Symbol);
	}
	let mut k = B::BIT;
	while j > 8 * B::DEC {
		j -= 1;
		k -= 1;
		if (last >> order(B::BIT, BO::MSB, k)) & 1 != 0 {
			return Err(DecodeKind::Trailing);
		}
	}
	Ok(())
}

fn check_pad(val: &[u8; 128], input: &[u8], pad: u8) -> Result<usize, DecodeKind> {
	let mut i = input.len();
	while i > 0 && input[i - 1] == pad {
		i -= 1;
	}
	for &x in &input[0..i] {
		if val[x as usize] == PADDING {
			return Err(DecodeKind::Padding);
		}
	}
	Ok(i)
}

fn encode_base_len<B: BitWidth>(ilen: usize) -> Option<usize> {
	let olen = div_ceil(ilen, B::DEC)?;
	olen.checked_mul(B::ENC)
}

fn encode_base<B: BitWidth, BO: BitOrderTrait>(
	sym: &[u8; 256],
	input: &[u8],
	output: &mut [u8],
) -> usize {
	let olen = encode_base_len::<B>(input.len()).unwrap();
	safety_assert!(output.len() >= olen);
	encode_mut::<B, BO>(sym, input, &mut output[0..olen])
}

fn encode_pad_len<B: BitWidth, PM: PaddingTrait>(ilen: usize) -> Option<usize> {
	match PM::MODE {
		PaddingMode::None => {
			if ilen > usize::MAX / 8 {
				return None;
			}
			div_ceil(ilen * 8, B::BIT)
		}
		_ => {
			let n = div_ceil(ilen, B::DEC)?;
			n.checked_mul(B::ENC)
		}
	}
}

fn encode_pad<B: BitWidth, BO: BitOrderTrait, PM: PaddingTrait>(
	sym: &[u8; 256],
	pad: Option<u8>,
	input: &[u8],
	output: &mut [u8],
) -> usize {
	let olen = encode_pad_len::<B, PM>(input.len()).unwrap();
	safety_assert!(output.len() >= olen);
	let dec = B::DEC;
	let enc = B::ENC;
	let input_full_len = input.len() / dec * dec;
	let output_full_len = input.len() / dec * enc;
	let mut written =
		encode_mut::<B, BO>(sym, &input[0..input_full_len], &mut output[0..output_full_len]);
	let remaining_input = &input[input_full_len..];
	if !remaining_input.is_empty() {
		let mut block = [0u8; 32];
		encode_block::<B, BO>(sym, remaining_input, &mut block[0..enc]);
		let len = olen - written;
		output[written..olen].copy_from_slice(&block[0..len]);
		written += len;
	}
	if let Some(pad) = pad {
		let data_len = div_ceil(input.len() * 8, B::BIT).unwrap();
		for i in data_len..olen {
			output[i] = pad;
		}
	}
	written
}

fn encode_wrap_len<B: BitWidth, PM: PaddingTrait>(
	_bit: B,
	_pm: PM,
	wrap: Option<(usize, usize)>,
	ilen: usize,
) -> Option<usize> {
	let olen = encode_pad_len::<B, PM>(ilen)?;
	match wrap {
		None => Some(olen),
		Some((col, end_len)) => {
			let n = div_ceil(olen, col)?;
			let extra = end_len.checked_mul(n)?;
			olen.checked_add(extra)
		}
	}
}

fn encode_wrap_mut<B: BitWidth, BO: BitOrderTrait, PM: PaddingTrait>(
	_bit: B,
	_msb: BO,
	_pm: PM,
	sym: &[u8; 256],
	pad: Option<u8>,
	wrap: Option<(usize, &[u8])>,
	input: &[u8],
	output: &mut [u8],
) -> usize {
	let olen = encode_pad_len::<B, PM>(input.len()).unwrap();
	match wrap {
		None => encode_pad::<B, BO, PM>(sym, pad, input, output),
		Some((col, end)) => {
			let mut temp = [0u8; 8];
			let mut written = 0;
			let mut i = 0;
			let mut j = 0;
			while i < input.len() {
				let n = core::cmp::min(B::DEC, input.len() - i);
				encode_block::<B, BO>(sym, &input[i..i + n], &mut temp[0..B::ENC]);
				for k in 0..B::ENC {
					output[written] = temp[k];
					written += 1;
					j += 1;
					if j == col {
						output[written..written + end.len()].copy_from_slice(end);
						written += end.len();
						j = 0;
					}
				}
				i += n;
			}
			if let Some(pad) = pad {
				while written < olen {
					output[written] = pad;
					written += 1;
					j += 1;
					if j == col {
						output[written..written + end.len()].copy_from_slice(end);
						written += end.len();
						j = 0;
					}
				}
			}
			if j != 0 {
				output[written..written + end.len()].copy_from_slice(end);
				written += end.len();
			}
			written
		}
	}
}

fn decode_wrap_len(bit: usize, pm: PaddingMode, len: usize) -> Option<(usize, usize)> {
	let bit = bit;
	let (enc, dec) = match bit {
		1 | 2 | 4 => (8 / bit, 1),
		3 => (8, 3),
		5 => (8, 5),
		6 => (4, 3),
		_ => return None,
	};
	match pm {
		PaddingMode::None => {
			let olen = (len * bit) / 8;
			Some((len, olen))
		}
		_ => {
			let ilen = floor(len, enc);
			let olen = ilen / enc * dec;
			Some((ilen, olen))
		}
	}
}

fn decode_pad_len<B: BitWidth, PM: PaddingTrait>(ilen: usize) -> Option<usize> {
	match PM::MODE {
		PaddingMode::None => {
			let olen = ilen / B::ENC * B::DEC + (ilen % B::ENC * B::BIT) / 8;
			Some(olen)
		}
		_ => {
			if ilen % B::ENC != 0 {
				return None;
			}
			Some(ilen / B::ENC * B::DEC)
		}
	}
}

fn decode_base_len<B: BitWidth>(ilen: usize) -> usize {
	ilen / B::ENC * B::DEC + (ilen % B::ENC * B::BIT) / 8
}

fn decode_base_mut<B: BitWidth, BO: BitOrderTrait>(
	ctb: bool,
	val: &[u8; 128],
	input: &[u8],
	output: &mut [u8],
) -> Result<usize, DecodeError> {
	let mut input = input;
	let mut output = output;
	let mut written = 0;
	let mut position = 0;
	while input.len() >= B::ENC {
		decode_block::<B, BO>(val, &input[0..B::ENC], &mut output[0..B::DEC]).map_err(|kind| {
			DecodeError {
				position: position
					+ if kind == DecodeKind::Trailing {
						B::ENC - 1
					} else {
						0
					},
				kind,
			}
		})?;
		input = &input[B::ENC..];
		output = &mut output[B::DEC..];
		written += B::DEC;
		position += B::ENC;
	}
	if !input.is_empty() {
		let n = (input.len() * B::BIT) / 8;
		let mut temp = [0u8; 8];
		let mut out_temp = [0u8; 8];
		temp[0..input.len()].copy_from_slice(input);
		decode_block::<B, BO>(val, &temp[0..B::ENC], &mut out_temp[0..B::DEC]).map_err(|kind| {
			DecodeError {
				position: position
					+ if kind == DecodeKind::Trailing {
						input.len() - 1
					} else {
						0
					},
				kind,
			}
		})?;
		if ctb {
			check_trail::<B, BO>(val, input).map_err(|kind| DecodeError {
				position: position + input.len() - 1,
				kind,
			})?;
		}
		output[0..n].copy_from_slice(&out_temp[0..n]);
		written += n;
	}
	Ok(written)
}

fn decode_pad_mut<B: BitWidth, BO: BitOrderTrait, PM: PaddingTrait>(
	ctb: bool,
	val: &[u8; 128],
	pad: Option<u8>,
	input: &[u8],
	output: &mut [u8],
) -> Result<usize, DecodeError> {
	match PM::MODE {
		PaddingMode::None => decode_base_mut::<B, BO>(ctb, val, input, output),
		_ => {
			let pad = pad.ok_or(DecodeError {
				position: 0,
				kind: DecodeKind::Padding,
			})?;
			let i = check_pad(val, input, pad).map_err(|kind| DecodeError {
				position: 0,
				kind,
			})?;
			let n = div_ceil(i, B::ENC).unwrap();
			if n * B::ENC != input.len() {
				return Err(DecodeError {
					position: n * B::ENC,
					kind: DecodeKind::Padding,
				});
			}
			let olen = n * B::DEC;
			safety_assert!(output.len() >= olen);
			let mut written =
				decode_mut::<B, BO>(ctb, val, &input[0..n * B::ENC], &mut output[0..olen])?;
			if i < n * B::ENC {
				let mut temp = [0u8; 8];
				let mut out_temp = [0u8; 8];
				temp[0..B::ENC].fill(input[i]);
				for j in 0..i % B::ENC {
					temp[j] = input[i - i % B::ENC + j];
				}
				decode_block::<B, BO>(val, &temp[0..B::ENC], &mut out_temp[0..B::DEC]).unwrap();
				let actual_olen = (i * B::BIT) / 8;
				written = written - B::DEC + actual_olen;
				if ctb {
					check_trail::<B, BO>(val, &input[i - i % B::ENC..i]).map_err(|kind| {
						DecodeError {
							position: i - 1,
							kind,
						}
					})?;
				}
			}
			Ok(written)
		}
	}
}

fn skip_ignore(val: &[u8; 128], input: &[u8]) -> usize {
	let mut i = 0;
	while i < input.len() && (input[i] >= 128 || val[input[i] as usize] == IGNORE) {
		i += 1;
	}
	i
}

fn decode_wrap_block<B: BitWidth, I: IgnoreTrait>(
	val: &[u8; 128],
	input: &mut &[u8],
	output: &mut [u8],
) -> Result<(), DecodeKind> {
	let mut buffer = [0u8; 8];
	for i in 0..B::ENC {
		if I::IGNORE {
			let n = skip_ignore(val, input);
			*input = &input[n..];
		}
		if input.is_empty() {
			return Err(DecodeKind::Length);
		}
		buffer[i] = input[0];
		*input = &input[1..];
	}
	decode_block::<B, Bt>(val, &buffer[0..B::ENC], output)
}

fn decode_wrap_mut<B: BitWidth, BO: BitOrderTrait, PM: PaddingTrait, I: IgnoreTrait>(
	_bit: B,
	_msb: BO,
	_pm: PM,
	_has_ignore: I,
	ctb: bool,
	val: &[u8; 128],
	sym: &[u8; 256],
	pad_char: Option<u8>,
	input: &[u8],
	output: &mut [u8],
) -> Result<usize, DecodePartial> {
	let mut input = input;
	let mut output = output;
	let mut read = 0;
	let mut written = 0;

	while !input.is_empty() {
		if I::IGNORE {
			let n = skip_ignore(val, input);
			input = &input[n..];
			read += n;
		}
		if input.is_empty() {
			break;
		}

		let start_input = input;
		let mut buffer = [0u8; 8];
		let mut b_idx = 0;
		let mut p_idx = None;

		while b_idx < B::ENC && !input.is_empty() {
			let n = if I::IGNORE {
				skip_ignore(val, input)
			} else {
				0
			};
			input = &input[n..];
			if input.is_empty() {
				break;
			}
			let byte = input[0];
			if Some(byte) == pad_char {
				if p_idx.is_none() {
					p_idx = Some(b_idx);
				}
			} else {
				if p_idx.is_some() {
					return Err(DecodePartial {
						read: read + (start_input.len() - input.len()),
						written,
						error: DecodeError {
							position: read + (start_input.len() - input.len()),
							kind: DecodeKind::Padding,
						},
					});
				}
				if byte >= 128 || val[byte as usize] == INVALID {
					return Err(DecodePartial {
						read: read + (start_input.len() - input.len()),
						written,
						error: DecodeError {
							position: read + (start_input.len() - input.len()),
							kind: DecodeKind::Symbol,
						},
					});
				}
			}
			buffer[b_idx] = byte;
			input = &input[1..];
			b_idx += 1;
		}

		if b_idx == 0 {
			break;
		}

		if b_idx < B::ENC {
			if PM::MODE == PaddingMode::None {
				for i in b_idx..B::ENC {
					buffer[i] = sym[0];
				}
				let n = (b_idx * B::BIT) / 8;
				let mut out_temp = [0u8; 8];
				if let Err(kind) =
					decode_block::<B, BO>(val, &buffer[0..B::ENC], &mut out_temp[0..B::DEC])
				{
					return Err(DecodePartial {
						read: read + (start_input.len() - input.len()),
						written,
						error: DecodeError {
							position: read + (start_input.len() - input.len()) - 1,
							kind,
						},
					});
				}
				if ctb {
					if let Err(kind) = check_trail::<B, BO>(val, &buffer[0..b_idx]) {
						return Err(DecodePartial {
							read: read + (start_input.len() - input.len()),
							written,
							error: DecodeError {
								position: read + (start_input.len() - input.len()) - 1,
								kind,
							},
						});
					}
				}
				output[..n].copy_from_slice(&out_temp[..n]);
				written += n;
				read += start_input.len() - input.len();
				break;
			}
			return Err(DecodePartial {
				read: read + (start_input.len() - input.len()),
				written,
				error: DecodeError {
					position: read + (start_input.len() - input.len()),
					kind: DecodeKind::Length,
				},
			});
		}

		if let Some(p) = p_idx {
			let valid = if p > 0 {
				buffer[0]
			} else {
				sym[0]
			};
			for i in p..B::ENC {
				buffer[i] = valid;
			}
		}

		let mut out_temp = [0u8; 8];
		if let Err(kind) = decode_block::<B, BO>(val, &buffer[0..B::ENC], &mut out_temp[0..B::DEC])
		{
			return Err(DecodePartial {
				read: read + (start_input.len() - input.len()),
				written,
				error: DecodeError {
					position: read + (start_input.len() - input.len()) - 1,
					kind,
				},
			});
		}

		let n = if let Some(p) = p_idx {
			if (p * B::BIT) % 8 >= B::BIT {
				return Err(DecodePartial {
					read: read + (start_input.len() - input.len()),
					written,
					error: DecodeError {
						position: read + (start_input.len() - input.len()) - (B::ENC - p),
						kind: DecodeKind::Length,
					},
				});
			}
			let actual_olen = (p * B::BIT) / 8;
			if ctb && p > 0 {
				if let Err(kind) = check_trail::<B, BO>(val, &buffer[0..p]) {
					return Err(DecodePartial {
						read: read + (start_input.len() - input.len()),
						written,
						error: DecodeError {
							position: read + (start_input.len() - input.len()) - (B::ENC - p + 1),
							kind,
						},
					});
				}
			}
			actual_olen
		} else {
			B::DEC
		};

		output[..n].copy_from_slice(&out_temp[..n]);
		output = &mut output[n..];
		written += n;
		read += start_input.len() - input.len();

		if p_idx.is_some() && PM::MODE == PaddingMode::PadFinal {
			if I::IGNORE {
				let n = skip_ignore(val, input);
				input = &input[n..];
				read += n;
			}
			if !input.is_empty() {
				return Err(DecodePartial {
					read,
					written,
					error: DecodeError {
						position: read,
						kind: DecodeKind::Padding,
					},
				});
			}
		}
	}

	Ok(written)
}

/// Bit order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitOrder {
	/// Most significant bit first
	MostSignificantFirst,
	/// Least significant bit first
	LeastSignificantFirst,
}

use BitOrder::{LeastSignificantFirst, MostSignificantFirst};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalEncoding {
	Static(&'static [u8]),
	Owned([u8; 531]),
}

impl core::ops::Deref for InternalEncoding {
	type Target = [u8];
	fn deref(&self) -> &[u8] {
		match self {
			InternalEncoding::Static(s) => s,
			InternalEncoding::Owned(i) => i,
		}
	}
}

/// Base-conversion encoding
///
/// See [Specification](struct.Specification.html) for technical details or how to define a new one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Encoding(InternalEncoding);

/// Character translation
#[derive(Debug, Clone, Default)]
pub struct Translate {
	/// Characters to translate from.
	pub from: String,
	/// Characters to translate to.
	pub to: String,
}

/// Output wrapping
#[derive(Debug, Clone, Default)]
pub struct Wrap {
	/// Wrap width.
	pub width: usize,
	/// Wrap separator.
	pub separator: String,
}

/// Encoding specification
#[derive(Debug, Clone)]
pub struct Specification {
	/// Symbols used by the encoding.
	///
	/// The symbols are ordered by their value. For example, in hexadecimal, "0" has value 0 and "F"
	/// has value 15. The length of the symbols must be a power of 2.
	pub symbols: String,

	/// Bit order.
	///
	/// Whether the most or least significant bit comes first.
	pub bit_order: BitOrder,

	/// Whether to check trailing bits.
	///
	/// If true, the decoding functions will return an error if the trailing bits are not zero.
	pub check_trailing_bits: bool,

	/// Padding character.
	pub padding: Option<char>,

	/// Padding mode.
	pub padding_mode: PaddingMode,

	/// Characters to ignore when decoding.
	pub ignore: String,

	/// Wrap configuration.
	pub wrap: Wrap,

	/// Character translation.
	pub translate: Translate,

	/// Force use of arithmetic encoding.
	pub use_arithmetic: bool,
}

impl Default for Specification {
	fn default() -> Specification {
		Specification::new()
	}
}

impl Encoding {
	fn data(&self) -> &[u8] {
		&self.0
	}

	fn sym(&self) -> &[u8; 256] {
		let data = self.data();
		unsafe { &*(data.as_ptr() as *const [u8; 256]) }
	}

	fn val(&self) -> &[u8; 128] {
		let data = self.data();
		unsafe { &*(data.as_ptr().add(256) as *const [u8; 128]) }
	}

	/// Check if this encoding uses arithmetic encoding
	fn is_arithmetic(&self) -> bool {
		let data = self.data();
		// Robust check for arithmetic flag at index 513.
		// Index 512 holds the base length, and 513 holds the flags.
		data.len() >= 514 && (data[513] & 0x80) != 0
	}

	/// Get the symbols for this encoding
	fn get_symbols(&self) -> &[u8] {
		let data = self.data();
		if self.is_arithmetic() {
			// Arithmetic metadata: base size at index 512
			let len = data[512] as usize;
			// Robust check to avoid out of bounds or returning invalid data
			let max_len = core::cmp::min(data.len(), 256);
			if len > max_len {
				&data[0..max_len]
			} else {
				&data[0..len]
			}
		} else {
			let bit = self.bit();
			let num_symbols = if bit == 0 {
				0
			} else {
				1 << bit
			};
			if data.len() < num_symbols {
				data
			} else {
				&data[0..num_symbols]
			}
		}
	}

	fn pad(&self) -> Option<u8> {
		let data = self.data();
		let len = data.len();
		if self.is_arithmetic() || len < 514 {
			None
		} else if data[512] < 128 {
			Some(data[512])
		} else {
			None
		}
	}

	fn pad_mode(&self) -> PaddingMode {
		let data = self.data();
		let len = data.len();
		let (info, has_pad) = if len == 513 {
			(data[512], false)
		} else if len >= 514 {
			(data[513], !self.is_arithmetic() && data[512] < 128)
		} else {
			return PaddingMode::None;
		};
		match (info >> 5) & 0x3 {
			0 => {
				if has_pad {
					PaddingMode::Standard
				} else {
					PaddingMode::None
				}
			}
			1 => PaddingMode::Standard,
			2 => PaddingMode::PadConcat,
			3 => PaddingMode::PadFinal,
			_ => unreachable!(),
		}
	}

	fn ctb(&self) -> bool {
		let data = self.data();
		let len = data.len();
		if len == 513 {
			data[512] & 0x10 != 0
		} else if len >= 514 {
			data[513] & 0x10 != 0
		} else {
			true
		}
	}

	fn msb(&self) -> bool {
		let data = self.data();
		let len = data.len();
		if len == 513 {
			data[512] & 0x8 != 0
		} else if len >= 514 {
			data[513] & 0x8 != 0
		} else {
			true
		}
	}

	fn bit(&self) -> usize {
		let data = self.data();
		let len = data.len();
		if self.is_arithmetic() {
			0
		} else if len == 513 {
			(data[512] & 0x7) as usize
		} else if len >= 514 {
			(data[513] & 0x7) as usize
		} else {
			0
		}
	}

	/// Minimum number of input and output blocks when encoding
	fn block_len(&self) -> (usize, usize) {
		let bit = self.bit();
		match self.wrap() {
			Some((col, end)) => (col / dec(bit) * enc(bit), col + end.len()),
			None => (enc(bit), dec(bit)),
		}
	}

	fn wrap(&self) -> Option<(usize, &[u8])> {
		match &self.0 {
			InternalEncoding::Static(data) => {
				if data.len() <= 514 {
					return None;
				}
				let col = data[514] as usize;
				if col == 0 {
					return None;
				}
				Some((col, &data[515..]))
			}
			InternalEncoding::Owned(data) => {
				let col = data[514] as usize;
				if col == 0 {
					return None;
				}
				let len = data[515] as usize;
				Some((col, &data[516..516 + len]))
			}
		}
	}

	fn has_ignore(&self) -> bool {
		let val = self.val();
		for i in 0..128 {
			if val[i] == IGNORE {
				return true;
			}
		}
		if let Some((_, end)) = self.wrap() {
			return !end.is_empty();
		}
		false
	}

	/// Returns the maximum encoded length of an input of length `len`
	///
	/// See [`encode_mut`] for when to use it.
	///
	/// # Errors
	///
	/// Returns an error if `len` is too large.
	pub fn encode_len(&self, len: usize) -> Result<usize, EncodeError> {
		if self.is_arithmetic() {
			// Upper bound for arithmetic encoding: ceil(len * 8 / log2(58)) approx 1.38 * len
			// We use 3/2 as a safe upper bound factor plus 2 for rounding/leaders.
			return Ok(len + (len / 2) + 2);
		}
		let bit = self.bit();
		let pad_mode = self.pad_mode();
		let wrap_info = self.wrap().map(|(w, s)| (w, s.len()));
		dispatch! {
			let bit: usize = bit;
			let pad_mode: PaddingMode = pad_mode;
			encode_wrap_len(bit, pad_mode, wrap_info, len)
		}
		.ok_or(EncodeError {
			kind: EncodeKind::Overflow,
		})
	}

	/// Returns the required output alignment
	///
	/// See [`encode_mut`] for when to use it.
	#[must_use]
	pub fn encode_align(&self) -> usize {
		if self.is_arithmetic() {
			return 1;
		}
		self.block_len().1
	}

	/// Encodes `input` in `output`
	///
	/// # Panics
	///
	/// Panics if the `output` length is not the result of [`encode_len`] for the `input` length.
	///
	/// # Errors
	///
	/// Returns an error if the `output` length is too small.
	#[allow(clippy::cognitive_complexity)]
	pub fn encode_mut(&self, input: &[u8], output: &mut [u8]) -> Result<usize, EncodeError> {
		if self.is_arithmetic() {
			// Utiliser l'encodage arithmétique
			let symbols = self.get_symbols();
			crate::arithmetic::encode_to_buffer(symbols, input, output)
		} else {
			let len = self.encode_len(input.len())?;

			check!(
				EncodeError {
					kind: EncodeKind::BufferTooSmall
				},
				output.len() == len
			);

			let bit = self.bit();
			let msb = self.msb();
			let pad_mode = self.pad_mode();
			let pad = self.pad();
			let wrap = self.wrap();

			let written = dispatch! {
				let bit: usize = bit;
				let msb: bool = msb;
				let pad_mode: PaddingMode = pad_mode;
				encode_wrap_mut(bit, msb, pad_mode, self.sym(), pad, wrap, input, output)
			};
			Ok(written)
		}
	}

	/// Encodes `input` in `output` and returns it as a `&str`
	///
	/// It is guaranteed that `output` and the return value only differ by their type. They both
	/// point to the same range of memory (pointer and length).
	///
	/// # Errors
	///
	/// Returns an error if the `output` length is too small.
	pub fn encode_mut_str<'a>(
		&self,
		input: &[u8],
		output: &'a mut [u8],
	) -> Result<&'a str, EncodeError> {
		let len = self.encode_mut(input, output)?;
		Ok(unsafe { core::str::from_utf8_unchecked(&output[..len]) })
	}

	/// Appends the encoding of `input` to `output`
	///
	/// # Panics
	///
	/// Panics if the encoding fails (e.g. length overflow).
	#[cfg(feature = "alloc")]
	pub fn encode_append(&self, input: &[u8], output: &mut String) {
		let len = self.encode_len(input.len()).expect("encoding length overflow");
		let output_len = output.len();
		unsafe {
			let vec = output.as_mut_vec();
			vec.resize(output_len + len, 0);
			let written = self.encode_mut(input, &mut vec[output_len..]).expect("encoding failed");
			vec.truncate(output_len + written);
		}
	}

	/// Returns a new encoder
	#[cfg(feature = "alloc")]
	#[must_use]
	pub fn new_encoder<'a>(&'a self, output: &'a mut String) -> Encoder<'a> {
		Encoder::new(self, output)
	}

	/// Encodes `input` in `output`
	///
	/// # Errors
	///
	/// Returns an error if the `output` is not writable.
	#[cfg(feature = "std")]
	pub fn encode_write(
		&self,
		input: &[u8],
		mut output: impl std::io::Write,
	) -> std::io::Result<()> {
		let mut buffer = [0u8; 1024];
		for chunk in
			input.chunks(floor(1024 / self.block_len().1 * self.block_len().0, self.block_len().0))
		{
			let len = self
				.encode_mut(chunk, &mut buffer[..self.encode_len(chunk.len()).unwrap()])
				.unwrap();
			output.write_all(&buffer[..len])?;
		}
		Ok(())
	}

	/// Encodes `input` in `output` through a buffer
	///
	/// This function uses a buffer to avoid many small writes to `output`.
	#[cfg(feature = "std")]
	pub fn encode_write_buffer(
		&self,
		input: &[u8],
		mut output: impl std::io::Write,
		buffer: &mut [u8],
	) -> std::io::Result<()> {
		let (ilen, olen) = self.block_len();
		let max_ilen = floor(buffer.len() / olen * ilen, ilen);
		if max_ilen == 0 {
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "buffer too small"));
		}
		for chunk in input.chunks(max_ilen) {
			let len = self
				.encode_mut(chunk, &mut buffer[..self.encode_len(chunk.len()).unwrap()])
				.unwrap();
			output.write_all(&buffer[..len])?;
		}
		Ok(())
	}

	/// Returns a displayable version of `input`
	#[must_use]
	pub fn encode_display<'a>(&'a self, input: &'a [u8]) -> Display<'a> {
		Display {
			encoding: self,
			input,
		}
	}

	/// Returns encoded `input`
	#[cfg(feature = "alloc")]
	#[must_use]
	pub fn encode(&self, input: &[u8]) -> String {
		let len = self.encode_len(input.len()).expect("encoding length overflow");
		let mut output = vec![0u8; len];
		let written = self.encode_mut(input, &mut output).expect("encoding failed");
		output.truncate(written);
		safety_assert!(output.is_ascii());
		// SAFETY: Ensured by correctness guarantees of encode_mut (and asserted above).
		unsafe { String::from_utf8_unchecked(output) }
	}

	/// Returns the maximum decoded length of an input of length `len`
	///
	/// See [`decode_mut`] for when to use it. In particular, the actual decoded length might be
	/// smaller if the actual input contains padding or ignored characters.
	///
	/// # Panics
	///
	/// May panic if `len` is greater than `usize::MAX / 8`:
	/// - `len <= 536_870_911` when `target_pointer_width = "32"`
	/// - `len <= 2_305843_009213_693951` when `target_pointer_width = "64"`
	///
	/// If you need to decode an input of length greater than this limit (possibly of infinite
	/// length), then you must decode your input chunk by chunk with [`decode_mut`], making sure
	/// that you take into account how many bytes have been read from the input and how many bytes
	/// have been written to the output:
	/// - `Ok(written)` means all bytes have been read and `written` bytes have been written
	/// - `Err(DecodePartial { error, .. })` means an error occurred if `error.kind !=
	///   DecodeKind::Length` or this was the last input chunk
	/// - `Err(DecodePartial { read, written, .. })` means that `read` bytes have been read and
	///   `written` bytes written (the error can be ignored)
	///
	/// Note that this function only _may_ panic in those cases. The function may also return the
	/// correct value in some cases depending on the implementation. In other words, those limits
	/// are the guarantee below which the function will not panic, and not the guarantee above which
	/// the function will panic.
	///
	/// # Errors
	///
	/// Returns an error if `len` is invalid. The error kind is [`Length`] and the [position] is the
	/// greatest valid input length.
	///
	/// [`decode_mut`]: struct.Encoding.html#method.decode_mut
	/// [`Length`]: enum.DecodeKind.html#variant.Length
	/// [position]: struct.DecodeError.html#structfield.position
	pub fn decode_len(&self, len: usize) -> Result<usize, DecodeError> {
		if self.is_arithmetic() {
			// Safe upper bound for arithmetic decoding.
			return Ok(len);
		} else {
			let bit = self.bit();
			let pad_mode = self.pad_mode();
			let (ilen, olen) = dispatch! {
				let bit: usize = bit;
				let pad_mode: PaddingMode = pad_mode;
				decode_wrap_len(bit.val(), pad_mode.val(), len)
			}
			.ok_or(DecodeError {
				position: 0,
				kind: DecodeKind::Overflow,
			})?;
			check!(
				DecodeError {
					position: ilen,
					kind: DecodeKind::Length
				},
				self.has_ignore() || len == ilen
			);
			Ok(olen)
		}
	}

	/// Decodes `input` in `output`
	///
	/// Returns the length of the decoded output. This length may be smaller than the output length
	/// if the input contained padding or ignored characters. The output bytes after the returned
	/// length are not initialized and should not be read.
	///
	/// # Panics
	///
	/// Panics if the `output` length does not match the result of [`decode_len`] for the `input`
	/// length. Also panics if `decode_len` fails for the `input` length.
	///
	/// # Errors
	///
	/// Returns an error if `input` is invalid. See [`decode`] for more details. The are two
	/// differences though:
	///
	/// - [`Length`] may be returned only if the encoding allows ignored characters, because
	///   otherwise this is already checked by [`decode_len`].
	/// - The [`read`] first bytes of the input have been successfully decoded to the [`written`]
	///   first bytes of the output.
	///
	/// # Examples
	///
	/// ```rust
	/// use data_encoding::BASE64;
	/// # let mut buffer = vec![0; 100];
	/// let input = b"SGVsbA==byB3b3JsZA==";
	/// let output = &mut buffer[0 .. BASE64.decode_len(input.len()).unwrap()];
	/// let len = BASE64.decode_mut(input, output).unwrap();
	/// assert_eq!(&output[0 .. len], b"Hello world");
	/// ```
	///
	/// [`decode_len`]: struct.Encoding.html#method.decode_len
	/// [`decode`]: struct.Encoding.html#method.decode
	/// [`Length`]: enum.DecodeKind.html#variant.Length
	/// [`read`]: struct.DecodePartial.html#structfield.read
	/// [`written`]: struct.DecodePartial.html#structfield.written
	#[allow(clippy::cognitive_complexity)]
	pub fn decode_mut(&self, input: &[u8], output: &mut [u8]) -> Result<usize, DecodePartial> {
		if self.is_arithmetic() {
			// Utiliser le décodage arithmétique
			let symbols = self.get_symbols();
			let input_str = core::str::from_utf8(input).map_err(|e| DecodePartial {
				read: e.valid_up_to(),
				written: 0,
				error: DecodeError {
					position: e.valid_up_to(),
					kind: DecodeKind::Symbol,
				},
			})?;
			match crate::arithmetic::decode_to_buffer(symbols, input_str, output) {
				Ok(len) => Ok(len),
				Err(e) => Err(DecodePartial {
					read: e.position,
					written: 0,
					error: e,
				}),
			}
		} else {
			let len = self.decode_len(input.len()).map_err(|e| DecodePartial {
				read: 0,
				written: 0,
				error: e,
			})?;
			check!(
				DecodePartial {
					read: 0,
					written: 0,
					error: DecodeError {
						position: input.len(),
						kind: DecodeKind::BufferTooSmall,
					},
				},
				output.len() == len
			);
			let bit = self.bit();
			let msb = self.msb();
			let pad_mode = self.pad_mode();
			let has_ignore = self.has_ignore();
			let written = dispatch! {
				let bit: usize = bit;
				let msb: bool = msb;
				let pad_mode: PaddingMode = pad_mode;
				let has_ignore: bool = has_ignore;
				decode_wrap_mut(
					bit,
					msb,
					pad_mode,
					has_ignore,
					self.ctb(),
					self.val(),
					self.sym(),
					self.pad(),
					input,
					output,
				)
			}?;
			Ok(written)
		}
	}

	/// Returns decoded `input`
	///
	/// # Errors
	///
	/// Returns an error if `input` is invalid. The error kind can be:
	///
	/// - [`Length`] if the input length is invalid. The [position] is the greatest valid input
	///   length.
	/// - [`Symbol`] if the input contains an invalid character. The [position] is the first invalid
	///   character.
	/// - [`Trailing`] if the input has non-zero trailing bits. This is only possible if the
	///   encoding checks trailing bits. The [position] is the first character containing non-zero
	///   trailing bits.
	/// - [`Padding`] if the input has an invalid padding length. This is only possible if the
	///   encoding uses padding. The [position] is the first padding character of the first padding
	///   of invalid length.
	///
	/// # Examples
	///
	/// ```rust
	/// use data_encoding::BASE64;
	/// assert_eq!(BASE64.decode(b"SGVsbA==byB3b3JsZA==").unwrap(), b"Hello world");
	/// ```
	///
	/// [`Length`]: enum.DecodeKind.html#variant.Length
	/// [`Symbol`]: enum.DecodeKind.html#variant.Symbol
	/// [`Trailing`]: enum.DecodeKind.html#variant.Trailing
	/// [`Padding`]: enum.DecodeKind.html#variant.Padding
	/// [position]: struct.DecodeError.html#structfield.position
	#[cfg(feature = "alloc")]
	pub fn decode(&self, input: &[u8]) -> Result<Vec<u8>, DecodeError> {
		let mut output = vec![0u8; self.decode_len(input.len())?];
		let len = self.decode_mut(input, &mut output).map_err(|partial| partial.error)?;
		output.truncate(len);
		Ok(output)
	}

	/// Returns the bit-width
	#[must_use]
	pub fn bit_width(&self) -> usize {
		if self.is_arithmetic() {
			let base = self.data()[512] as u32;
			if base == 0 {
				return 0;
			}
			(32 - base.leading_zeros()) as usize
		} else {
			self.bit()
		}
	}

	/// Returns whether the encoding is canonical
	///
	/// An encoding is canonical if:
	///
	/// - trailing bits are zero
	/// - no characters are ignored
	/// - no characters are translated
	/// - for padded encodings, there is no trailing padding
	///
	/// In other words, an encoding is canonical if all inputs that could be produced by the
	/// encoding function are successfully decoded, and all other inputs are rejected.
	///
	/// Canonical encodings are useful for hashing.
	///
	/// A non-canonical encoding can be made canonical if:
	///
	/// - trailing bits are checked
	/// - padding is used
	/// - characters are ignored
	/// - characters are translated
	#[must_use]
	pub fn is_canonical(&self) -> bool {
		if !self.ctb() {
			return false;
		}
		let symbols = self.get_symbols();
		let num_symbols = symbols.len();
		if num_symbols == 0 {
			return false;
		}
		let sym = self.sym();
		let val = self.val();
		for i in 0..256 {
			if i >= val.len() || val[i] == INVALID {
				continue;
			}
			if (val[i] as usize) >= num_symbols {
				return false;
			}
			if sym[val[i] as usize] as usize != i {
				return false;
			}
		}
		true
	}

	/// Returns the encoding specification
	#[allow(clippy::missing_panics_doc)] // no panic
	#[cfg(feature = "alloc")]
	#[must_use]
	pub fn specification(&self) -> Specification {
		let mut specification = Specification::new();
		let symbols = self.get_symbols();
		specification.symbols.push_str(core::str::from_utf8(symbols).unwrap_or(""));
		specification.bit_order = if self.msb() {
			MostSignificantFirst
		} else {
			LeastSignificantFirst
		};
		specification.check_trailing_bits = self.ctb();
		if let Some(pad) = self.pad() {
			specification.padding = Some(pad as char);
		}
		for i in 0..128u8 {
			if self.val()[i as usize] != IGNORE {
				continue;
			}
			specification.ignore.push(i as char);
		}
		if let Some((col, end)) = self.wrap() {
			specification.wrap.width = col;
			specification.wrap.separator = core::str::from_utf8(end).unwrap_or("").to_owned();
		}
		let num_symbols = symbols.len();
		for i in 0..128u8 {
			let v = self.val()[i as usize];
			let canonical = if (v as usize) < num_symbols {
				self.sym()[v as usize]
			} else if v == PADDING {
				match self.pad() {
					Some(p) => p,
					None => continue,
				}
			} else {
				continue;
			};
			if i == canonical {
				continue;
			}
			specification.translate.from.push(i as char);
			specification.translate.to.push(canonical as char);
		}
		specification.use_arithmetic = self.is_arithmetic();
		specification
	}

	#[doc(hidden)]
	#[must_use]
	pub const fn internal_new(implementation: &'static [u8]) -> Encoding {
		Encoding(InternalEncoding::Static(implementation))
	}

	#[doc(hidden)]
	#[must_use]
	pub fn internal_implementation(&self) -> &[u8] {
		&*self.0
	}
}

/// Encodes fragmented input to an output
///
/// Use this struct if your input is in several pieces.
#[derive(Debug)]
pub struct Encoder<'a> {
	encoding: &'a Encoding,
	output: &'a mut String,
	buffer: [u8; 8],
	length: usize,
}

impl Drop for Encoder<'_> {
	fn drop(&mut self) {
		self.finalize();
	}
}

impl<'a> Encoder<'a> {
	fn new(encoding: &'a Encoding, output: &'a mut String) -> Encoder<'a> {
		Encoder {
			encoding,
			output,
			buffer: [0u8; 8],
			length: 0,
		}
	}

	/// Appends the encoding of `input` to the output
	pub fn append(&mut self, input: &[u8]) {
		let mut input = input;
		let (ilen, olen) = self.encoding.block_len();
		if self.length > 0 {
			let n = core::cmp::min(ilen - self.length, input.len());
			self.buffer[self.length..self.length + n].copy_from_slice(&input[0..n]);
			self.length += n;
			input = &input[n..];
			if self.length == ilen {
				self.encoding.encode_append(&self.buffer[0..ilen], self.output);
				self.length = 0;
			}
		}
		let n = floor(input.len(), ilen);
		self.encoding.encode_append(&input[0..n], self.output);
		input = &input[n..];
		if !input.is_empty() {
			self.buffer[0..input.len()].copy_from_slice(input);
			self.length = input.len();
		}
	}

	/// Finalizes the encoding
	pub fn finalize(&mut self) {
		if self.length > 0 {
			self.encoding.encode_append(&self.buffer[0..self.length], self.output);
			self.length = 0;
		}
	}
}

/// Displayable version of encoded data
#[derive(Debug)]
pub struct Display<'a> {
	encoding: &'a Encoding,
	input: &'a [u8],
}

impl core::fmt::Display for Display<'_> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let mut buffer = [0u8; 1024];
		for chunk in self.input.chunks(floor(
			1024 / self.encoding.block_len().1 * self.encoding.block_len().0,
			self.encoding.block_len().0,
		)) {
			let len = self
				.encoding
				.encode_mut(chunk, &mut buffer[..self.encoding.encode_len(chunk.len()).unwrap()])
				.unwrap();
			write!(f, "{}", unsafe { core::str::from_utf8_unchecked(&buffer[..len]) })?;
		}
		Ok(())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpecificationErrorImpl {
	BadSize,
	NotAscii,
	Duplicate(u8),
	ExtraPadding,
	WrapLength,
	WrapSeparator,
	WrapWidth(u8),
	FromTo,
	Undefined(u8),
}

/// Specification error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecificationError(SpecificationErrorImpl);

impl core::fmt::Display for SpecificationError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self.0 {
			SpecificationErrorImpl::BadSize => write!(f, "invalid number of symbols"),
			SpecificationErrorImpl::NotAscii => write!(f, "non-ascii character"),
			SpecificationErrorImpl::Duplicate(c) => {
				write!(f, "{:?} has conflicting definitions", c as char)
			}
			SpecificationErrorImpl::ExtraPadding => write!(f, "unnecessary padding"),
			SpecificationErrorImpl::WrapLength => {
				write!(f, "invalid wrap width or separator length")
			}
			SpecificationErrorImpl::WrapSeparator => write!(f, "invalid wrap separator"),
			SpecificationErrorImpl::WrapWidth(n) => write!(f, "wrap width not a multiple of {}", n),
			SpecificationErrorImpl::FromTo => write!(f, "translate from/to length mismatch"),
			SpecificationErrorImpl::Undefined(c) => write!(f, "{:?} is undefined", c as char),
		}
	}
}

impl std::error::Error for SpecificationError {
	fn description(&self) -> &str {
		match self.0 {
			SpecificationErrorImpl::BadSize => "invalid number of symbols",
			SpecificationErrorImpl::NotAscii => "non-ascii character",
			SpecificationErrorImpl::Duplicate(_) => "conflicting definitions",
			SpecificationErrorImpl::ExtraPadding => "unnecessary padding",
			SpecificationErrorImpl::WrapLength => "invalid wrap width or separator length",
			SpecificationErrorImpl::WrapSeparator => "invalid wrap separator",
			SpecificationErrorImpl::WrapWidth(_) => "wrap width mismatch",
			SpecificationErrorImpl::FromTo => "translate mismatch",
			SpecificationErrorImpl::Undefined(_) => "undefined character",
		}
	}
}

impl Specification {
	/// Returns a new empty specification
	#[must_use]
	pub fn new() -> Specification {
		Specification {
			symbols: String::new(),
			bit_order: MostSignificantFirst,
			check_trailing_bits: true,
			padding: None,
			padding_mode: PaddingMode::Standard,
			ignore: String::new(),
			wrap: Wrap {
				width: 0,
				separator: String::new(),
			},
			translate: Translate {
				from: String::new(),
				to: String::new(),
			},

			use_arithmetic: false, // Par défaut, détection automatique
		}
	}

	/// Returns the specified encoding
	///
	/// # Errors
	///
	/// Returns an error if the specification is invalid.

	pub fn encoding(&self) -> Result<Encoding, SpecificationError> {
		let symbols = self.symbols.as_bytes();

		if symbols.is_empty() {
			return Err(SpecificationError(SpecificationErrorImpl::BadSize));
		}

		// Détection automatique si l'encodage arithmétique doit être utilisé
		let use_arithmetic = self.use_arithmetic || ![2, 4, 8, 16, 32, 64].contains(&symbols.len());

		// Pour les bases arithmétiques, valider que les symboles sont ASCII
		if use_arithmetic {
			for &symbol in symbols {
				check!(SpecificationError(SpecificationErrorImpl::NotAscii), symbol < 128);
			}
		}

		let bit: u8 = match symbols.len() {
			2 => 1,
			4 => 2,
			8 => 3,
			16 => 4,
			32 => 5,
			64 => 6,
			_ => {
				if use_arithmetic {
					0
				} else {
					return Err(SpecificationError(SpecificationErrorImpl::BadSize));
				}
			}
		};
		let mut values = [INVALID; 128];
		let set = |v: &mut [u8; 128], i: u8, x: u8| {
			check!(SpecificationError(SpecificationErrorImpl::NotAscii), i < 128);
			if v[i as usize] == x {
				return Ok(());
			}
			check!(
				SpecificationError(SpecificationErrorImpl::Duplicate(i)),
				v[i as usize] == INVALID
			);
			v[i as usize] = x;
			Ok(())
		};
		for (v, symbols) in symbols.iter().enumerate() {
			#[allow(clippy::cast_possible_truncation)] // no truncation
			set(&mut values, *symbols, v as u8)?;
		}
		let msb = self.bit_order == MostSignificantFirst;
		let ctb = self.check_trailing_bits || (!use_arithmetic && 8 % bit == 0);
		let pad = match self.padding {
			None => None,
			Some(pad) => {
				if !use_arithmetic {
					check!(SpecificationError(SpecificationErrorImpl::ExtraPadding), 8 % bit != 0);
				}
				check!(SpecificationError(SpecificationErrorImpl::NotAscii), pad.len_utf8() == 1);
				set(&mut values, pad as u8, PADDING)?;
				Some(pad as u8)
			}
		};
		for i in self.ignore.bytes() {
			set(&mut values, i, IGNORE)?;
		}
		let wrap = if self.wrap.separator.is_empty() || self.wrap.width == 0 {
			None
		} else {
			let col = self.wrap.width;
			let end = self.wrap.separator.as_bytes();
			check!(SpecificationError(SpecificationErrorImpl::WrapSeparator), end.len() <= 15);
			check!(
				SpecificationError(SpecificationErrorImpl::WrapLength),
				col < 256 && end.len() < 256
			);
			#[allow(clippy::cast_possible_truncation)] // no truncation
			let col = col as u8;
			#[allow(clippy::cast_possible_truncation)] // no truncation
			let dec = dec(bit as usize) as u8;
			check!(SpecificationError(SpecificationErrorImpl::WrapWidth(dec)), col % dec == 0);
			for &i in end {
				set(&mut values, i, IGNORE)?;
			}
			Some((col, end))
		};
		let from = self.translate.from.as_bytes();
		let to = self.translate.to.as_bytes();
		check!(SpecificationError(SpecificationErrorImpl::FromTo), from.len() == to.len());
		for i in 0..from.len() {
			check!(SpecificationError(SpecificationErrorImpl::NotAscii), to[i] < 128);
			let v = values[to[i] as usize];
			check!(SpecificationError(SpecificationErrorImpl::Undefined(to[i])), v != INVALID);
			set(&mut values, from[i], v)?;
		}
		let mut encoding = [INVALID; 531];
		if use_arithmetic {
			encoding[0..symbols.len()].copy_from_slice(symbols);
		} else {
			for i in 0..256 {
				encoding[i] = symbols[i % symbols.len()];
			}
		}
		encoding[256..512].copy_from_slice(&[INVALID; 256]);
		encoding[256..384].copy_from_slice(&values);
		if use_arithmetic {
			encoding[512] = symbols.len() as u8;
		} else {
			match pad {
				None => encoding[512] = INVALID,
				Some(pad) => encoding[512] = pad,
			}
		}

		if use_arithmetic {
			encoding[513] = 0x80;
		} else {
			encoding[513] = bit;
		}

		if msb {
			encoding[513] |= 0x08;
		}

		if ctb {
			encoding[513] |= 0x10;
		}

		let mode_bits = match self.padding_mode {
			PaddingMode::None => 0,

			PaddingMode::Standard => 1,

			PaddingMode::PadConcat => 2,

			PaddingMode::PadFinal => 3,
		};

		encoding[513] |= (mode_bits as u8) << 5;

		if let Some((col, end)) = wrap {
			encoding[514] = col;
			encoding[515] = end.len() as u8;
			encoding[516..516 + end.len()].copy_from_slice(end);
		} else {
			encoding[514] = 0;
			encoding[515] = 0;
		}
		Ok(Encoding(InternalEncoding::Owned(encoding)))
	}
}

/// Lowercase hexadecimal encoding
pub const HEXLOWER: Encoding = Encoding::internal_new(data::HEXLOWER_IMPL);

/// Lowercase hexadecimal encoding with case-insensitive decoding
pub const HEXLOWER_PERMISSIVE: Encoding = Encoding::internal_new(data::HEXLOWER_PERMISSIVE_IMPL);

/// Uppercase hexadecimal encoding
pub const HEXUPPER: Encoding = Encoding::internal_new(data::HEXUPPER_IMPL);

/// Uppercase hexadecimal encoding with case-insensitive decoding
pub const HEXUPPER_PERMISSIVE: Encoding = Encoding::internal_new(data::HEXUPPER_PERMISSIVE_IMPL);

/// Padded base32 encoding
pub const BASE32: Encoding = Encoding::internal_new(data::BASE32_IMPL);

/// Unpadded base32 encoding
pub const BASE32_NOPAD: Encoding = Encoding::internal_new(data::BASE32_NOPAD_IMPL);

/// Unpadded base32 encoding with case-insensitive decoding
pub const BASE32_NOPAD_NOCASE: Encoding = Encoding::internal_new(data::BASE32_NOPAD_NOCASE_IMPL);

/// Unpadded base32 encoding with visual error correction during decoding
pub const BASE32_NOPAD_VISUAL: Encoding = Encoding::internal_new(data::BASE32_NOPAD_VISUAL_IMPL);

/// Padded base32hex encoding
pub const BASE32HEX: Encoding = Encoding::internal_new(data::BASE32HEX_IMPL);

/// Unpadded base32hex encoding
pub const BASE32HEX_NOPAD: Encoding = Encoding::internal_new(data::BASE32HEX_NOPAD_IMPL);

/// DNSSEC base32 encoding
pub const BASE32_DNSSEC: Encoding = Encoding::internal_new(data::BASE32_DNSSEC_IMPL);

/// DNSCurve base32 encoding
pub const BASE32_DNSCURVE: Encoding = Encoding::internal_new(data::BASE32_DNSCURVE_IMPL);

/// Padded base64 encoding
pub const BASE64: Encoding = Encoding::internal_new(data::BASE64_IMPL);

/// Unpadded base64 encoding
pub const BASE64_NOPAD: Encoding = Encoding::internal_new(data::BASE64_NOPAD_IMPL);

/// MIME base64 encoding
pub const BASE64_MIME: Encoding = Encoding::internal_new(data::BASE64_MIME_IMPL);

/// MIME base64 encoding without trailing bits check
pub const BASE64_MIME_PERMISSIVE: Encoding =
	Encoding::internal_new(data::BASE64_MIME_PERMISSIVE_IMPL);

/// Padded base64url encoding
pub const BASE64URL: Encoding = Encoding::internal_new(data::BASE64URL_IMPL);

/// Unpadded base64url encoding
pub const BASE64URL_NOPAD: Encoding = Encoding::internal_new(data::BASE64URL_NOPAD_IMPL);

/// Base58 encoding (Bitcoin alphabet)
pub const BASE58: Encoding = Encoding::internal_new(data::BASE58_IMPL);

/// Base62 encoding
pub const BASE62: Encoding = Encoding::internal_new(data::BASE62_IMPL);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_base58_roundtrip() {
		let input = b"Hello World!";
		let encoded = BASE58.encode(input);
		let decoded = BASE58
			.decode(encoded.as_bytes())
			.map_err(|e| {
				panic!(
					"Decode Base58 failed at position {} with kind {:?}. Encoded: {}",
					e.position, e.kind, encoded
				);
			})
			.unwrap();
		assert_eq!(input, decoded.as_slice());
	}

	#[test]
	fn test_base62_roundtrip() {
		let input = b"Hello World!";
		let encoded = BASE62.encode(input);
		let decoded = BASE62
			.decode(encoded.as_bytes())
			.map_err(|e| {
				panic!(
					"Decode Base62 failed at position {} with kind {:?}. Encoded: {}",
					e.position, e.kind, encoded
				);
			})
			.unwrap();
		assert_eq!(input, decoded.as_slice());
	}

	#[test]
	fn test_base58_leaders() {
		let input = b"\0\0\0Hello";
		let encoded = BASE58.encode(input);
		let decoded = BASE58
			.decode(encoded.as_bytes())
			.map_err(|e| {
				panic!("Decode failed at position {} with kind {:?}", e.position, e.kind);
			})
			.unwrap();
		assert_eq!(input, decoded.as_slice());
	}
}
