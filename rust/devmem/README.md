# devmem - Physical Memory Access Tool

A Rust implementation of a devmem utility for reading and writing physical memory addresses via `/dev/mem`. This tool is commonly used for accessing memory-mapped device registers on embedded Linux systems.

## Features

- Read from physical memory addresses
- Write to physical memory addresses
- Support for multiple access widths (byte, word, long, longlong)
- Proper memory alignment checking
- Safe memory mapping using mmap/munmap
- Both library and command-line interface

## Building

```bash
cargo build --release
```

The binary will be located at `target/release/devmem`.

## Usage

The tool requires root privileges or `CAP_SYS_RAWIO` capability to access `/dev/mem`.

### Reading Memory

```bash
sudo ./devmem <address> [width]
```

Example:
```bash
# Read a 32-bit (long) value from address 0x40000000
sudo ./devmem 0x40000000

# Read a byte from address 0x40000000
sudo ./devmem 0x40000000 b

# Read a 64-bit (quad/longlong) value
sudo ./devmem 0x40000000 q
```

### Writing Memory

```bash
sudo ./devmem <address> [width] <value>
```

Example:
```bash
# Write 0x12345678 to address 0x40000000 (32-bit)
sudo ./devmem 0x40000000 w 0x12345678

# Write 0xFF to address 0x40000000 (byte)
sudo ./devmem 0x40000000 b 0xFF

# Write 64-bit value
sudo ./devmem 0x40000000 q 0x123456789ABCDEF0
```

### Access Widths

- `b` or `B`: Byte (8 bits)
- `h` or `H`: Halfword/Word (16 bits)
- `w` or `W`: Word/Long (32 bits) - default
- `q` or `Q`: Quad/Longlong (64 bits)

### Address and Value Formats

Both addresses and values can be specified in:
- Hexadecimal with `0x` prefix: `0x40000000`
- Decimal without prefix: `1073741824`

## Library Usage

The project also provides a library that can be used in other Rust programs:

```rust
use devmem::{DevMem, AccessWidth};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devmem = DevMem::new()?;

    // Read a 32-bit value
    let value = devmem.read(0x40000000, AccessWidth::Long)?;
    println!("Value: 0x{:08X}", value);

    // Write a value
    devmem.write(0x40000000, 0x12345678, AccessWidth::Long)?;

    // Read-modify-write operation
    let old_value = devmem.read_modify_write(0x40000000, 0xFFFFFFFF, AccessWidth::Long)?;

    Ok(())
}
```

## Safety and Security

This tool provides direct access to physical memory and should be used with extreme caution:

- Requires root privileges or `CAP_SYS_RAWIO` capability
- Can potentially crash the system if used incorrectly
- Only use on addresses that correspond to memory-mapped device registers
- Ensure proper alignment for the access width
- The library uses `unsafe` Rust for mmap operations

## Implementation Details

- Uses `mmap()` to map physical memory into the process address space
- Opens `/dev/mem` with `O_SYNC` flag for synchronized access
- Automatically handles page alignment for memory mapping
- Validates address alignment for the requested access width
- Properly unmaps memory after each operation

## Testing

```bash
cargo test
```

Note: The actual memory access tests require root privileges and hardware access, so the test suite focuses on unit tests for helper functions.

## Comparison with Original devmem2

This Rust implementation provides the same core functionality as the original C devmem2:
- Same command-line interface
- Same access widths
- Same output format
- Additional safety through Rust's type system
- More detailed error messages

## License

MIT License - see [LICENSE](LICENSE) file for details.

This project is for educational and system development purposes. Use at your own risk.
