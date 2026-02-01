#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

use core::fmt;

/// Errors that can occur during encoding or decoding.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The character is not part of the base256emoji alphabet.
    InvalidCharacter(char, usize),
    /// The provided output buffer is too small to hold the result.
    BufferTooSmall,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCharacter(c, i) => write!(f, "Character '{}' at index {} is not in alphabet", c, i),
            Error::BufferTooSmall => write!(f, "Output buffer is too small"),
        }
    }
}

/// The base256emoji alphabet consisting of 256 unique emojis.
pub const ALPHABET: [char; 256] = [
    '🚀', '🪐', '☄', '🛰', '🌌', '🌑', '🌒', '🌓', '🌔', '🌕', '🌖', '🌗', '🌘', '🌍', '🌏', '🌎', '🐉', '☀', '💻', '🖥', '💾', '💿', '😂', '❤', '😍', '🤣', '😊', '🙏', '💕', '😭', '😘', '👍',
    '😅', '👏', '😁', '🔥', '🥰', '💔', '💖', '💙', '😢', '🤔', '😆', '🙄', '💪', '😉', '☺', '👌', '🤗', '💜', '😔', '😎', '😇', '🌹', '🤦', '🎉', '💞', '✌', '✨', '🤷', '😱', '😌', '🌸', '🙌',
    '😋', '💗', '💚', '😏', '💛', '🙂', '💓', '🤩', '😄', '😀', '🖤', '😃', '💯', '🙈', '👇', '🎶', '😒', '🤭', '❣', '😜', '💋', '👀', '😪', '😑', '💥', '🙋', '😞', '😩', '😡', '🤪', '👊', '🥳',
    '😥', '🤤', '👉', '💃', '😳', '✋', '😚', '😝', '😴', '🌟', '😬', '🙃', '🍀', '🌷', '😻', '😓', '⭐', '✅', '🥺', '🌈', '😈', '🤘', '💦', '✔', '😣', '🏃', '💐', '☹', '🎊', '💘', '😠', '☝',
    '😕', '🌺', '🎂', '🌻', '😐', '🖕', '💝', '🙊', '😹', '🗣', '💫', '💀', '👑', '🎵', '🤞', '😛', '🔴', '😤', '🌼', '😫', '⚽', '🤙', '☕', '🏆', '🤫', '👈', '😮', '🙆', '🍻', '🍃', '🐶', '💁',
    '😲', '🌿', '🧡', '🎁', '⚡', '🌞', '🎈', '❌', '✊', '👋', '😰', '🤨', '😶', '🤝', '🚶', '💰', '🍓', '💢', '🤟', '🙁', '🚨', '💨', '🤬', '✈', '🎀', '🍺', '🤓', '😙', '💟', '🌱', '😖', '👶',
    '🥴', '▶', '➡', '❓', '💎', '💸', '⬇', '😨', '🌚', '🦋', '😷', '🕺', '⚠', '🙅', '😟', '😵', '👎', '🤲', '🤠', '🤧', '📌', '🔵', '💅', '🧐', '🐾', '🍒', '😗', '🤑', '🌊', '🤯', '🐷', '☎',
    '💧', '😯', '💆', '👆', '🎤', '🙇', '🍑', '❄', '🌴', '💣', '🐸', '💌', '📍', '🥀', '🤢', '👅', '💡', '💩', '👐', '📸', '👻', '🤐', '🤮', '🎼', '🥵', '🚩', '🍎', '🍊', '👼', '💍', '📣', '🥂',
];

/// Encodes a byte slice into an existing UTF-8 buffer.
/// Returns the number of bytes written to the output.
///
/// # Errors
/// Returns `Error::BufferTooSmall` if the output buffer cannot hold the encoded string.
pub fn encode_to_buffer(input: &[u8], output: &mut [u8]) -> Result<usize, Error> {
    let mut total_len = 0;
    for &byte in input {
        let emoji = ALPHABET[byte as usize];
        let len = emoji.len_utf8();
        if total_len + len > output.len() {
            return Err(Error::BufferTooSmall);
        }
        emoji.encode_utf8(&mut output[total_len..]);
        total_len += len;
    }
    Ok(total_len)
}

/// Encodes a byte slice into a `String`.
/// 
/// Each byte is mapped to its corresponding emoji in the alphabet.
#[cfg(feature = "alloc")]
pub fn encode(input: &[u8]) -> String {
    let mut output = String::new();
    for &byte in input {
        output.push(ALPHABET[byte as usize]);
    }
    output
}

/// Helper to find index of an emoji.
fn get_index(c: char) -> Option<u8> {
    for (i, &emoji) in ALPHABET.iter().enumerate() {
        if emoji == c {
            return Some(i as u8);
        }
    }
    None
}

/// Decodes a base256emoji string into an existing buffer.
/// Returns the number of bytes written to the output.
///
/// # Errors
/// - Returns `Error::InvalidCharacter` if a character in the input is not in the alphabet.
/// - Returns `Error::BufferTooSmall` if the output buffer is too small.
pub fn decode_to_buffer(input: &str, output: &mut [u8]) -> Result<usize, Error> {
    let mut i = 0;
    for (idx, c) in input.chars().enumerate() {
        if i >= output.len() {
            return Err(Error::BufferTooSmall);
        }
        output[i] = get_index(c).ok_or(Error::InvalidCharacter(c, idx))?;
        i += 1;
    }
    Ok(i)
}

/// Decodes a base256emoji string into a `Vec<u8>`.
///
/// # Errors
/// Returns `Error::InvalidCharacter` if a character in the input is not in the alphabet.
#[cfg(feature = "alloc")]
pub fn decode(input: &str) -> Result<Vec<u8>, Error> {
    let mut output = Vec::with_capacity(input.chars().count());
    for (idx, c) in input.chars().enumerate() {
        output.push(get_index(c).ok_or(Error::InvalidCharacter(c, idx))?);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let input = b"Hello Lyxal!";
        let encoded = encode(input);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(input, decoded.as_slice());
    }

    #[test]
    fn test_fixtures() {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Fixture {
            input_hex: String,
            expected_emoji: String,
        }

        let fixtures_path = "fixtures/emoji_fixtures.json";
        let content = std::fs::read_to_string(fixtures_path).expect("Failed to read fixtures");
        let fixtures: Vec<Fixture> = serde_json::from_str(&content).expect("Failed to parse fixtures");

        for f in fixtures {
            let input = hex::decode(&f.input_hex).expect("Invalid hex in fixture");
            let encoded = encode(&input);
            assert_eq!(encoded, f.expected_emoji, "Match failed for hex {}", f.input_hex);
            
            let decoded = decode(&f.expected_emoji).expect("Decode failed for emoji");
            assert_eq!(decoded, input, "Roundtrip failed for emoji {}", f.expected_emoji);
        }
    }

    #[cfg(feature = "alloc")]
    mod property_tests {
        use super::super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10000))]
            #[test]
            fn roundtrip(ref input in any::<Vec<u8>>()) {
                let encoded = encode(input);
                let decoded = decode(&encoded).expect("Decode failed");
                prop_assert_eq!(input, &decoded);
            }
        }
    }

    #[test]
    fn test_buffer_api() {
        let input = b"SurrealDB";
        let mut enc_buf = [0u8; 128];
        let len = encode_to_buffer(input, &mut enc_buf).unwrap();
        let encoded_str = core::str::from_utf8(&enc_buf[..len]).unwrap();
        
        let mut dec_buf = [0u8; 128];
        let d_len = decode_to_buffer(encoded_str, &mut dec_buf).unwrap();
        assert_eq!(input, &dec_buf[..d_len]);
    }
}
