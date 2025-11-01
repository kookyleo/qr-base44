//! qr-base43: Base43 encoder/decoder for arbitrary bytes using URL-safe QR-compatible alphabet.
//! - Encoding groups: 2 bytes -> 3 chars; 1 byte -> 2 chars.
//! - Alphabet: "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ%*+-./:" (43 chars, excludes space and $)
//! - Public API encodes &[u8] -> String and decodes &str -> Vec<u8>.

#[derive(Debug, thiserror::Error)]
pub enum Base43Error {
    #[error("invalid base43 character")]
    InvalidChar,
    #[error("dangling character group")]
    Dangling,
    #[error("value overflow")]
    Overflow,
}

/// Base43 alphabet: URL-safe QR-compatible subset (excludes space and $)
pub const BASE43_ALPHABET: &[u8; 43] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ%*+-./:";

#[inline]
fn b43_val(ch: u8) -> Option<u16> {
    match ch {
        b'0'..=b'9' => Some((ch - b'0') as u16),
        b'A'..=b'Z' => Some(10 + (ch - b'A') as u16),
        b'%' => Some(36),
        b'*' => Some(37),
        b'+' => Some(38),
        b'-' => Some(39),
        b'.' => Some(40),
        b'/' => Some(41),
        b':' => Some(42),
        _ => None,
    }
}

/// Encode arbitrary bytes into a Base43 string.
/// Groups of 2 bytes produce 3 characters; a final single byte produces 2 characters.
pub fn encode(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() * 3).div_ceil(2));
    let mut i = 0;
    while i + 1 < input.len() {
        let x = (input[i] as u16) * 256 + (input[i + 1] as u16);
        let c = x % 43; // least significant digit
        let x = x / 43;
        let b = x % 43;
        let a = x / 43; // most significant digit
        // Base43 outputs least-significant digit first
        out.push(BASE43_ALPHABET[c as usize] as char);
        out.push(BASE43_ALPHABET[b as usize] as char);
        out.push(BASE43_ALPHABET[a as usize] as char);
        i += 2;
    }
    if i < input.len() {
        let x = input[i] as u16;
        let b = x % 43;
        let a = x / 43;
        // Base43 outputs least-significant digit first for single byte too
        out.push(BASE43_ALPHABET[b as usize] as char);
        out.push(BASE43_ALPHABET[a as usize] as char);
    }
    out
}

