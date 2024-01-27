use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use clap::Parser;
use pretty_hex::PrettyHex;
use std::{
    fmt,
    mem::size_of,
    net::{SocketAddr, UdpSocket},
    str,
};
use thiserror::Error;

#[derive(Debug, Default, PartialEq)]
struct DnsHeader {
    id: u16,
    flags: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

impl fmt::Display for DnsHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "id:{} flags:{} questions:{} answers:{} authorities:{} additionals:{}",
            self.id, self.flags, self.qdcount, self.ancount, self.nscount, self.arcount
        )
    }
}

impl DnsHeader {
    fn parse(buf: &[u8]) -> Result<(Self, usize)> {
        let mut hdr = DnsHeader::default();
        if buf.len() < size_of::<DnsHeader>() {
            return Err(ParseError::InvalidHeader.into());
        }
        let mut pos = buf;
        hdr.id = pos.read_u16::<BigEndian>()?;
        hdr.flags = pos.read_u16::<BigEndian>()?;
        hdr.qdcount = pos.read_u16::<BigEndian>()?;
        hdr.ancount = pos.read_u16::<BigEndian>()?;
        hdr.nscount = pos.read_u16::<BigEndian>()?;
        hdr.arcount = pos.read_u16::<BigEndian>()?;

        Ok((hdr, size_of::<DnsHeader>()))
    }
}

#[derive(Debug, Default, PartialEq)]
struct DnsQuestion {
    name: String,
    qtype: u16,
    qclass: u16,
}

impl fmt::Display for DnsQuestion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "name:{} type:{} class:{}",
            self.name, self.qtype, self.qclass,
        )
    }
}

impl DnsQuestion {
    fn parse(buf: &[u8]) -> (Self, usize) {
        let mut dnsq = DnsQuestion::default();
        // Name is encoded as described in RFC 1035, Section 4.1.2
        // Lengh octet followed by that number of octets.
        // Repeated until a length octet with value 0 is found.
        let mut parts = Vec::new();
        let mut pos = 0;
        loop {
            let count = buf[pos];
            pos += 1;
            if count == 0 {
                break;
            }
            parts.push(String::from_utf8(buf[pos..(pos + count as usize)].to_vec()).unwrap());
            pos += count as usize;
        }
        dnsq.name = parts.join(".");

        dnsq.qtype = u16::from_be_bytes(buf[pos..(pos + 2)].try_into().unwrap());
        pos += 2;
        dnsq.qclass = u16::from_be_bytes(buf[pos..(pos + 2)].try_into().unwrap());
        pos += 2;

        (dnsq, pos)
    }
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("invalid header")]
    InvalidHeader,
}

fn run_server(addr: &str) -> Result<()> {
    let socket = UdpSocket::bind(addr)?;

    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.
    let mut buf = [0; 2048];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        println!("MSG received: {amt} bytes from {src}");
        println!("{:?}", &buf[0..amt].hex_dump());
        handle_dns_request(&buf[..amt], src, &socket)?;
    }
}

fn handle_dns_request(req: &[u8], src: SocketAddr, sock: &UdpSocket) -> Result<()> {
    let (dnshdr, size) = DnsHeader::parse(&req)?;
    println!("Header - {dnshdr}");
    let mut offset = size;
    for _ in 0..dnshdr.qdcount {
        let (qd, size) = DnsQuestion::parse(&req[offset..]);
        offset += size;
        println!("Question - {qd}")
    }

    let mut buf = [0; 2048];
    let dns_socket = UdpSocket::bind("0.0.0.0:10001")?;
    let external_dns_addr = "8.8.8.8:53";

    dns_socket
        .send_to(req, external_dns_addr)
        .expect("couldn't send data");
    println!("DNS Request Forwarded to {external_dns_addr}");

    let (amt, src1) = dns_socket.recv_from(&mut buf)?;
    println!("MSG received: {amt} bytes from {src1}");

    println!("{:?}", &buf[0..amt].hex_dump());
    sock.send_to(&buf[..amt], src)
        .expect("couldn't send response");

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// DNS server port
    #[arg(short, long, default_value_t = 53)]
    port: u16,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let port = args.port;
    let address = format!("127.0.0.1:{port}");
    run_server(&address)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_pkt_hdr_parse() {
        let mut dns: Vec<u8> = Vec::new();
        dns.extend_from_slice(&40500u16.to_be_bytes());
        dns.extend_from_slice(&288u16.to_be_bytes());
        dns.extend_from_slice(&1u16.to_be_bytes());
        dns.extend_from_slice(&0u16.to_be_bytes());
        dns.extend_from_slice(&0u16.to_be_bytes());
        dns.extend_from_slice(&1u16.to_be_bytes());

        let expected = DnsHeader {
            id: 40500,
            flags: 288,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 1,
        };

        let (dnshdr, size) = DnsHeader::parse(&dns).unwrap();
        assert_eq!(dnshdr, expected);
        assert_eq!(size, dns.len());
    }

    #[test]
    fn test_dns_question_parse() {
        let buf = b"\x03www\x06google\x03com\x00\x12\x34\x56\x78";
        let (qd, size) = DnsQuestion::parse(buf);
        assert_eq!(size, 20);
        assert_eq!(qd.name, "www.google.com");
        assert_eq!(qd.qtype, 0x1234);
        assert_eq!(qd.qclass, 0x5678);
    }
}
