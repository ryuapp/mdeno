// Copyright 2018-2025 the Deno authors. MIT license.

/// Deno's fast_uuid_v4 implementation
/// Converts 16 random bytes into a UUID v4 string
fn fast_uuid_v4(rng_bytes: &mut [u8; 16]) -> String {
    // Set version (4) and variant (RFC4122) bits
    rng_bytes[6] = (rng_bytes[6] & 0x0f) | 0x40;
    rng_bytes[8] = (rng_bytes[8] & 0x3f) | 0x80;

    // Hex lookup table
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    let mut buf = [0u8; 36];

    for i in 0..16 {
        let byte = rng_bytes[i];
        let pos = i * 2
            + if i >= 10 {
                4
            } else if i >= 8 {
                3
            } else if i >= 6 {
                2
            } else if i >= 4 {
                1
            } else {
                0
            };
        buf[pos] = HEX_CHARS[(byte >> 4) as usize];
        buf[pos + 1] = HEX_CHARS[(byte & 0x0f) as usize];
    }

    buf[8] = b'-';
    buf[13] = b'-';
    buf[18] = b'-';
    buf[23] = b'-';

    // SAFETY: We only write ASCII hex digits and hyphens
    unsafe { String::from_utf8_unchecked(buf.to_vec()) }
}

/// Generate UUID v4 string
pub fn random_uuid() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("Failed to get random bytes");
    fast_uuid_v4(&mut bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_uuid_format() {
        let uuid = random_uuid();
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[8..9], "-");
        assert_eq!(&uuid[13..14], "-");
        assert_eq!(&uuid[18..19], "-");
        assert_eq!(&uuid[23..24], "-");

        // Check version 4
        let version_char = uuid.chars().nth(14).unwrap();
        assert_eq!(version_char, '4');
    }

    #[test]
    fn test_random_uuid_uniqueness() {
        let uuid1 = random_uuid();
        let uuid2 = random_uuid();
        assert_ne!(uuid1, uuid2);
    }
}
