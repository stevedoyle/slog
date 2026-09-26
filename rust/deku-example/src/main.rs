use deku::{DekuContainerRead, DekuContainerWrite, DekuRead, DekuWrite};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
struct RequestDescriptor {
    #[deku(bits = 1)]
    valid: u8,
    #[deku(bits = 2)]
    dl: u8,
    #[deku(bits = 5)]
    reserved0: u8,
    #[deku(bits = 8)]
    command: u8,
    #[deku(bits = 8)]
    service: u8,
    #[deku(bits = 8)]
    reserved1: u8,

    #[deku(bits = 16)]
    reserved2: u16,
    #[deku(bits = 16)]
    flags: u16,

    #[deku(bits = 64)]
    source_address: u64,
    #[deku(bits = 64)]
    destination_address: u64,
    #[deku(bits = 32)]
    source_len: u32,
    #[deku(bits = 32)]
    destination_len: u32,
}

fn main() {
    let request = RequestDescriptor {
        valid: 1,
        dl: 3,
        reserved0: 0,
        command: 0x01,
        service: 0x02,
        reserved1: 0,
        reserved2: 0,
        flags: 0,
        source_address: 0x123456789ABCDEF0,
        destination_address: 0xFEDCBA9876543210,
        source_len: 128,
        destination_len: 256,
    };

    let encoded = request.to_bytes().unwrap();
    assert_eq!(encoded.len(), 32); // Check the size of the serialized data
    println!("Serialized RequestDescriptor: {:?}", encoded);
    hexdump::hexdump(&encoded);

    let (remaining, parsed_request) = RequestDescriptor::from_bytes((&encoded, 0)).unwrap();
    assert!(
        remaining.0.is_empty(),
        "There should be no remaining bytes after parsing"
    );
    assert_eq!(request, parsed_request);
    println!("Parsed RequestDescriptor: {:?}", parsed_request);
}
