# Lyxal Encoding

A modular, high-performance Rust workspace for various encoding standards. This repository serves as the foundational encoding layer for the [Lyxal Ecosystem](https://github.com/LYXAL-LAB).

## 📦 Modules

This workspace contains several crates dedicated to specific encoding schemes:

- **`multibase`**: Implementation of the Multibase specification (self-identifying base encodings).
- **`base-x`**: Fast and efficient base encoding/decoding.
- **`base256emoji`**: A visual encoding scheme using a 256-emoji alphabet.
- **`base45`**: Implementation of the Base45 encoding scheme, often used in QR codes.
- **`data-encoding`**: Efficient data encoding utilities.

## 🚀 Usage

Add the specific crate you need to your `Cargo.toml`:

```toml
[dependencies]
# Example: using the multibase crate
multibase = { git = "https://github.com/LYXAL-LAB/lyxal-encoding" }
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as below, without any additional terms or conditions.

## 📄 License

This project is licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🌟 Acknowledgements & Credits

This project includes code forked from or heavily inspired by the following open-source projects. We are grateful to the original authors for their excellent work:

*   **[multibase](https://github.com/multiformats/rust-multibase)** by the Multiformats Community.
*   **[base-x](https://github.com/orph/rust-base-x)** by Orph.
*   **[base256emoji](https://github.com/Jorropo/base256emoji)** by Jorropo.
*   **[base45](https://github.com/opendevtools/base45)** by OpenDevTools.

Modifications have been made to integrate these libraries into the Lyxal ecosystem.
