use etherparse::{
    ip_number, Ethernet2Header, Ipv4Header, PacketBuilder, PacketHeaders, SerializedSize,
    SlicedPacket,
};
use pcap_file::pcap::PcapReader;
use std::fs::File;

fn slice_packet(pkt: &[u8]) {
    match SlicedPacket::from_ethernet(pkt) {
        Err(value) => println!("Err {:?}", value),
        Ok(value) => {
            println!("link: {:?}", value.link);
            println!("vlan: {:?}", value.vlan);
            println!("ip: {:?}", value.ip);
            println!("transport: {:?}", value.transport);
            println!("payload length: {}", value.payload.len());
        }
    }
}

fn parse_packet(pkt: &[u8]) -> Option<PacketHeaders> {
    match PacketHeaders::from_ethernet_slice(pkt) {
        Err(value) => {
            println!("Err {:?}", value);
            None
        }
        Ok(value) => {
            println!("link: {:?}", value.link);
            println!("vlan: {:?}", value.vlan);
            println!("ip: {:?}", value.ip);
            println!("transport: {:?}", value.transport);
            println!("payload length: {:?}", value.payload.len());
            Some(value)
        }
    }
}

fn parse_pcap_file(path: &str) {
    let fin = File::open(path).expect("Error opening file");
    let mut pcap_reader = PcapReader::new(fin).unwrap();

    println!("Packets from file: {} ...\n", path);
    // Read packets from the pcap file
    while let Some(pkt) = pcap_reader.next_packet() {
        let pkt = pkt.unwrap();
        println!("Slicing ...");
        slice_packet(&pkt.data);
        println!();
        println!("Parsing ...");
        parse_packet(&pkt.data);
        println!("");
    }
}

fn build_eth_ipv4_udp_pkt() {
    let builder = PacketBuilder::ethernet2([1, 2, 3, 4, 5, 6], [7, 8, 9, 10, 11, 12])
        .ipv4([192, 168, 1, 1], [192, 168, 1, 2], 32)
        .udp(10001, 10002);
    let payload = [0x5Au8; 64];
    let mut pkt = Vec::<u8>::with_capacity(builder.size(payload.len()));
    builder.write(&mut pkt, &payload).unwrap();
    println!("Wire packet: {:02X?}", pkt);
}

fn encap_pkt(pkt: &[u8]) -> Vec<u8> {
    let mut out = Vec::<u8>::with_capacity(pkt.len() + Ipv4Header::SERIALIZED_SIZE);

    let (in_eth, in_eth_payload) = Ethernet2Header::from_slice(pkt).unwrap();
    in_eth.write(&mut out).unwrap();

    let out_ip_tun_hdr = Ipv4Header::new(
        in_eth_payload.len() as u16,
        32, // TTL
        ip_number::IPV4,
        [192, 168, 10, 2],
        [192, 168, 20, 2],
    );
    out_ip_tun_hdr.write(&mut out).unwrap();
    out.extend_from_slice(in_eth_payload);
    out
}

fn encap_pkts_from_pcap_file(path: &str) {
    let fin = File::open(path).expect("Error opening file");
    let mut pcap_reader = PcapReader::new(fin).unwrap();

    println!("Packets from file: {} ...\n", path);
    // Read packets from the pcap file
    while let Some(pkt) = pcap_reader.next_packet() {
        let pkt = pkt.unwrap();
        let out_pkt = encap_pkt(&pkt.data);
        println!(
            "Input packet (1st 100 bytes):\n{:02X?}",
            pkt.data.chunks(100).next().unwrap()
        );
        println!(
            "Output packet (1st 100 bytes):\n{:02X?}",
            out_pkt.chunks(100).next().unwrap()
        );
        println!("");
    }
}

fn main() {
    parse_pcap_file("pcap/v4_cleartext.pcap");
    // parse_pcap_file("pcap/v6_cleartext.pcap");
    // parse_pcap_file("pcap/v4_cleartext_empty.pcap");
    // parse_pcap_file("pcap/v6_cleartext_empty.pcap");
    // parse_pcap_file("pcap/vxlan.pcap");

    build_eth_ipv4_udp_pkt();

    encap_pkts_from_pcap_file("pcap/v4_cleartext.pcap");
}
