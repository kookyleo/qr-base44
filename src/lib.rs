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

/// Encode exactly 103 bits (packed in 13 bytes) as a u128 integer into a 19-character Base44 string.
///
/// This is optimal encoding for 103-bit data: 2^103 < 44^19, so all 103-bit values
/// fit exactly in 19 Base44 characters.
///
/// **Important**: The input must represent a value that fits in 103 bits. This means
/// byte 12 (the last byte) must have its MSB set to 0 (i.e., byte[12] <= 0x7F).
/// Values exceeding 103 bits may produce incorrect results or panic.
///
/// # Note on byte ordering
/// Bytes are interpreted in little-endian order (LSB-first), consistent with
/// typical bit-packing conventions where bits are packed from LSB to MSB.
///
/// # Example
/// ```
/// use qr_base44::encode_103bits;
///
/// // Example: 103-bit value packed in 13 bytes (last byte MSB = 0)
/// let data: [u8; 13] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
///                       0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x7F];  // Note: 0x7F, not 0xFF
/// let encoded = encode_103bits(&data);
/// assert_eq!(encoded.len(), 19);
/// ```
pub fn encode_103bits(bytes: &[u8; 13]) -> String {
    // Convert 13 bytes to u128 (little-endian, LSB-first)
    let mut value: u128 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        value |= (b as u128) << (i * 8);
    }

    // Convert to base44 (exactly 19 digits)
    let mut result = Vec::with_capacity(19);
    let mut v = value;
    for _ in 0..19 {
        let digit = (v % 44) as usize;
        result.push(BASE44_ALPHABET[digit]);
        v /= 44;
    }

    // Reverse to get most significant digit first
    result.reverse();
    // SAFETY: BASE44_ALPHABET contains only ASCII characters
    unsafe { String::from_utf8_unchecked(result) }
}

/// Decode a 19-character Base44 string back to exactly 103 bits (packed in 13 bytes).
///
/// Returns bytes in little-endian order (LSB-first), matching the encoding convention.
/// The returned value is guaranteed to fit in 103 bits (byte 12 will have MSB = 0).
///
/// # Errors
///
/// Returns error if:
/// - Length is not exactly 19 characters
/// - Contains characters not in Base44 alphabet
/// - Numeric value exceeds 103 bits (overflow)
///
/// # Example
/// ```
/// use qr_base44::{encode_103bits, decode_103bits};
///
/// let data: [u8; 13] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
///                       0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x7F];  // Last byte <= 0x7F
/// let encoded = encode_103bits(&data);
/// let decoded = decode_103bits(&encoded).unwrap();
/// assert_eq!(data, decoded);
/// ```
pub fn decode_103bits(s: &str) -> Result<[u8; 13], Base44Error> {
    if s.len() != 19 {
        return Err(Base44Error::Dangling);
    }

    // Convert base44 string to u128
    let mut value: u128 = 0;
    for ch in s.chars() {
        let digit = b44_val(ch as u8).ok_or(Base44Error::InvalidChar)? as u128;

        // Check for overflow before multiplication
        // 44^19 = 16,811,282,773,058,972,887,713,478,344,704
        // u128::MAX = 340,282,366,920,938,463,463,374,607,431,768,211,455
        // Safe to multiply by 44 as long as value < u128::MAX / 44
        if value > u128::MAX / 44 {
            return Err(Base44Error::Overflow);
        }

        value = value * 44 + digit;
    }

    // Convert u128 back to 13 bytes (little-endian)
    let mut bytes = [0u8; 13];
    for i in 0..13 {
        bytes[i] = (value & 0xFF) as u8;
        value >>= 8;
    }

    // Verify that the value fit in 103 bits
    // After extracting 13 bytes (104 bits), remaining value should be 0
    if value != 0 {
        return Err(Base44Error::Overflow);
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
    fn fixed_length_roundtrip() {
        // Test with various patterns
        let test_cases: &[[u8; 13]] = &[
            [0x00; 13], // All zeros
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            ],
            // Max value for 103 bits: last byte (byte 12) MSB must be 0
            [
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F,
            ],
        ];

        for &data in test_cases {
            let encoded = encode_103bits(&data);
            assert_eq!(encoded.len(), 19, "Encoded length should be exactly 19");

            let decoded = decode_103bits(&encoded)
                .unwrap_or_else(|_| panic!("Failed to decode: {}", encoded));
            assert_eq!(data, decoded, "Roundtrip failed for {:02X?}", data);
        }
    }

    #[test]
    fn fixed_length_exactly_19_chars() {
        // Verify 100 random-like patterns all encode to exactly 19 characters
        for i in 0..100 {
            let mut data = [0u8; 13];
            // Use pseudo-random pattern based on index
            for j in 0..13 {
                data[j] = ((i * 17 + j * 23) % 256) as u8;
            }
            // Ensure MSB of last byte is 0 to stay within 103 bits
            data[12] &= 0x7F;

            let encoded = encode_103bits(&data);
            assert_eq!(encoded.len(), 19);

            let decoded = decode_103bits(&encoded).unwrap();
            assert_eq!(data, decoded);
        }
    }

    #[test]
    fn fixed_invalid_length() {
        // Too short
        assert!(matches!(
            decode_103bits("TOOSHORT"),
            Err(Base44Error::Dangling)
        ));

        // Too long (23 chars)
        assert!(matches!(
            decode_103bits("WAYTOOLONGFORBASE44SURE"),
            Err(Base44Error::Dangling)
        ));

        // 18 chars
        assert!(matches!(
            decode_103bits("012345678901234567"),
            Err(Base44Error::Dangling)
        ));

        // 20 chars
        assert!(matches!(
            decode_103bits("01234567890123456789"),
            Err(Base44Error::Dangling)
        ));
    }

    #[test]
    fn fixed_invalid_chars() {
        assert!(matches!(
            decode_103bits("ABC!EFGHIJ123456789"), // '!' not in alphabet
            Err(Base44Error::InvalidChar)
        ));

        assert!(matches!(
            decode_103bits("abcdefghij123456789"), // lowercase not allowed
            Err(Base44Error::InvalidChar)
        ));

        assert!(matches!(
            decode_103bits("ABC EFGHIJ123456789"), // space not in alphabet
            Err(Base44Error::InvalidChar)
        ));
    }

    #[test]
    fn comparison_with_byte_pair_encoding() {
        // Compare output length: optimal vs byte-pair
        let data = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
        ];

        let optimal = encode_103bits(&data);
        let byte_pair = encode(&data);

        // Verify size difference
        assert_eq!(optimal.len(), 19);
        assert_eq!(byte_pair.len(), 20);

        // Both should decode back to original
        let decoded_optimal = decode_103bits(&optimal).unwrap();
        let decoded_byte_pair = decode(&byte_pair).unwrap();

        assert_eq!(data.as_slice(), decoded_optimal.as_slice());
        assert_eq!(data.as_slice(), decoded_byte_pair.as_slice());
    }
}
