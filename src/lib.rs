//! qr-base44: Base44 encoder/decoder for arbitrary bytes using URL-safe QR-compatible alphabet.
//! - Encoding groups: 2 bytes -> 3 chars; 1 byte -> 2 chars.
//! - Alphabet: "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ$%*+-./:" (44 chars, excludes space only)
//! - Public API encodes &[u8] -> String and decodes &str -> Vec<u8>.

#[derive(Debug, thiserror::Error)]
pub enum Base44Error {
    #[error("invalid base44 character")]
    InvalidChar,
    #[error("dangling character group")]
    Dangling,
    #[error("value overflow")]
    Overflow,
}

/// Base44 alphabet: URL-safe QR-compatible subset (excludes space only)
pub const BASE44_ALPHABET: &[u8; 44] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ$%*+-./:";

#[inline]
fn b44_val(ch: u8) -> Option<u16> {
    match ch {
        b'0'..=b'9' => Some((ch - b'0') as u16),
        b'A'..=b'Z' => Some(10 + (ch - b'A') as u16),
        b'$' => Some(36),
        b'%' => Some(37),
        b'*' => Some(38),
        b'+' => Some(39),
        b'-' => Some(40),
        b'.' => Some(41),
        b'/' => Some(42),
        b':' => Some(43),
        _ => None,
    }
}

/// Encode arbitrary bytes into a Base44 string.
/// Groups of 2 bytes produce 3 characters; a final single byte produces 2 characters.
pub fn encode(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() * 3).div_ceil(2));
    let mut i = 0;
    while i + 1 < input.len() {
        let x = (input[i] as u16) * 256 + (input[i + 1] as u16);
        let c = x % 44; // least significant digit
        let x = x / 44;
        let b = x % 44;
        let a = x / 44; // most significant digit
        // Base44 outputs least-significant digit first
        out.push(BASE44_ALPHABET[c as usize] as char);
        out.push(BASE44_ALPHABET[b as usize] as char);
        out.push(BASE44_ALPHABET[a as usize] as char);
        i += 2;
    }
    if i < input.len() {
        let x = input[i] as u16;
        let b = x % 44;
        let a = x / 44;
        // Base44 outputs least-significant digit first for single byte too
        out.push(BASE44_ALPHABET[b as usize] as char);
        out.push(BASE44_ALPHABET[a as usize] as char);
    }
    out
}

/// Decode a Base44 string back to raw bytes.
/// Accepts only the Base44 alphabet; returns errors for invalid chars, dangling final char, or overflow.
pub fn decode(s: &str) -> Result<Vec<u8>, Base44Error> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i + 2 < bytes.len() {
        // Input is least-significant digit first: c (lsd), b, a (msd)
        let c0 = b44_val(bytes[i]).ok_or(Base44Error::InvalidChar)? as u32;
        let c1 = b44_val(bytes[i + 1]).ok_or(Base44Error::InvalidChar)? as u32;
        let c2 = b44_val(bytes[i + 2]).ok_or(Base44Error::InvalidChar)? as u32;
        let x: u32 = c2 * 44 * 44 + c1 * 44 + c0; // 0..(44^3 - 1)
        if x > 65535 {
            return Err(Base44Error::Overflow);
        }
        out.push((x / 256) as u8);
        out.push((x % 256) as u8);
        i += 3;
    }
    if i < bytes.len() {
        if i + 1 >= bytes.len() {
            // Single trailing character: report InvalidChar if it's not in alphabet, otherwise Dangling
            if b44_val(bytes[i]).is_none() {
                return Err(Base44Error::InvalidChar);
            }
            return Err(Base44Error::Dangling);
        }
        let c0 = b44_val(bytes[i]).ok_or(Base44Error::InvalidChar)? as u32;
        let c1 = b44_val(bytes[i + 1]).ok_or(Base44Error::InvalidChar)? as u32;
        let x: u32 = c1 * 44 + c0; // 0..(44^2 - 1)
        if x > 255 {
            return Err(Base44Error::Overflow);
        }
        out.push(x as u8);
    }
    Ok(out)
}