/// Decode a Base43 string back to raw bytes.
/// Accepts only the Base43 alphabet; returns errors for invalid chars, dangling final char, or overflow.
pub fn decode(s: &str) -> Result<Vec<u8>, Base43Error> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i + 2 < bytes.len() {
        // Input is least-significant digit first: c (lsd), b, a (msd)
        let c0 = b43_val(bytes[i]).ok_or(Base43Error::InvalidChar)? as u32;
        let c1 = b43_val(bytes[i + 1]).ok_or(Base43Error::InvalidChar)? as u32;
        let c2 = b43_val(bytes[i + 2]).ok_or(Base43Error::InvalidChar)? as u32;
        let x: u32 = c2 * 43 * 43 + c1 * 43 + c0; // 0..(43^3 - 1)
        if x > 65535 {
            return Err(Base43Error::Overflow);
        }
        out.push((x / 256) as u8);
        out.push((x % 256) as u8);
        i += 3;
    }
    if i < bytes.len() {
        if i + 1 >= bytes.len() {
            // Single trailing character: report InvalidChar if it's not in alphabet, otherwise Dangling
            if b43_val(bytes[i]).is_none() {
                return Err(Base43Error::InvalidChar);
            }
            return Err(Base43Error::Dangling);
        }
        let c0 = b43_val(bytes[i]).ok_or(Base43Error::InvalidChar)? as u32;
        let c1 = b43_val(bytes[i + 1]).ok_or(Base43Error::InvalidChar)? as u32;
        let x: u32 = c1 * 43 + c0; // 0..(43^2 - 1)
        if x > 255 {
            return Err(Base43Error::Overflow);
        }
        out.push(x as u8);
    }
    Ok(out)
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
        // Base43 uses least-significant digit first (lsd-first): output order is c, b, a.
        // For a 2-byte group [u, v], form x = u*256 + v, then:
        // c = x % 43; x /= 43; b = x % 43; a = x / 43; and output chars are [c, b, a].
        // For a 1-byte group [u], b = u % 43; a = u / 43; and output chars are [b, a].
        // Edge cases at boundaries
        // [0x00, 0x00] -> x = 0; digits: c=0, b=0, a=0; output lsd-first -> "000"
        assert_eq!(encode(&[0x00, 0x00]), "000");

        // Test single byte encoding
        // [0x41] (ASCII 'A' = 65) -> b = 65 % 43 = 22 (M), a = 65 / 43 = 1 (1) -> "M1"
        assert_eq!(encode(&[0x41]), "M1");

        // Test two byte encoding
        // [0x00, 0x01] -> x = 1; c = 1 % 43 = 1, x = 0, b = 0, a = 0 -> "100"
        assert_eq!(encode(&[0x00, 0x01]), "100");

        // Verify decoding matches
        assert_eq!(decode("000").unwrap(), &[0x00, 0x00]);
        assert_eq!(decode("M1").unwrap(), &[0x41]);
        assert_eq!(decode("100").unwrap(), &[0x00, 0x01]);
    }

    #[test]
    fn errors() {
        // Error categories under test:
        // - InvalidChar: character not in Base43 alphabet
        // - Dangling: incomplete group (e.g., single trailing valid character)
        // - Overflow: numeric value exceeds maximum for the group
        // Invalid characters and structural errors
        assert!(matches!(decode("\t"), Err(Base43Error::InvalidChar))); // '\t' not in Base43 alphabet
        assert!(matches!(decode("\n"), Err(Base43Error::InvalidChar))); // '\n' not in Base43 alphabet
        assert!(matches!(decode(" "), Err(Base43Error::InvalidChar))); // space removed from Base43
        assert!(matches!(decode("$"), Err(Base43Error::InvalidChar))); // $ removed from Base43
        // Overflow cases
        // 3-char group with max digits -> value > 65535
        assert!(matches!(decode(":::"), Err(Base43Error::Overflow))); // ':::' -> 42*43^2 + 42*43 + 42 = 79506 > 65535
        // 2-char group producing >255
        assert!(matches!(decode("//"), Err(Base43Error::Overflow))); // '//' -> 41*43 + 41 = 1804 > 255

        assert!(matches!(decode("A"), Err(Base43Error::Dangling))); // single valid char -> incomplete group
        assert!(matches!(decode("😀"), Err(Base43Error::InvalidChar))); // not in Base43 alphabet
    }

    #[test]
    fn boundary_cases() {
        // Test maximum valid values for 2-char encoding (single byte)
        // Max single byte: 255
        // 255 = 5*43 + 40, so encoding should be alphabet[40] + alphabet[5] = ".5"
        assert_eq!(encode(&[0xFF]), ".5");
        assert_eq!(decode(".5").unwrap(), &[0xFF]);

        // Test maximum valid 2-byte value: [0xFF, 0xFF]
        // x = 255*256 + 255 = 65535
        // c = 65535 % 43 = 3 (3), x = 1524
        // b = 1524 % 43 = 19 (J), a = 1524 / 43 = 35 (Z)
        assert_eq!(encode(&[0xFF, 0xFF]), "3JZ");
        assert_eq!(decode("3JZ").unwrap(), &[0xFF, 0xFF]);

        // Test all alphabet characters are valid for decoding
        let alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ%*+-./:";
        for (idx, ch) in alphabet.chars().enumerate() {
            // For positions 0-35 (0-9, A-Z), can safely use "00{ch}" without overflow
            // For positions 36+ (% onwards), use "{ch}0" to avoid overflow
            if idx < 36 {
                let s = format!("00{}", ch);
                decode(&s).expect(&format!("Character {} should be valid in 3-char group", ch));
            } else {
                // For special chars, use {ch}0 to avoid overflow (value < 255)
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
        // (no space, no $, which were removed from Base45)
        let test_data = &[
            &[0x00][..],
            &[0xFF],
            &[0x00, 0xFF],
            &[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0],
        ];

        for data in test_data {
            let encoded = encode(data);
            assert!(!encoded.contains(' '), "Encoded should not contain space");
            assert!(!encoded.contains('$'), "Encoded should not contain $");
            // Verify all chars are in our alphabet
            for ch in encoded.chars() {
                assert!(
                    BASE43_ALPHABET.contains(&(ch as u8)),
                    "Character {} not in alphabet",
                    ch
                );
            }
        }
    }
}
