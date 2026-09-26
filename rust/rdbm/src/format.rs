//! The on-disk format: pure encoding and decoding of the file header,
//! directory, and pages. Nothing here performs I/O.
//!
//! The file is an array of fixed-size pages. All integers are little-endian.
//!
//! ```text
//! page 0              header
//! pages dir_page..    directory: 2^global_depth u64 page numbers
//! other pages         bucket, overflow, or free pages
//! ```
//!
//! Header (first 56 bytes of page 0; the rest of the page is zero):
//!
//! ```text
//! off size field
//!   0    4 magic "RDBM"
//!   4    4 format version (1)
//!   8    4 page size
//!  12    1 global depth
//!  13    1 max depth
//!  14    2 reserved (0)
//!  16    8 first directory page
//!  24    8 directory page count
//!  32    8 total page count
//!  40    8 first free page (0 = none)
//!  48    8 entry count
//! ```
//!
//! Bucket, overflow, and free pages share one layout. Record bytes are packed
//! from the end of the page towards the slot array.
//!
//! ```text
//! off size field
//!   0    1 kind: 1 bucket, 2 overflow, 3 free
//!   1    1 local depth of the bucket's chain (0 for free pages)
//!   2    2 record count
//!   4    4 reserved (0)
//!   8    8 next page: overflow chain or free list (0 = none)
//!  16      slots, 16 bytes each:
//!            0 8 hash   8 2 offset   10 2 key length   12 2 value length
//!            14 2 reserved (0)
//!  ...     free space
//!  ...     records: key bytes then value bytes, at slot offset
//! ```
//!
//! Page 0 is always the header, so page number 0 doubles as "none".

use crate::error::{Error, Result};

pub const MAGIC: [u8; 4] = *b"RDBM";
pub const VERSION: u32 = 1;
pub const HEADER_LEN: usize = 56;
pub const PAGE_HEADER_LEN: usize = 16;
pub const SLOT_LEN: usize = 16;

pub const MIN_PAGE_SIZE: u32 = 512;
pub const MAX_PAGE_SIZE: u32 = 32 * 1024;
pub const DEFAULT_PAGE_SIZE: u32 = 4096;
/// Caps the directory at 2^32 slots (32 GiB).
pub const MAX_DEPTH_LIMIT: u8 = 32;
/// A 2^24-slot directory is 128 MiB; beyond that, buckets chain overflow pages.
pub const DEFAULT_MAX_DEPTH: u8 = 24;

pub fn check_page_size(page_size: u32) -> Result<()> {
    if page_size.is_power_of_two() && (MIN_PAGE_SIZE..=MAX_PAGE_SIZE).contains(&page_size) {
        Ok(())
    } else {
        Err(Error::InvalidOption(format!(
            "page size {page_size} must be a power of two from {MIN_PAGE_SIZE} to {MAX_PAGE_SIZE}"
        )))
    }
}

pub fn check_max_depth(max_depth: u8) -> Result<()> {
    if max_depth <= MAX_DEPTH_LIMIT {
        Ok(())
    } else {
        Err(Error::InvalidOption(format!(
            "max depth {max_depth} exceeds {MAX_DEPTH_LIMIT}"
        )))
    }
}

fn corrupt<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Corrupt(msg.into()))
}

fn u16_at(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}

fn u32_at(b: &[u8], off: usize) -> u32 {
    let mut a = [0u8; 4];
    a.copy_from_slice(&b[off..off + 4]);
    u32::from_le_bytes(a)
}

fn u64_at(b: &[u8], off: usize) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&b[off..off + 8]);
    u64::from_le_bytes(a)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub page_size: u32,
    pub global_depth: u8,
    pub max_depth: u8,
    pub dir_page: u64,
    pub dir_pages: u64,
    pub page_count: u64,
    pub free_head: u64,
    pub entry_count: u64,
}

