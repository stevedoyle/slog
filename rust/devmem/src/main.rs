use clap::Parser;
use devmem::{AccessWidth, DevMem};
use std::process;

#[derive(Parser, Debug)]
#[command(name = "devmem")]
#[command(about = "Read/write from physical memory addresses", long_about = None)]
#[command(version)]
struct Args {
    #[arg(help = "Physical address (in hex with 0x prefix or decimal)")]
    address: String,

    #[arg(help = "Access width: b(yte), h(alfword/word), w(ord/long), or q(uad/longlong) [default: w]")]
    width: Option<String>,

    #[arg(help = "Value to write (in hex with 0x prefix or decimal)")]
    value: Option<String>,
}

fn parse_number(s: &str) -> Result<u64, String> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex number: {}", e))
    } else {
        s.parse::<u64>()
            .map_err(|e| format!("Invalid number: {}", e))
    }
}

fn parse_width(s: Option<&str>) -> Result<AccessWidth, String> {
    match s {
        Some("b") | Some("B") => Ok(AccessWidth::Byte),
        Some("h") | Some("H") => Ok(AccessWidth::Word),
        Some("w") | Some("W") | None => Ok(AccessWidth::Long),
        Some("q") | Some("Q") => Ok(AccessWidth::LongLong),
        Some(other) => Err(format!(
            "Invalid width '{}'. Use b(yte), h(alfword), w(ord), or q(uad)",
            other
        )),
    }
}

fn main() {
    let args = Args::parse();

    let address = match parse_number(&args.address) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Error parsing address: {}", e);
            process::exit(1);
        }
    };

    let width = match parse_width(args.width.as_deref()) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let devmem = match DevMem::new() {
        Ok(dm) => dm,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Note: This program requires root privileges or CAP_SYS_RAWIO capability");
            process::exit(1);
        }
    };

    match args.value {
        None => {
            match devmem.read(address, width) {
                Ok(value) => {
                    println!(
                        "Value at address 0x{:08X} ({}): 0x{:0width$X}",
                        address,
                        width_name(width),
                        value,
                        width = width.size() * 2
                    );
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    process::exit(1);
                }
            }
        }
        Some(value_str) => {
            let value = match parse_number(&value_str) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error parsing value: {}", e);
                    process::exit(1);
                }
            };

            match devmem.read(address, width) {
                Ok(old_value) => {
                    println!(
                        "Old value at address 0x{:08X} ({}): 0x{:0width$X}",
                        address,
                        width_name(width),
                        old_value,
                        width = width.size() * 2
                    );
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    process::exit(1);
                }
            }

            match devmem.write(address, value, width) {
                Ok(()) => {
                    println!(
                        "Write 0x{:0width$X} to address 0x{:08X} ({})",
                        value,
                        address,
                        width_name(width),
                        width = width.size() * 2
                    );
                }
                Err(e) => {
                    eprintln!("Write error: {}", e);
                    process::exit(1);
                }
            }

            match devmem.read(address, width) {
                Ok(new_value) => {
                    println!(
                        "New value at address 0x{:08X} ({}): 0x{:0width$X}",
                        address,
                        width_name(width),
                        new_value,
                        width = width.size() * 2
                    );
                }
                Err(e) => {
                    eprintln!("Read-back error: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}

fn width_name(width: AccessWidth) -> &'static str {
    match width {
        AccessWidth::Byte => "byte",
        AccessWidth::Word => "word",
        AccessWidth::Long => "long",
        AccessWidth::LongLong => "longlong",
    }
}
