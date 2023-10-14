use bitfield::bitfield;
use serde::{Deserialize, Serialize};
use derive_builder::Builder;

/// A structure that represents a packet header or a descriptor that contains
/// some bitfield elements. For example, it may contain a collection of flags.
/// We want to be able to set/get the individual fields within the bitfield and
/// we also want to be able to use the value of the compound type for
/// serialization operations.
///
/// There are various ways to approach this, from using a new type to wrap a u8
/// and add some getters/setters and default values to using one of the many
/// exising bitfield crates. The approach take here is to use the `bitfield`
/// crate.
#[derive(Builder, Serialize, Deserialize, Debug, Default)]
#[builder(default)]
pub struct PspHeader {
    /// An IP protocol number, identifying the type of the next header.
    /// For example:
    /// - 6 for transport mode when next header is TCP
    /// - 17 for transport mode when next header is UDP
    /// - 4 for tunnel mode when next header is IPv4
    /// - 41 for tunnel mode when next header is IPv6
    next_hdr: u8,
    /// The number of 4 byte words in the header in addition to the mandatory 16
    /// bytes of header fields.
    hdr_ext_len: u8,
    crypt_off: u8,
    /// A collection of flags.
    flags: PspHeaderFlags,
    spi: u32,
    iv: u64,
}

bitfield! {
    #[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
    pub struct PspHeaderFlags(u8);
    impl Debug;
    s, set_s: 0;
    d, set_d: 1;
    version, set_version: 5, 2;
    vc, set_vc: 6;
    r, set_r: 7;
}

impl Default for PspHeaderFlags {
    fn default() -> Self {
        let mut flags = PspHeaderFlags(0);
        flags.set_s(false);
        flags.set_d(false);
        flags.set_version(PspVersion::PspVer0 as u8);
        flags.set_vc(false);
        flags.set_r(true);
        flags
    }
}

#[derive(Debug)]
pub enum PspVersion {
    PspVer0,
    PspVer1,
    PspVer2,
    PspVer3,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_psp_header_builder() {
        let hdr = PspHeaderBuilder::default()
            .next_hdr(17)
            .spi(0x12345678)
            .iv(0x12345678_9ABCDEF0)
            .build().unwrap();

        assert_eq!(hdr.spi, 0x12345678);
        assert_eq!(hdr.iv, 0x12345678_9ABCDEF0);
        assert_eq!(hdr.next_hdr, 17);
        assert_eq!(hdr.flags.0, 0x80u8);
    }

    #[test]
    fn check_serde() {
        let mut flags = PspHeaderFlags::default();
        flags.set_version(PspVersion::PspVer1 as u8);

        let hdr = PspHeaderBuilder::default()
            .next_hdr(17)
            .spi(0x12345678)
            .iv(0x12345678_9ABCDEF0)
            .flags(flags)
            .build().unwrap();

        let encoded = bincode::serialize(&hdr);
        assert!(encoded.is_ok());
        let encoded = encoded.unwrap();
        assert_eq!(encoded.len(), 16);

        let decoded: Result<PspHeader, Box<bincode::ErrorKind>> =
            bincode::deserialize(&encoded);
        assert!(decoded.is_ok());
        assert_eq!(hdr.spi, 0x12345678);
        assert_eq!(hdr.iv, 0x12345678_9ABCDEF0);
        assert_eq!(hdr.next_hdr, 17);
        assert_eq!(hdr.flags.0, 0x84u8);
    }
}
