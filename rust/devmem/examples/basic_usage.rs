use devmem::{AccessWidth, DevMem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("devmem Library Usage Example");
    println!("============================\n");

    let devmem = match DevMem::new() {
        Ok(dm) => dm,
        Err(e) => {
            eprintln!("Failed to open /dev/mem: {}", e);
            eprintln!("This example requires root privileges or CAP_SYS_RAWIO capability");
            return Ok(());
        }
    };

    let test_address = 0x1000;

    println!("Reading different widths from address 0x{:08X}:\n", test_address);

    match devmem.read(test_address, AccessWidth::Byte) {
        Ok(value) => println!("Byte (8-bit):      0x{:02X}", value),
        Err(e) => println!("Byte read failed: {}", e),
    }

    match devmem.read(test_address, AccessWidth::Word) {
        Ok(value) => println!("Word (16-bit):     0x{:04X}", value),
        Err(e) => println!("Word read failed: {}", e),
    }

    match devmem.read(test_address, AccessWidth::Long) {
        Ok(value) => println!("Long (32-bit):     0x{:08X}", value),
        Err(e) => println!("Long read failed: {}", e),
    }

    match devmem.read(test_address, AccessWidth::LongLong) {
        Ok(value) => println!("LongLong (64-bit): 0x{:016X}", value),
        Err(e) => println!("LongLong read failed: {}", e),
    }

    println!("\nNote: Actual values depend on your hardware and memory contents.");
    println!("Be careful when writing to physical memory addresses!");

    Ok(())
}
