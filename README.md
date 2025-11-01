# qr-base43

[![Crates.io](https://img.shields.io/crates/v/qr-base43.svg)](https://crates.io/crates/qr-base43)
[![Docs.rs](https://docs.rs/qr-base43/badge.svg)](https://docs.rs/qr-base43)

Base43 编码/解码库，使用 URL 安全的 QR 兼容字符集对任意字节进行编码。

- 编码方式：2 字节 -> 3 字符；1 字节 -> 2 字符
- 字符集：`0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ%*+-./:` (43 个字符)
- URL 安全：去除了 Base45 (RFC 9285) 中的空格和 `$` 符号
- 公共 API：面向字节操作，无字符串重新解释

## 使用方法

```bash
cargo add qr-base43
```

```rust
use qr_base43::{encode, decode};

let data: &[u8] = &[0x01, 0x02, 0xFF];
let s = encode(data);
let back = decode(&s).unwrap();
assert_eq!(back, data);
```

## 说明
- MSRV: 1.85+ (Rust 2024 edition 要求)
- 此 crate 专门用于编码/解码任意字节，而非 UTF-8 文本。如果有文本字符串，请显式传递其字节表示。
- 错误类型包括：无效字符、悬空最终字符和数值溢出。
- 相比 Base45，Base43 移除了 URL 中可能造成问题的字符（空格和 `$`），更适合在 URL 和 QR 码中使用。

## 许可证
Apache-2.0
