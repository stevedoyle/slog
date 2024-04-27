use pnet_macros::packet;
use pnet_macros_support::{types::*, packet::{PacketSize, Packet}};

/// Represents a PSP Packet.
#[packet]
pub struct Psp {
    next_hdr: u8,
    hdr_ext_len: u8,
    r: u2,
    crypt_offset: u6,
    s: u1,
    d: u1,
    version: u4,
    vc: u1,
    always1: u1,
    spi: u32be,
    iv: u64be,
    #[payload]
    payload: Vec<u8>,
}

#[test]
fn psp_header_test() {
    let buf = vec![
        0x12, 0x23, 0x34, 0x56, 0x00, 0x00, 0x00, 0x03,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let psp = PspPacket::new(&buf).unwrap();
    assert_eq!(psp.get_spi(), 3);
    assert_eq!(psp.get_iv(), 0x0102030405060708);
    assert_eq!(psp.payload().len(), 24);
    assert_eq!(psp.packet().len(), 40);
}
