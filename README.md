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

**Performance optimization:** `encode_bits`/`decode_bits` automatically use native integer types (u64 for ≤64 bits, u128 for ≤128 bits) instead of BigInt for better performance on common bit sizes.

## Features

- **URL-safe**: Unlike Base45, Base44 removes the space character which can cause issues in URLs
- **QR-compatible**: Uses a subset of QR Code alphanumeric mode characters
- **Dual encoding modes**:
  - **Byte-pair encoding** (`encode`/`decode`): Fast, general-purpose encoding with simple integer operations. Best for arbitrary-length byte data.
  - **Optimal bit encoding** (`encode_bits`/`decode_bits`): Space-optimal encoding for fixed bit lengths using BigInt. Best when every character counts and bit length is known.
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

### Choosing the Right API

**Use `encode`/`decode` (byte-pair) when:**
- ✅ Working with arbitrary-length byte data
- ✅ Performance is important (no BigInt overhead)
- ✅ Simplicity matters (no need to specify bit count)
- ✅ Want minimal dependencies

**Use `encode_bits`/`decode_bits` (optimal) when:**
- ✅ Bit count is known and fixed
- ✅ Bit count doesn't align with byte boundaries (e.g., 103, 256, 512 bits)
- ✅ Every character counts and space efficiency is critical
- ✅ Need guaranteed fixed output length for specific bit counts

**Performance vs Space Trade-off:**
- `encode` is **significantly faster** (simple integer ops vs BigInt)
- `encode_bits` saves **at most 5% space** for non-byte-aligned bit counts
- For byte-aligned data (8, 16, 24, 128 bits), both produce the **same output length**

## Notes

- **MSRV**: 1.85+ (Rust 2024 edition requirement)
- This crate intentionally encodes/decodes arbitrary bytes, not UTF-8 text. If you have a text string, pass its bytes explicitly.
- Error types include: invalid characters, dangling final character, and numeric overflow.
- Compared to Base45, Base44 is more suitable for use in URLs and QR codes by removing the space character.

## Documentation

- [中文文档 (Chinese Documentation)](README.zh.md)

## License

Apache-2.0
