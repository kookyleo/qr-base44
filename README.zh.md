# qr-base44

[![Crates.io](https://img.shields.io/crates/v/qr-base44.svg)](https://crates.io/crates/qr-base44)
[![Docs.rs](https://docs.rs/qr-base44/badge.svg)](https://docs.rs/qr-base44)

Base44 编码/解码库，使用 URL 安全的 QR 兼容字符集对任意字节进行编码。

- 编码方式：2 字节 -> 3 字符；1 字节 -> 2 字符
- 字符集：`0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ$%*+-./:` (44 个字符)
- URL 安全：去除了 Base45 (RFC 9285) 中的空格
- 公共 API：面向字节操作，无字符串重新解释

## 使用方法

```bash
cargo add qr-base44
```

### 基础字节对编码

```rust
use qr_base44::{encode, decode};

let data: &[u8] = &[0x01, 0x02, 0xFF];
let s = encode(data);
let back = decode(&s).unwrap();
assert_eq!(back, data);
```

### 最优比特级编码（新功能！）

对于固定比特长度（任意大小）的数据，使用 `encode_bits` 获得最优空间效率：

```rust
use qr_base44::{encode_bits, decode_bits};

// 编码 103 bits（例如：压缩的 UUID）
let data = vec![0x12, 0x34, /* ... 总共 13 bytes */];
let s = encode_bits(103, &data);  // 103 bits 总是编码为 19 chars
let back = decode_bits(103, &s).unwrap();
assert_eq!(back, data);

// 效率对比：
// - encode_bits(103, &data) → 19 chars (最优)
// - encode(&data)           → 20 chars (字节对)
// 节省：103-bit 数据节省 5%
```

**性能优化：** `encode_bits`/`decode_bits` 会自动针对常见比特大小使用原生整数类型（≤64 bits 使用 u64，≤128 bits 使用 u128）而非 BigInt，以获得更好的性能。

## 特性

- **URL 安全**：与 Base45 不同，Base44 移除了可能在 URL 中造成问题的空格字符
- **QR 兼容**：使用 QR 码字母数字模式字符的子集
- **双重编码模式**：
  - **字节对编码**（`encode`/`decode`）：快速、通用编码，使用简单整数运算。最适合任意长度的字节数据。
  - **最优比特编码**（`encode_bits`/`decode_bits`）：针对固定比特长度的空间最优编码，使用 BigInt。最适合字符数敏感且已知比特长度的场景。
- **错误处理**：验证输入并报告无效字符、悬空分组和溢出

### 编码对比

对于固定比特长度的数据，`encode_bits` 实现最优压缩：

| 比特数 | 字节数 | `encode`（字节对）| `encode_bits`（最优）| 节省 | 使用场景 |
|--------|--------|------------------|---------------------|------|----------|
| 103    | 13     | 20 chars         | 19 chars            | 5.0% | 压缩 UUID（qr-url）|
| 104    | 13     | 20 chars         | 20 chars            | 0%   | - |
| 128    | 16     | 24 chars         | 24 chars            | 0%   | UUID、AES-128 密钥 |
| 256    | 32     | 48 chars         | 47 chars            | 2.1% | SHA-256 哈希、AES-256 密钥 |
| 512    | 64     | 96 chars         | 94 chars            | 2.1% | SHA-512 哈希 |

### 选择正确的 API

**使用 `encode`/`decode`（字节对）的场景：**
- ✅ 处理任意长度的字节数据
- ✅ 性能很重要（无 BigInt 开销）
- ✅ 追求简单性（无需指定比特数）
- ✅ 希望最小化依赖

**使用 `encode_bits`/`decode_bits`（最优）的场景：**
- ✅ 比特数已知且固定
- ✅ 比特数不与字节边界对齐（例如：103、256、512 bits）
- ✅ 字符数敏感，空间效率至关重要
- ✅ 需要对特定比特数保证固定输出长度

**性能 vs 空间权衡：**
- `encode` **速度显著更快**（简单整数运算 vs BigInt）
- `encode_bits` 对非字节对齐的比特数**最多节省 5% 空间**
- 对于字节对齐的数据（8、16、24、128 bits），两者产生**相同的输出长度**

## 说明

- **MSRV**: 1.85+（Rust 2024 edition 要求）
- 此 crate 专门用于编码/解码任意字节，而非 UTF-8 文本。如果有文本字符串，请显式传递其字节表示。
- 错误类型包括：无效字符、悬空最终字符和数值溢出。
- 相比 Base45，Base44 移除了 URL 中可能造成问题的空格字符，更适合在 URL 和 QR 码中使用。

## 文档

- [English Documentation](README.md)

## 许可证

Apache-2.0
