When working with network packets or hardware devices, it is common for packets
and device descriptors to contain fields which are not byte multiples. For
example, a field may be a group of flags or bitfields.

There are many ways to create structures to represent these packets or
descriptors. In C/C++ we use bitfields within structure types. In Rust, there is
no native bitfield support within std, but there are several crates that allow
bitfield types to be defined and manipuated. They often take the approach of
using a Rust newtype aronud a primitive u8/u16/u32 (whatever is needed to hold
the bitfield collection) and then defining an API for interacting with the
fields that are to be packed within the primitive type.

The code in this crate was hacked together as an experiment in how to use a Rust
newtype to create such an abstract type and to define and implement some API
functions and traits. It's not meant to be a complete example, just an
experiment to help build a basic understanding of how to do this in Rust.

The implementation is based around a builder pattern for constructing the
bitfield. It's a rather basic builder pattern without a separation of the data
type being built and the the builder.