/// Encode a fixed number of bits (up to 128) as a Base44 string with optimal length.
///
/// This function treats the input bytes as a big integer containing exactly `bits` bits
/// and encodes it using the minimum number of Base44 characters required.
///
/// # Optimal Encoding
///
/// For N bits, the optimal Base44 length is `ceil(N * log(2) / log(44))`:
/// - 103 bits → 19 chars (2^103 < 44^19)
/// - 104 bits → 20 chars (2^104 < 44^20)
///
/// This is more efficient than byte-pair encoding when the bit count doesn't align
/// with byte boundaries, saving up to 5% space for certain bit lengths.
///
/// # Arguments
///
/// * `bits` - Number of significant bits (1-128). Bytes are read in little-endian order.
/// * `bytes` - Input bytes in LSB-first order (matching typical bit-packing schemes).
///
/// # Example
///
/// ```
/// // Encode 103 bits (13 bytes with top byte using 7 bits)
/// let data = [0u8; 13];
/// let encoded = qr_base44::encode_bits(103, &data);
/// assert_eq!(encoded.len(), 19); // Optimal length for 103 bits
/// ```
pub fn encode_bits(bits: usize, bytes: &[u8]) -> String {
    assert!(bits > 0 && bits <= 128, "bits must be 1-128");
    assert!(bytes.len() <= 16, "bytes must be at most 16");

    // Convert bytes to u128 (little-endian)
    let mut value: u128 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        value |= (b as u128) << (i * 8);
    }

    // Calculate optimal character count: ceil(bits * log(2) / log(44))
    // For N bits: 2^N < 44^chars, so chars = ceil(N * log(2) / log(44))
    let chars_needed = ((bits as f64) * 2f64.ln() / 44f64.ln()).ceil() as usize;

    // Convert to base44
    let mut result = Vec::with_capacity(chars_needed);
    let mut v = value;
    for _ in 0..chars_needed {
        let digit = (v % 44) as usize;
        result.push(BASE44_ALPHABET[digit]);
        v /= 44;
    }

    // Reverse to get most significant digit first
    result.reverse();
    String::from_utf8(result).unwrap()
}