impl Header {
    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let mut b = [0u8; HEADER_LEN];
        b[0..4].copy_from_slice(&MAGIC);
        b[4..8].copy_from_slice(&VERSION.to_le_bytes());
        b[8..12].copy_from_slice(&self.page_size.to_le_bytes());
        b[12] = self.global_depth;
        b[13] = self.max_depth;
        b[16..24].copy_from_slice(&self.dir_page.to_le_bytes());
        b[24..32].copy_from_slice(&self.dir_pages.to_le_bytes());
        b[32..40].copy_from_slice(&self.page_count.to_le_bytes());
        b[40..48].copy_from_slice(&self.free_head.to_le_bytes());
        b[48..56].copy_from_slice(&self.entry_count.to_le_bytes());
        b
    }

    /// Decodes and validates the fields that make sense on their own. Checks
    /// against the file itself (its length, page ranges) are the caller's.
    pub fn decode(b: &[u8]) -> Result<Header> {
        if b.len() < HEADER_LEN || b[0..4] != MAGIC {
            return corrupt("not an rdbm database (bad magic number)");
        }
        let version = u32_at(b, 4);
        if version != VERSION {
            return corrupt(format!("unsupported format version {version}"));
        }
        let h = Header {
            page_size: u32_at(b, 8),
            global_depth: b[12],
            max_depth: b[13],
            dir_page: u64_at(b, 16),
            dir_pages: u64_at(b, 24),
            page_count: u64_at(b, 32),
            free_head: u64_at(b, 40),
            entry_count: u64_at(b, 48),
        };
        check_page_size(h.page_size).or_else(|e| corrupt(e.to_string()))?;
        check_max_depth(h.max_depth).or_else(|e| corrupt(e.to_string()))?;
        if h.global_depth > h.max_depth {
            return corrupt(format!(
                "global depth {} exceeds max depth {}",
                h.global_depth, h.max_depth
            ));
        }
        Ok(h)
    }
}

/// Pages needed to hold a directory of `slots` entries.
pub fn directory_pages(slots: usize, page_size: u32) -> u64 {
    (slots * 8).div_ceil(page_size as usize) as u64
}

/// Encodes the directory, zero-padded to `len` bytes.
pub fn encode_directory(directory: &[u64], len: usize) -> Vec<u8> {
    let mut b: Vec<u8> = directory.iter().flat_map(|p| p.to_le_bytes()).collect();
    b.resize(len.max(b.len()), 0);
    b
}

