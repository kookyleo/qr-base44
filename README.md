# qr-base44

[![Crates.io](https://img.shields.io/crates/v/qr-base44.svg)](https://crates.io/crates/qr-base44)
[![Docs.rs](https://docs.rs/qr-base44/badge.svg)](https://docs.rs/qr-base44)

Base44 encoder/decoder for arbitrary bytes using a URL-safe QR-compatible alphabet.

- Encoding scheme: 2 bytes -> 3 chars; 1 byte -> 2 chars
- Alphabet: `0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ$%*+-./:` (44 characters)
- URL-safe: excludes space from Base45 (RFC 9285)
- Public API: byte-oriented operations, no string reinterpretation

## Usage

```bash
cargo add qr-base44
```

```rust
use qr_base44::{encode, decode};

let data: &[u8] = &[0x01, 0x02, 0xFF];
let s = encode(data);
let back = decode(&s).unwrap();
assert_eq!(back, data);
```

## Features

- **URL-safe**: Unlike Base45, Base44 removes the space character which can cause issues in URLs
- **QR-compatible**: Uses a subset of QR Code alphanumeric mode characters
- **Efficient**: Compact binary encoding with ~1.5x size overhead
- **Error handling**: Validates input and reports invalid characters, dangling groups, and overflow

## Notes

- **MSRV**: 1.85+ (Rust 2024 edition requirement)
- This crate intentionally encodes/decodes arbitrary bytes, not UTF-8 text. If you have a text string, pass its bytes explicitly.
- Error types include: invalid characters, dangling final character, and numeric overflow.
- Compared to Base45, Base44 is more suitable for use in URLs and QR codes by removing the space character.

## Documentation

- [中文文档 (Chinese Documentation)](README.zh.md)

## License

Apache-2.0
