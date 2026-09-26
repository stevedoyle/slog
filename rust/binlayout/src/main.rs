use bin_layout::{Decoder, Encoder};

#[derive(Debug, Encoder, Decoder)]
struct RequestDescriptor {
    valid_dl: u8,
    service_id: u8,
    command_id: u8,
    reserved0: u8,
    reserved1: u16,
    flags: u16,
    source_buffer_address: u64,
    destination_buffer_address: u64,
    source_buffer_size: u32,
    destination_buffer_size: u32,
}

fn main() {
    // Example usage of the RequestDescriptor struct
    let request = RequestDescriptor {
        valid_dl: 0xE0,
        service_id: 1,
        command_id: 2,
        reserved0: 0,
        reserved1: 0,
        flags: 0,
        source_buffer_address: 0x1000_2000,
        destination_buffer_address: 0x3000_4000,
        source_buffer_size: 1024,
        destination_buffer_size: 2048,
    };

    let encoded = request.encode();
    println!("Encoded RequestDescriptor: {:?}", encoded);
    hexdump::hexdump(&encoded);
    let decoded: RequestDescriptor = RequestDescriptor::decode(&encoded).unwrap();
    println!("Decoded RequestDescriptor: {:?}", decoded);
}
