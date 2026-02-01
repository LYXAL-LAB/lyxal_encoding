# Lyxal Encoding

A modular, high-performance Rust workspace for various encoding standards. This repository serves as the foundational encoding layer for the [Lyxal Ecosystem](https://github.com/lyxal).

## 📦 Modules

This workspace contains several crates dedicated to specific encoding schemes:

- **`multibase`**: Implementation of the Multibase specification (self-identifying base encodings).
- **`base-x`**: Fast and efficient base encoding/decoding.
- **`base256emoji`**: A visual encoding scheme using a 256-emoji alphabet.
- **`data-encoding`**: Efficient data encoding utilities.

## 🚀 Usage

Add the specific crate you need to your `Cargo.toml`:

```toml
[dependencies]
# Example: using the multibase crate
multibase = { git = "https://github.com/lyxal/lyxal-encoding" }
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as below, without any additional terms or conditions.

## 📄 License

This project is licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.