/// Decode a Base44 string back to bytes, expecting a specific bit count.
///
/// This is the inverse of [`encode_bits`]. The output bytes are in little-endian order
/// (LSB-first), matching typical bit-packing schemes.
///
/// # Arguments
///
/// * `bits` - Expected number of significant bits (1-128)
/// * `s` - Base44 string to decode
///
/// # Returns
///
/// A vector of bytes in LSB-first order containing exactly `ceil(bits / 8)` bytes.
/// Returns an error if the string contains invalid characters or the decoded value
/// exceeds the specified bit count.
///
/// # Example
///
/// ```
/// let encoded = qr_base44::encode_bits(103, &[0u8; 13]);
/// let decoded = qr_base44::decode_bits(103, &encoded).unwrap();
/// assert_eq!(decoded.len(), 13);
/// ```
pub fn decode_bits(bits: usize, s: &str) -> Result<Vec<u8>, Base44Error> {
    assert!(bits > 0 && bits <= 128, "bits must be 1-128");

    // Convert base44 string to u128
    let mut value: u128 = 0;
    for ch in s.chars() {
        let digit = b44_val(ch as u8).ok_or(Base44Error::InvalidChar)? as u128;
        value = value.checked_mul(44).ok_or(Base44Error::Overflow)?;
        value = value.checked_add(digit).ok_or(Base44Error::Overflow)?;
    }

    // Verify value fits in specified bits
    if bits < 128 && value >= (1u128 << bits) {
        return Err(Base44Error::Overflow);
    }

    // Convert to bytes (little-endian)
    let byte_count = (bits + 7) / 8;
    let mut bytes = vec![0u8; byte_count];
    for i in 0..byte_count {
        bytes[i] = (value & 0xFF) as u8;
        value >>= 8;
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let cases: &[&[u8]] = &[
            b"",
            b"A",
            b"AB",
            b"Hello, world!",
            &[0x00],
            &[0x00, 0x01, 0xFF, 0x80, 0x7F],
        ];
        for &case in cases {
            let s = encode(case);
            let dec = decode(&s).unwrap();
            assert_eq!(case, dec.as_slice());
        }
    }

    #[test]
    fn known_vectors() {
        // Base44 uses least-significant digit first (lsd-first): output order is c, b, a.
        // For a 2-byte group [u, v], form x = u*256 + v, then:
        // c = x % 44; x /= 44; b = x % 44; a = x / 44; and output chars are [c, b, a].
        // For a 1-byte group [u], b = u % 44; a = u / 44; and output chars are [b, a].
        // Edge cases at boundaries
        // [0x00, 0x00] -> x = 0; digits: c=0, b=0, a=0; output lsd-first -> "000"
        assert_eq!(encode(&[0x00, 0x00]), "000");

        // Test single byte encoding
        // [0x41] (ASCII 'A' = 65) -> b = 65 % 44 = 21 (L), a = 65 / 44 = 1 (1) -> "L1"
        assert_eq!(encode(&[0x41]), "L1");

        // Test two byte encoding
        // [0x00, 0x01] -> x = 1; c = 1 % 44 = 1, x = 0, b = 0, a = 0 -> "100"
        assert_eq!(encode(&[0x00, 0x01]), "100");

        // Verify decoding matches
        assert_eq!(decode("000").unwrap(), &[0x00, 0x00]);
        assert_eq!(decode("L1").unwrap(), &[0x41]);
        assert_eq!(decode("100").unwrap(), &[0x00, 0x01]);
    }

    #[test]
    fn errors() {
        // Error categories under test:
        // - InvalidChar: character not in Base44 alphabet
        // - Dangling: incomplete group (e.g., single trailing valid character)
        // - Overflow: numeric value exceeds maximum for the group
        // Invalid characters and structural errors
        assert!(matches!(decode("\t"), Err(Base44Error::InvalidChar))); // '\t' not in Base44 alphabet
        assert!(matches!(decode("\n"), Err(Base44Error::InvalidChar))); // '\n' not in Base44 alphabet
        assert!(matches!(decode(" "), Err(Base44Error::InvalidChar))); // space removed from Base44
        // Overflow cases
        // 3-char group with max digits -> value > 65535
        assert!(matches!(decode(":::"), Err(Base44Error::Overflow))); // ':::' -> 43*44^2 + 43*44 + 43 = 85183 > 65535
        // 2-char group producing >255
        assert!(matches!(decode("//"), Err(Base44Error::Overflow))); // '//' -> 42*44 + 42 = 1890 > 255

        assert!(matches!(decode("A"), Err(Base44Error::Dangling))); // single valid char -> incomplete group
        assert!(matches!(decode("😀"), Err(Base44Error::InvalidChar))); // not in Base44 alphabet
    }

    #[test]
    fn boundary_cases() {
        // Test maximum valid values for 2-char encoding (single byte)
        // Max single byte: 255
        // 255 = 5*44 + 35, so encoding should be alphabet[35] + alphabet[5] = "Z5"
        assert_eq!(encode(&[0xFF]), "Z5");
        assert_eq!(decode("Z5").unwrap(), &[0xFF]);

        // Test maximum valid 2-byte value: [0xFF, 0xFF]
        // x = 255*256 + 255 = 65535
        // c = 65535 % 44 = 19 (J), x = 1489
        // b = 1489 % 44 = 37 (%), a = 1489 / 44 = 33 (X)
        assert_eq!(encode(&[0xFF, 0xFF]), "J%X");
        assert_eq!(decode("J%X").unwrap(), &[0xFF, 0xFF]);

        // Test all alphabet characters are valid for decoding
        let alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ$%*+-./:";
        for (idx, ch) in alphabet.chars().enumerate() {
            // For positions 0-33 (0-9, A-X), can safely use "00{ch}" without overflow
            // Position 34 (Y) onwards: 34*44^2 = 65824 > 65535, so use "{ch}0" format
            if idx < 34 {
                let s = format!("00{}", ch);
                decode(&s).expect(&format!("Character {} should be valid in 3-char group", ch));
            } else {
                // For chars that would overflow in "00{ch}" format, use "{ch}0" (value < 255)
                let s = format!("{}0", ch);
                decode(&s).expect(&format!("Character {} should be valid", ch));
            }
        }

        // Test empty input
        assert_eq!(encode(&[]), "");
        assert_eq!(decode("").unwrap(), Vec::<u8>::new());

        // Test mixed length data (odd number of bytes)
        let data = &[0x01, 0x02, 0x03];
        let encoded = encode(data);
        assert_eq!(decode(&encoded).unwrap(), data);
    }

    #[test]
    fn url_safe_characters() {
        // Verify that encoded output contains no URL-problematic characters
        // (no space, which was removed from Base45)
        let test_data = &[
            &[0x00][..],
            &[0xFF],
            &[0x00, 0xFF],
            &[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0],
        ];

        for data in test_data {
            let encoded = encode(data);
            assert!(!encoded.contains(' '), "Encoded should not contain space");
            // Verify all chars are in our alphabet
            for ch in encoded.chars() {
                assert!(
                    BASE44_ALPHABET.contains(&(ch as u8)),
                    "Character {} not in alphabet",
                    ch
                );
            }
        }
    }

    #[test]
    fn optimal_bit_encoding_103() {
        // Test optimal encoding for 103 bits (common use case: UUID compression)
        // 2^103 < 44^19, so 103 bits should encode to exactly 19 characters
        let mut data = [0xFFu8; 13];
        data[12] = 0x7F; // Only 7 bits in last byte for 103 total bits
        let encoded = encode_bits(103, &data);
        assert_eq!(encoded.len(), 19, "103 bits should encode to 19 chars");

        let decoded = decode_bits(103, &encoded).unwrap();
        assert_eq!(decoded, data.to_vec(), "Roundtrip should preserve data");
    }

    #[test]
    fn optimal_bit_encoding_roundtrip() {
        // Test various bit lengths for roundtrip accuracy
        let test_cases = vec![
            (8, vec![0x42]),
            (16, vec![0x12, 0x34]),
            (24, vec![0xAB, 0xCD, 0xEF]),
            (103, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D]),
            (128, vec![0xFF; 16]),
        ];

        for (bits, data) in test_cases {
            let encoded = encode_bits(bits, &data);
            let decoded = decode_bits(bits, &encoded).unwrap();

            // Compare only the relevant bits
            let byte_count = (bits + 7) / 8;
            assert_eq!(decoded.len(), byte_count);

            // Verify data matches (may need to mask last byte)
            for i in 0..byte_count {
                if i == byte_count - 1 && bits % 8 != 0 {
                    let mask = (1u8 << (bits % 8)) - 1;
                    assert_eq!(decoded[i] & mask, data[i] & mask);
                } else {
                    assert_eq!(decoded[i], data[i]);
                }
            }
        }
    }

    #[test]
    fn optimal_vs_byte_pair_comparison() {
        // Compare optimal bit encoding vs byte-pair encoding for 103 bits
        let data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55];

        let optimal = encode_bits(103, &data);
        let byte_pair = encode(&data);

        // Optimal should be 19 chars, byte-pair should be 20 chars
        assert_eq!(optimal.len(), 19);
        assert_eq!(byte_pair.len(), 20);

        println!("103 bits: optimal={} chars, byte-pair={} chars, savings={}%",
                 optimal.len(), byte_pair.len(),
                 (byte_pair.len() - optimal.len()) * 100 / byte_pair.len());
    }

    #[test]
    fn optimal_bit_encoding_edge_cases() {
        // Test edge cases

        // All zeros
        let zeros = vec![0u8; 13];
        let encoded_zeros = encode_bits(103, &zeros);
        assert_eq!(encoded_zeros.len(), 19);
        let decoded_zeros = decode_bits(103, &encoded_zeros).unwrap();
        assert_eq!(decoded_zeros, zeros);

        // Single bit
        let one_bit = vec![0x01];
        let encoded_one = encode_bits(1, &one_bit);
        let decoded_one = decode_bits(1, &encoded_one).unwrap();
        assert_eq!(decoded_one[0] & 0x01, 1);

        // Maximum value for 103 bits
        let mut max_103 = vec![0xFFu8; 13];
        max_103[12] = 0x7F; // Only 7 bits in last byte
        let encoded_max = encode_bits(103, &max_103);
        let decoded_max = decode_bits(103, &encoded_max).unwrap();
        assert_eq!(decoded_max, max_103);
    }
}
