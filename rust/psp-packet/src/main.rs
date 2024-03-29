extern crate pnet;
extern crate pnet_macros_support;

use pnet::packet::{Packet, MutablePacket};

mod packet;
use packet::psp::{PspPacket, MutablePspPacket};

fn main() {
    let buf = vec![
        0x12, 0x23, 0x34, 0x56, 0x00, 0x00, 0x00, 0x03,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let psp = PspPacket::new(&buf).unwrap();
    println!("{:?}", psp);
    println!("SPI:     0x{:08x}", psp.get_spi());
    println!("Seq Num: 0x{:08x}", psp.get_next_hdr());
    println!("PSP packet length:  {}", psp.packet().len());
    println!("PSP payload length: {}", psp.payload().len());
}