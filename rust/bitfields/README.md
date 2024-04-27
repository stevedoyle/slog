
When working with network packets or hardware devices, it is common for packets
and device descriptors to contain fields which are not byte multiples. For
example, a field may be a group of flags or bitfields.

We want to be able to construct a type (struct) that represents the packet
header or device descriptor and be able to set/get the individual fields,
including the bitfields and we also want to be able to use the value of the
compound type for serialization operations.

There are various ways to approach this, from using a new type to wrap a
u8/u16/u32 and add some getters/setters and default values to using one of the
many exising bitfield crates. The approach taken here is to use the `bitfield`
crate. The example here is based around a PSP header.

The definition of the flags / bitfields ...

```Rust
bitfield! {
    #[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
    pub struct PspHeaderFlags(u8);
    impl Debug;
    s, set_s: 0;
    d, set_d: 1;
    version, set_version: 5, 2;
    vc, set_vc: 6;
    r, set_r: 7;
}

impl Default for PspHeaderFlags {
    fn default() -> Self {
        let mut flags = PspHeaderFlags(0);
        flags.set_s(false);
        flags.set_d(false);
        flags.set_version(PspVersion::PspVer0 as u8);
        flags.set_vc(false);
        flags.set_r(true);
        flags
    }
}
```
And the usage in the packet header structure ...

```Rust
#[derive(Builder, Serialize, Deserialize, Debug, Default)]
#[builder(default)]
pub struct PspHeader {
    next_hdr: u8,
    hdr_ext_len: u8,
    crypt_off: u8,
    /// A collection of flags.
    flags: PspHeaderFlags,
    spi: u32,
    iv: u64,
}
```
