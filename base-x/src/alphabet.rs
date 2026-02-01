#[cfg(feature = "alloc")]
use crate::{String, Vec};
use crate::{DecodeError, EncodeError};
use crate::decoder::*;

pub trait Alphabet {
    #[cfg(feature = "alloc")]
    fn encode(self, input: &[u8]) -> Result<String, EncodeError>;

    #[cfg(feature = "alloc")]
    fn decode(self, input: &str) -> Result<Vec<u8>, DecodeError>;

    fn encode_to_buffer(self, input: &[u8], output: &mut [u8]) -> Result<usize, EncodeError>;

    fn decode_to_buffer(self, input: &str, output: &mut [u8]) -> Result<usize, DecodeError>;
}

impl Alphabet for &[u8] {
    #[inline(always)]
    #[cfg(feature = "alloc")]
    fn encode(self, input: &[u8]) -> Result<String, EncodeError> {
        if !self.is_ascii() {
            return Err(EncodeError::InvalidAlphabet);
        }

        let mut out = crate::encoder::encode(self, input);
        out.reverse();
        // C'est sûr car l'alphabet est ASCII et les calculs bigint préservent les bytes.
        Ok(String::from_utf8(out).expect("Alphabet checked ASCII; indices always within bounds; qed"))
    }

    #[inline(always)]
    #[cfg(feature = "alloc")]
    fn decode(self, input: &str) -> Result<Vec<u8>, DecodeError> {
        U8Decoder::new(self).decode(input)
    }

    fn encode_to_buffer(self, input: &[u8], output: &mut [u8]) -> Result<usize, EncodeError> {
        if !self.is_ascii() {
            return Err(EncodeError::InvalidAlphabet);
        }
        crate::encoder::encode_to_buffer(self, input, output)
    }

    fn decode_to_buffer(self, input: &str, output: &mut [u8]) -> Result<usize, DecodeError> {
        U8Decoder::new(self).decode_to_buffer(input, output)
    }
}

impl Alphabet for &str {
    #[inline(always)]
    #[cfg(feature = "alloc")]
    fn encode(self, input: &[u8]) -> Result<String, EncodeError> {
        if self.is_ascii() {
            let mut out = crate::encoder::encode(self.as_bytes(), input);
            out.reverse();
            Ok(String::from_utf8(out).expect("Alphabet checked ASCII; indices always within bounds; qed"))
        } else {
            let alphabet: Vec<char> = self.chars().collect();
            let out = crate::encoder::encode(&alphabet, input);
            Ok(out.iter().rev().collect())
        }
    }

    #[inline(always)]
    #[cfg(feature = "alloc")]
    fn decode(self, input: &str) -> Result<Vec<u8>, DecodeError> {
        if self.is_ascii() {
            U8Decoder::new(self.as_bytes()).decode(input)
        } else {
            let alphabet: Vec<char> = self.chars().collect();
            CharDecoder(&alphabet).decode(input)
        }
    }

    fn encode_to_buffer(self, input: &[u8], output: &mut [u8]) -> Result<usize, EncodeError> {
        if self.is_ascii() {
            crate::encoder::encode_to_buffer(self.as_bytes(), input, output)
        } else {
            // No-alloc Unicode support non implémentée pour éviter la complexité de gestion des multi-octets UTF-8
            // sans buffer dynamique.
            Err(EncodeError::InvalidAlphabet)
        }
    }

    fn decode_to_buffer(self, input: &str, output: &mut [u8]) -> Result<usize, DecodeError> {
        if self.is_ascii() {
            U8Decoder::new(self.as_bytes()).decode_to_buffer(input, output)
        } else {
            Err(DecodeError)
        }
    }
}
