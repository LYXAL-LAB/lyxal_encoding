# Lyxal Base256Emoji

A high-performance, "Google-Grade" Base256Emoji encoding engine designed for Lyxal and SurrealDB.

## Features

- **Hardened**: Zero-panic and zero-unsafe implementation.
- **Embedded Ready**: Supports `no_std` environments.
- **Performance**: Optional `no_alloc` API with direct buffer-to-buffer operations.
- **Reliable**: Verified with unit tests, property-based tests (fuzzing), and fixed fixtures.

## Usage

### Basic Encoding/Decoding (Alloc)

```rust
use lyxal_base256emoji::{encode, decode};

let data = b"Hello Lyxal!";
let encoded = encode(data);
println!("Encoded: {}", encoded);

let decoded = decode(&encoded).unwrap();
assert_eq!(data, decoded.as_slice());
```

### Buffer-to-Buffer (No-Alloc)

```rust
use lyxal_base256emoji::{encode_to_buffer, decode_to_buffer};

let data = b"SurrealDB";
let mut enc_buf = [0u8; 128];
let len = encode_to_buffer(data, &mut enc_buf).unwrap();

let mut dec_buf = [0u8; 128];
let d_len = decode_to_buffer(std::str::from_utf8(&enc_buf[..len]).unwrap(), &mut dec_buf).unwrap();
```

## Alphabet

The alphabet consists of 256 unique emojis, mapping each byte value (0-255) to a single emoji character.
