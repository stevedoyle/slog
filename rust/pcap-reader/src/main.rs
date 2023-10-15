use pcap_file::pcap::PcapReader;
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::{IpNextHeaderProtocol, IpNextHeaderProtocols},
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    udp::UdpPacket,
    Packet,
};
use std::{fs::File, net::IpAddr};

fn handle_udp_packet(source: IpAddr, destination: IpAddr, packet: &[u8]) {
    let udp = UdpPacket::new(packet);

    if let Some(udp) = udp {
        println!(
            "UDP: {}:{} > {}:{}; length: {}",
            source,
            udp.get_source(),
            destination,
            udp.get_destination(),
            udp.get_length()
        );
    } else {
        println!("Malformed UDP packet");
    }
}

fn handle_transport_pdu(
    source: IpAddr,
    destination: IpAddr,
    protocol: IpNextHeaderProtocol,
    packet: &[u8],
) {
    match protocol {
        IpNextHeaderProtocols::Udp => {
            handle_udp_packet(source, destination, packet);
        }
        _ => println!(
            "Unknown {} packet: {} > {}; protocol: {:?} length: {}",
            match source {
                IpAddr::V4(..) => "IPv4",
                _ => "IPv6,",
            },
            source,
            destination,
            protocol,
            packet.len()
        ),
    }
}

fn handle_ipv4_packet(ether: &EthernetPacket) {
    let ip = Ipv4Packet::new(ether.payload());
    if let Some(ip) = ip {
        println!(
            "IPv4: {} > {}; protocol: {}, length: {}",
            ip.get_source(),
            ip.get_destination(),
            ip.get_next_level_protocol(),
            ip.get_total_length()
        );

        handle_transport_pdu(
            IpAddr::V4(ip.get_source()),
            IpAddr::V4(ip.get_destination()),
            ip.get_next_level_protocol(),
            ip.payload(),
        );
    }
}

fn handle_ipv6_packet(ether: &EthernetPacket) {
    let ip = Ipv6Packet::new(ether.payload());
    if let Some(ip) = ip {
        println!(
            "IPv6: {} > {}; protocol: {}, length: {}",
            ip.get_destination(),
            ip.get_source(),
            ip.get_next_header(),
            ip.packet().len()
        );

        handle_transport_pdu(
            IpAddr::V6(ip.get_source()),
            IpAddr::V6(ip.get_destination()),
            ip.get_next_header(),
            ip.payload(),
        );
    }
}

fn handle_ethernet_frame(ether: &EthernetPacket) {

    println!("Ethernet: {} > {}; ethertype: {}, length: {}",
        ether.get_source(),
        ether.get_destination(),
        ether.get_ethertype(),
        ether.packet().len()
    );

    match ether.get_ethertype() {
        EtherTypes::Ipv4 => handle_ipv4_packet(ether),
        EtherTypes::Ipv6 => handle_ipv6_packet(ether),
        _ => println!(
            "Unknown packet: {} > {}; ethertype: {:?}, length: {}",
            ether.get_source(),
            ether.get_destination(),
            ether.get_ethertype(),
            ether.packet().len()
        ),
    }
}

fn parse_pcap_file(path: &str) {
    let fin = File::open(path).expect("Error opening file");
    let mut pcap_reader = PcapReader::new(fin).unwrap();

    println!("Packets from file: {} ...\n", path);
    // Read packets from the pcap file
    while let Some(pkt) = pcap_reader.next_packet() {
        let pkt = pkt.unwrap();
        let ether = EthernetPacket::new(&pkt.data).unwrap();
        handle_ethernet_frame(&ether);
        println!("");
    }
}

fn main() {
    parse_pcap_file("pcap/v4_cleartext.pcap");
    parse_pcap_file("pcap/v6_cleartext.pcap");
    parse_pcap_file("pcap/v4_cleartext_empty.pcap");
    parse_pcap_file("pcap/v6_cleartext_empty.pcap");
}