pub fn decode_directory(b: &[u8], slots: usize) -> Result<Vec<u64>> {
    if b.len() < slots * 8 {
        return corrupt("directory is truncated");
    }
    Ok((0..slots).map(|i| u64_at(b, i * 8)).collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub hash: u64,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

/// Bytes a record occupies in a page: its slot plus its key and value.
pub fn record_size(key_len: usize, value_len: usize) -> usize {
    SLOT_LEN + key_len + value_len
}

/// Bytes a page holding `records` needs, including the page header.
pub fn used_bytes(records: &[Record]) -> usize {
    PAGE_HEADER_LEN
        + records
            .iter()
            .map(|r| record_size(r.key.len(), r.value.len()))
            .sum::<usize>()
}

/// The largest key length plus value length that fits in an empty page.
pub fn max_payload(page_size: u32) -> usize {
    page_size as usize - PAGE_HEADER_LEN - SLOT_LEN
}

/// Splits `records` into page-sized groups, first-fit in order. Always
/// returns at least one (possibly empty) group. Each record must fit in an
/// empty page on its own.
pub fn pack(records: Vec<Record>, page_size: u32) -> Vec<Vec<Record>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    let mut used = PAGE_HEADER_LEN;
    for r in records {
        let size = record_size(r.key.len(), r.value.len());
        if used + size > page_size as usize && !current.is_empty() {
            groups.push(std::mem::take(&mut current));
            used = PAGE_HEADER_LEN;
        }
        used += size;
        current.push(r);
    }
    groups.push(current);
    groups
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageKind {
    Bucket = 1,
    Overflow = 2,
    Free = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub kind: PageKind,
    pub local_depth: u8,
    pub next: u64,
    pub records: Vec<Record>,
}

impl Page {
    pub fn free(next: u64) -> Page {
        Page {
            kind: PageKind::Free,
            local_depth: 0,
            next,
            records: Vec::new(),
        }
    }

    pub fn encode(&self, page_size: u32) -> Result<Vec<u8>> {
        let size = page_size as usize;
        let used = used_bytes(&self.records);
        if used > size {
            return Err(Error::TooLarge {
                size: used - PAGE_HEADER_LEN - SLOT_LEN,
                max: max_payload(page_size),
            });
        }
        let mut b = vec![0u8; size];
        b[0] = self.kind as u8;
        b[1] = self.local_depth;
        // used <= size <= 32 KiB bounds the count, offsets, and lengths to u16.
        b[2..4].copy_from_slice(&(self.records.len() as u16).to_le_bytes());
        b[8..16].copy_from_slice(&self.next.to_le_bytes());

        let mut end = size;
        for (i, r) in self.records.iter().enumerate() {
            end -= r.key.len() + r.value.len();
            b[end..end + r.key.len()].copy_from_slice(&r.key);
            b[end + r.key.len()..end + r.key.len() + r.value.len()].copy_from_slice(&r.value);

            let slot = PAGE_HEADER_LEN + i * SLOT_LEN;
            b[slot..slot + 8].copy_from_slice(&r.hash.to_le_bytes());
            b[slot + 8..slot + 10].copy_from_slice(&(end as u16).to_le_bytes());
            b[slot + 10..slot + 12].copy_from_slice(&(r.key.len() as u16).to_le_bytes());
            b[slot + 12..slot + 14].copy_from_slice(&(r.value.len() as u16).to_le_bytes());
        }
        Ok(b)
    }

    pub fn decode(b: &[u8]) -> Result<Page> {
        if b.len() < PAGE_HEADER_LEN {
            return corrupt("page is shorter than its header");
        }
        let kind = match b[0] {
            1 => PageKind::Bucket,
            2 => PageKind::Overflow,
            3 => PageKind::Free,
            k => return corrupt(format!("unknown page kind {k}")),
        };
        let count = u16_at(b, 2) as usize;
        let slots_end = PAGE_HEADER_LEN + count * SLOT_LEN;
        if slots_end > b.len() {
            return corrupt(format!("{count} slots overrun the page"));
        }
        if kind == PageKind::Free && count != 0 {
            return corrupt("free page holds records");
        }

        let mut records = Vec::with_capacity(count);
        for i in 0..count {
            let slot = PAGE_HEADER_LEN + i * SLOT_LEN;
            let offset = u16_at(b, slot + 8) as usize;
            let key_len = u16_at(b, slot + 10) as usize;
            let value_len = u16_at(b, slot + 12) as usize;
            let end = offset + key_len + value_len;
            if offset < slots_end || end > b.len() {
                return corrupt(format!("record {i} lies outside the record area"));
            }
            records.push(Record {
                hash: u64_at(b, slot),
                key: b[offset..offset + key_len].to_vec(),
                value: b[offset + key_len..end].to_vec(),
            });
        }
        Ok(Page {
            kind,
            local_depth: b[1],
            next: u64_at(b, 8),
            records,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(key: &[u8], value: &[u8]) -> Record {
        Record {
            hash: crate::hash::key_hash(key),
            key: key.to_vec(),
            value: value.to_vec(),
        }
    }

    fn header() -> Header {
        Header {
            page_size: 4096,
            global_depth: 3,
            max_depth: 24,
            dir_page: 1,
            dir_pages: 1,
            page_count: 12,
            free_head: 7,
            entry_count: 99,
        }
    }

    #[test]
    fn header_round_trips() {
        let h = header();
        let b = h.encode();
        assert_eq!(&b[0..4], b"RDBM");
        assert_eq!(Header::decode(&b).unwrap(), h);
    }

    #[test]
    fn header_rejects_bad_magic_version_and_sizes() {
        let mut b = header().encode();
        b[0] = b'X';
        assert!(Header::decode(&b).is_err());

        let mut b = header().encode();
        b[4] = 2;
        assert!(Header::decode(&b).is_err());

        let bad_page = Header {
            page_size: 1000,
            ..header()
        };
        assert!(Header::decode(&bad_page.encode()).is_err());

        let too_deep = Header {
            global_depth: 25,
            ..header()
        };
        assert!(Header::decode(&too_deep.encode()).is_err());

        assert!(Header::decode(&[0u8; 10]).is_err());
    }

    #[test]
    fn page_size_and_depth_limits() {
        assert!(check_page_size(512).is_ok());
        assert!(check_page_size(32768).is_ok());
        assert!(check_page_size(256).is_err());
        assert!(check_page_size(65536).is_err());
        assert!(check_page_size(3000).is_err());
        assert!(check_max_depth(32).is_ok());
        assert!(check_max_depth(33).is_err());
    }

    #[test]
    fn page_round_trips() {
        let page = Page {
            kind: PageKind::Bucket,
            local_depth: 5,
            next: 42,
            records: vec![
                rec(b"key1", b"value one"),
                rec(b"", b""),
                rec(&[0, 255], &[7; 100]),
            ],
        };
        let b = page.encode(512).unwrap();
        assert_eq!(b.len(), 512);
        assert_eq!(Page::decode(&b).unwrap(), page);
    }

    #[test]
    fn records_pack_from_the_end_of_the_page() {
        let page = Page {
            kind: PageKind::Bucket,
            local_depth: 0,
            next: 0,
            records: vec![rec(b"key1", b"value one")],
        };
        let b = page.encode(512).unwrap();
        let tail = &b[512 - 13..];
        assert_eq!(tail, b"key1value one");
        assert_eq!(u16_at(&b, PAGE_HEADER_LEN + 8), 512 - 13);
        assert_eq!(u16_at(&b, PAGE_HEADER_LEN + 10), 4);
        assert_eq!(u16_at(&b, PAGE_HEADER_LEN + 12), 9);
    }

    #[test]
    fn encode_rejects_overfull_page() {
        let page = Page {
            kind: PageKind::Bucket,
            local_depth: 0,
            next: 0,
            records: vec![rec(b"k", &[0; 600])],
        };
        assert!(matches!(page.encode(512), Err(Error::TooLarge { .. })));
    }

    #[test]
    fn exactly_full_page_encodes() {
        let page = Page {
            kind: PageKind::Overflow,
            local_depth: 0,
            next: 0,
            records: vec![rec(b"", &vec![1; max_payload(512)])],
        };
        let b = page.encode(512).unwrap();
        assert_eq!(Page::decode(&b).unwrap(), page);
    }

    #[test]
    fn decode_rejects_malformed_pages() {
        let good = Page {
            kind: PageKind::Bucket,
            local_depth: 0,
            next: 0,
            records: vec![rec(b"k", b"v")],
        }
        .encode(512)
        .unwrap();

        let mut b = good.clone();
        b[0] = 9;
        assert!(Page::decode(&b).is_err(), "bad kind");

        let mut b = good.clone();
        b[2..4].copy_from_slice(&100u16.to_le_bytes());
        assert!(Page::decode(&b).is_err(), "slots overrun page");

        let mut b = good.clone();
        b[PAGE_HEADER_LEN + 8..PAGE_HEADER_LEN + 10].copy_from_slice(&511u16.to_le_bytes());
        assert!(Page::decode(&b).is_err(), "record overruns page");

        let mut b = good.clone();
        b[PAGE_HEADER_LEN + 8..PAGE_HEADER_LEN + 10].copy_from_slice(&0u16.to_le_bytes());
        assert!(Page::decode(&b).is_err(), "record overlaps slots");

        let mut b = Page::free(0).encode(512).unwrap();
        b[2] = 1;
        assert!(Page::decode(&b).is_err(), "free page with records");
    }

    #[test]
    fn free_page_round_trips() {
        let b = Page::free(17).encode(512).unwrap();
        assert_eq!(b[0], 3);
        assert_eq!(Page::decode(&b).unwrap(), Page::free(17));
    }

    #[test]
    fn directory_round_trips_with_padding() {
        let dir = vec![2, 2, 5, 9];
        let b = encode_directory(&dir, 512);
        assert_eq!(b.len(), 512);
        assert_eq!(decode_directory(&b, 4).unwrap(), dir);
        assert!(decode_directory(&b[..16], 4).is_err());
        assert_eq!(directory_pages(64, 512), 1);
        assert_eq!(directory_pages(65, 512), 2);
    }

    #[test]
    fn pack_fills_pages_in_order() {
        // Each record is 16 + 1 + 100 = 117 bytes; 4 fit after a 16-byte header.
        let records: Vec<_> = (0..9u8).map(|i| rec(&[i], &[0; 100])).collect();
        let groups = pack(records, 512);
        let sizes: Vec<_> = groups.iter().map(Vec::len).collect();
        assert_eq!(sizes, [4, 4, 1]);
        assert_eq!(groups[2][0].key, [8]);
        assert!(groups.iter().all(|g| used_bytes(g) <= 512));
    }

    #[test]
    fn pack_of_nothing_is_one_empty_page() {
        assert_eq!(pack(Vec::new(), 512), vec![Vec::<Record>::new()]);
    }
}
