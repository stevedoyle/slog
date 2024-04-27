use std::{
    collections::VecDeque,
    error::Error,
    fs::{self},
};

use clap::Parser;
use env_logger::Builder;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct PspConfig {
    master_keys: [PspMasterKey; 2],
    spi: u32,
    mode: PspEncap,
    algo: CryptoAlg,
    crypto_offset: u32,
    include_vc: bool,
}

pub type PspMasterKey = [u8; 32];

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub enum PspEncap {
    #[default]
    Transport,
    Tunnel,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub enum CryptoAlg {
    AesGcm128,
    #[default]
    AesGcm256,
}

/// Example program to parse a configuration file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CfgParseArgs {
    /// Configuration file to parse.
    #[arg(short, long, default_value_t = String::from("sample.cfg"))]
    file: String,

    #[arg(long, default_value_t = log::LevelFilter::Info)]
    log: log::LevelFilter,
}

fn parse_key(line: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let keystr: String = line.split(' ').collect();
    let key = hex::decode(keystr)?;
    Ok(key)
}

fn parse_spi(line: &str) -> Result<u32, Box<dyn Error>> {
    let spi = u32::from_str_radix(line, 16)?;

    Ok(spi)
}

fn key_to_string(key: &[u8]) -> String {
    key.into_iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_cfg_file(cfg_file: &str) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(cfg_file)?;
    debug!("Contents: {contents}");
    let mut lines: VecDeque<&str> = contents.split('\n').collect();
    lines = lines.into_iter().map(|line| line.trim()).collect();
    debug!("{:?}", lines);

    let key1str = lines.pop_front().ok_or("Key 1 not found")?;
    let key1 = parse_key(key1str)?;
    info!("Key 1: {}", key_to_string(&key1));

    let key2str = lines.pop_front().ok_or("Key 2 not found")?;
    let key2 = parse_key(key2str)?;
    info!("Key 2: {}", key_to_string(&key2));

    let spistr = lines.pop_front().ok_or("SPI not found")?;
    let spi = parse_spi(spistr)?;
    info!("SPI  : {:08X}", spi);

    let mode = lines.pop_front().ok_or("Mode not found")?;
    info!("Mode : {mode}");

    let alg = lines.pop_front().ok_or("Algorithm not found")?;
    info!("Alg  : {alg}");

    let linestr = lines.pop_front().ok_or("No transport crypt offset")?;
    let transport_crypt_off: u8 = linestr.parse()?;
    info!("tco  : {transport_crypt_off}");

    let linestr = lines.pop_front().ok_or("No ipv4 tunnel crypt offset")?;
    let ipv4_tunnel_crypt_off: u8 = linestr.parse()?;
    info!("v4tco: {ipv4_tunnel_crypt_off}");

    let linestr = lines.pop_front().ok_or("No ipv6 tunnel crypt offset")?;
    let ipv6_tunnel_crypt_off: u8 = linestr.parse()?;
    info!("v6tco: {ipv6_tunnel_crypt_off}");

    let linestr = lines.pop_front().ok_or("No vc setting")?;
    let vc: bool = match linestr {
        "vc" => true,
        _ => false,
    };
    info!("vc: {vc}");

    Ok(())
}

fn main() {
    let args = CfgParseArgs::parse();
    Builder::new()
        .filter(None, args.log)
        .format_timestamp(None)
        .format_target(false)
        .format_level(false)
        .init();

    info!("Config file: {}", args.file);
    let err = parse_cfg_file(&args.file);

    if let Err(err) = err {
        eprintln!("Error: {err}");
    }
}
