//! Hashing and hash-bit helpers shared by the in-memory and on-disk tables.
//!
//! Every function here is pure and deterministic across runs and machines:
//! the on-disk format depends on a key hashing to the same value forever.

pub const HASH_BITS: u32 = u64::BITS;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// 64-bit FNV-1a hash. Deterministic across runs, which matters once buckets
/// live on disk. Use [`key_hash`] for bucket placement.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET_BASIS, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(FNV_PRIME)
    })
}

/// MurmurHash3's 64-bit finalizer. A bijection in which every input bit
/// affects every output bit with roughly even probability.
fn fmix64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^ (h >> 33)
}

/// The hash the table uses to place `key`.
///
/// Raw FNV-1a mixes poorly into its high bits: keys differing only in their
/// last byte share roughly the top 20 bits, and the directory is indexed by
/// the top bits. Finalizing with `fmix64` spreads every input bit across the
/// whole word. Being a bijection, it keeps distinct FNV hashes distinct.
pub fn key_hash(key: &[u8]) -> u64 {
    fmix64(fnv1a(key))
}

/// The top `depth` bits of `hash`, as a directory index.
pub fn prefix(hash: u64, depth: u32) -> usize {
    if depth == 0 {
        0
    } else {
        // The directory holds 2^depth slots, so depth never exceeds usize bits.
        (hash >> (HASH_BITS - depth)) as usize
    }
}

/// Bit number `depth` of `hash`, counting from the most significant bit.
pub fn bit(hash: u64, depth: u32) -> bool {
    (hash >> (HASH_BITS - 1 - depth)) & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_known_vectors() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn key_hash_spreads_last_byte_into_top_bits() {
        // Raw FNV-1a leaves these sharing a long high-bit prefix.
        let top = |h: u64| h >> 56;
        assert_eq!(top(fnv1a(b"key 1")), top(fnv1a(b"key 2")));
        let tops: std::collections::HashSet<u64> = (0..10)
            .map(|i| top(key_hash(format!("key {i}").as_bytes())))
            .collect();
        assert!(tops.len() > 5, "top byte barely varies: {tops:?}");
    }

    #[test]
    fn fmix64_is_invertible_on_samples() {
        let samples: std::collections::HashSet<u64> = (0..10_000u64).map(fmix64).collect();
        assert_eq!(samples.len(), 10_000);
        assert_eq!(fmix64(0), 0);
    }

    #[test]
    fn prefix_and_bit_use_most_significant_bits() {
        let h = 0b1011u64 << 60;
        assert_eq!(prefix(h, 0), 0);
        assert_eq!(prefix(h, 1), 0b1);
        assert_eq!(prefix(h, 4), 0b1011);
        assert!(bit(h, 0));
        assert!(!bit(h, 1));
        assert!(bit(h, 3));
    }
}
