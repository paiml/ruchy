//! MD5 message digest (RFC 1321) behind the `compute_hash()` builtin (STDLIB-005).
//!
//! `compute_hash(path)` is a content fingerprint for duplicate detection, specified
//! as 32 lowercase hex digits of MD5. It is not a security primitive. The digest is
//! implemented in-tree so the builtin keeps that contract without an extra crate.

/// Per-step left-rotation amounts (RFC 1321 §3.4).
const SHIFTS: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, //
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, //
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, //
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

/// Per-step additive constants `floor(|sin(i + 1)| * 2^32)` (RFC 1321 §3.4).
const K: [u32; 64] = [
    0xd76a_a478,
    0xe8c7_b756,
    0x2420_70db,
    0xc1bd_ceee,
    0xf57c_0faf,
    0x4787_c62a,
    0xa830_4613,
    0xfd46_9501,
    0x6980_98d8,
    0x8b44_f7af,
    0xffff_5bb1,
    0x895c_d7be,
    0x6b90_1122,
    0xfd98_7193,
    0xa679_438e,
    0x49b4_0821,
    0xf61e_2562,
    0xc040_b340,
    0x265e_5a51,
    0xe9b6_c7aa,
    0xd62f_105d,
    0x0244_1453,
    0xd8a1_e681,
    0xe7d3_fbc8,
    0x21e1_cde6,
    0xc337_07d6,
    0xf4d5_0d87,
    0x455a_14ed,
    0xa9e3_e905,
    0xfcef_a3f8,
    0x676f_02d9,
    0x8d2a_4c8a,
    0xfffa_3942,
    0x8771_f681,
    0x6d9d_6122,
    0xfde5_380c,
    0xa4be_ea44,
    0x4bde_cfa9,
    0xf6bb_4b60,
    0xbebf_bc70,
    0x289b_7ec6,
    0xeaa1_27fa,
    0xd4ef_3085,
    0x0488_1d05,
    0xd9d4_d039,
    0xe6db_99e5,
    0x1fa2_7cf8,
    0xc4ac_5665,
    0xf429_2244,
    0x432a_ff97,
    0xab94_23a7,
    0xfc93_a039,
    0x655b_59c3,
    0x8f0c_cc92,
    0xffef_f47d,
    0x8584_5dd1,
    0x6fa8_7e4f,
    0xfe2c_e6e0,
    0xa301_4314,
    0x4e08_11a1,
    0xf753_7e82,
    0xbd3a_f235,
    0x2ad7_d2bb,
    0xeb86_d391,
];

/// Initial chaining values A, B, C, D (RFC 1321 §3.3).
const INIT: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];

/// MD5 digest of `data` as 32 lowercase hex digits.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     ruchy::stdlib::hash::md5_hex(b"hello world"),
///     "5eb63bbbe01eeed093cb22bb8f5acdc3"
/// );
/// ```
#[must_use]
pub fn md5_hex(data: &[u8]) -> String {
    md5_digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Raw 16-byte MD5 digest of `data`.
#[must_use]
pub fn md5_digest(data: &[u8]) -> [u8; 16] {
    let mut state = INIT;
    for block in pad(data).chunks_exact(64) {
        compress(&mut state, block);
    }
    let mut out = [0u8; 16];
    for (chunk, word) in out.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    out
}

/// Append the 0x80 marker, zero fill to 56 mod 64, and the little-endian bit length.
fn pad(data: &[u8]) -> Vec<u8> {
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity(data.len() + 72);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());
    msg
}

/// Round function value and message-word index for step `i` (RFC 1321 §3.4).
fn step_fn(i: usize, b: u32, c: u32, d: u32) -> (u32, usize) {
    match i / 16 {
        0 => ((b & c) | (!b & d), i),
        1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
        2 => (b ^ c ^ d, (3 * i + 5) % 16),
        _ => (c ^ (b | !d), (7 * i) % 16),
    }
}

/// Fold one 64-byte block into the chaining state.
fn compress(state: &mut [u32; 4], block: &[u8]) {
    let mut words = [0u32; 16];
    for (word, bytes) in words.iter_mut().zip(block.chunks_exact(4)) {
        *word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
    let [mut a, mut b, mut c, mut d] = *state;
    for i in 0..64 {
        let (f, g) = step_fn(i, b, c, d);
        let sum = a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(words[g]);
        a = d;
        d = c;
        c = b;
        b = b.wrapping_add(sum.rotate_left(SHIFTS[i]));
    }
    for (s, v) in state.iter_mut().zip([a, b, c, d]) {
        *s = s.wrapping_add(v);
    }
}

#[cfg(test)]
mod tests {
    use super::md5_hex;

    /// RFC 1321 Appendix A.5 test suite.
    #[test]
    fn test_stdlib_005_md5_rfc1321_suite() {
        let cases: [(&[u8], &str); 7] = [
            (b"", "d41d8cd98f00b204e9800998ecf8427e"),
            (b"a", "0cc175b9c0f1b6a831c399e269772661"),
            (b"abc", "900150983cd24fb0d6963f7d28e17f72"),
            (b"message digest", "f96b697d7cb7938d525a2f31aaf161d0"),
            (
                b"abcdefghijklmnopqrstuvwxyz",
                "c3fcd3d76192e4007dfb496cca67e13b",
            ),
            (
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                "d174ab98d277d9f5a5611c2c9f419d9f",
            ),
            (
                b"12345678901234567890123456789012345678901234567890123456789012345678901234567890",
                "57edf4a22be3c955ac49da2e2107b67a",
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(md5_hex(input), expected, "input {input:?}");
        }
    }

    /// Padding boundaries: 55 bytes fits one block, 56 and 64 need a second.
    #[test]
    fn test_stdlib_005_md5_padding_boundaries() {
        assert_eq!(md5_hex(&[b'a'; 55]), "ef1772b6dff9a122358552954ad0df65");
        assert_eq!(md5_hex(&[b'a'; 56]), "3b0c8ac703f828b04c6c197006d17218");
        assert_eq!(md5_hex(&[b'a'; 64]), "014842d480b571495a4a0363793f7367");
    }

    mod properties {
        use super::super::md5_hex;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_stdlib_005_md5_is_32_lower_hex(data in proptest::collection::vec(any::<u8>(), 0..300)) {
                let hex = md5_hex(&data);
                prop_assert_eq!(hex.len(), 32);
                prop_assert!(hex.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
            }
        }
    }
}
