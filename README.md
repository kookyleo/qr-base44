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

### Basic byte-pair encoding

```rust
use qr_base44::{encode, decode};

let data: &[u8] = &[0x01, 0x02, 0xFF];
let s = encode(data);
let back = decode(&s).unwrap();
assert_eq!(back, data);
```

### Optimal bit-level encoding (new!)

For fixed bit lengths (arbitrary size), use `encode_bits` for optimal space efficiency:

```rust
use qr_base44::{encode_bits, decode_bits};

// Encode 103 bits (e.g., compressed UUID)
let data = vec![0x12, 0x34, /* ... 13 bytes total */];
let s = encode_bits(103, &data);  // Always 19 chars for 103 bits
let back = decode_bits(103, &s).unwrap();
assert_eq!(back, data);

// Compare efficiency:
// - encode_bits(103, &data) → 19 chars (optimal)
// - encode(&data)           → 20 chars (byte-pair)
// Savings: 5% for 103-bit data
```

## Features

- **URL-safe**: Unlike Base45, Base44 removes the space character which can cause issues in URLs
- **QR-compatible**: Uses a subset of QR Code alphanumeric mode characters
- **Dual encoding modes**:
  - **Byte-pair encoding**: General-purpose, ~1.5x size overhead
  - **Optimal bit encoding**: For fixed bit lengths (arbitrary size, uses BigInt), achieves theoretical minimum character count
- **Error handling**: Validates input and reports invalid characters, dangling groups, and overflow

### Encoding Comparison

For fixed-bit-length data, `encode_bits` achieves optimal compression:

| Bits | Bytes | `encode` (byte-pair) | `encode_bits` (optimal) | Savings | Use Case |
|------|-------|---------------------|------------------------|---------|----------|
| 103  | 13    | 20 chars            | 19 chars               | 5.0%    | Compressed UUID (qr-url) |
| 104  | 13    | 20 chars            | 20 chars               | 0%      | - |
| 128  | 16    | 24 chars            | 24 chars               | 0%      | UUID, AES-128 key |
| 256  | 32    | 48 chars            | 47 chars               | 2.1%    | SHA-256 hash, AES-256 key |
| 512  | 64    | 96 chars            | 94 chars               | 2.1%    | SHA-512 hash |

**When to use `encode_bits`**:
- ✅ Bit count known and doesn't align with byte boundaries (103, 256, 512 bits)
- ✅ Need guaranteed fixed output length
- ❌ Arbitrary-length data or byte-aligned sizes (use `encode` instead)

## Notes

- **MSRV**: 1.85+ (Rust 2024 edition requirement)
- This crate intentionally encodes/decodes arbitrary bytes, not UTF-8 text. If you have a text string, pass its bytes explicitly.
- Error types include: invalid characters, dangling final character, and numeric overflow.
- Compared to Base45, Base44 is more suitable for use in URLs and QR codes by removing the space character.

## Documentation

- [中文文档 (Chinese Documentation)](README.zh.md)

## License

Apache-2.0
