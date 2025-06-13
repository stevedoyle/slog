#![allow(dead_code)]

use std::mem;

#[derive(Copy, Clone, Debug)]
struct MyStruct {
    a: u8,
    b: u32,
    c: u16,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct MyCStruct {
    a: u8,
    b: u32,
    c: u16,
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
struct PackedStruct {
    a: u8,
    b: u32,
    c: u16,
}

#[derive(Copy, Clone, Debug)]
#[repr(align(16))]
struct SixteenByteAligned {
    a: u8,
    b: u32,
    c: u16,
}

#[derive(Copy, Clone, Debug)]
#[repr(C, align(16))]
struct SixteenByteCAligned {
    a: u8,
    b: u32,
    c: u16,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct StructWithArray {
    data: [MyCStruct; 10],
}

fn main() {
    println!("---- Default Rust Layout ----");
    println!("Size of MyStruct: {} bytes", mem::size_of::<MyStruct>());
    println!(
        "Alignment of MyStruct: {} bytes",
        mem::align_of::<MyStruct>()
    );

    let array = [MyStruct { a: 1, b: 2, c: 3 }; 10];
    println!(
        "Size of array of 10 MyStructs: {} bytes",
        mem::size_of_val(&array)
    );

    println!();
    println!("---- C/C++ Compatible Layout ----");
    println!("Size of MyCStruct: {} bytes", mem::size_of::<MyCStruct>());
    println!(
        "Alignment of MyCStruct: {} bytes",
        mem::align_of::<MyCStruct>()
    );

    let array = [MyCStruct { a: 1, b: 2, c: 3 }; 10];
    println!(
        "Size of array of 10 MyCStructs: {} bytes",
        mem::size_of_val(&array)
    );

    println!();
    println!("---- Packed Layout ----");
    println!(
        "Size of PackedStruct: {} bytes",
        mem::size_of::<PackedStruct>()
    );
    println!(
        "Alignment of PackedStruct: {} bytes",
        mem::align_of::<PackedStruct>()
    );

    let array = [PackedStruct { a: 1, b: 2, c: 3 }; 10];
    println!(
        "Size of array of 10 PackedStructs: {} bytes",
        mem::size_of_val(&array)
    );

    println!();
    println!();
    println!("---- Custom Alignment ----");
    println!(
        "Size of SixteenByteAligned: {} bytes",
        mem::size_of::<SixteenByteAligned>()
    );
    println!(
        "Alignment of SixteenByteAligned: {} bytes",
        mem::align_of::<SixteenByteAligned>()
    );
    println!(
        "Size of array of 10 SixteenByteAligned: {} bytes",
        mem::size_of_val(&[SixteenByteAligned { a: 1, b: 2, c: 3 }; 10])
    );

    println!();
    println!();
    println!("---- Custom Alignment - Multiple Alignment Specifiers ----");
    println!(
        "Size of SixteenByteCAligned: {} bytes",
        mem::size_of::<SixteenByteCAligned>()
    );
    println!(
        "Alignment of SixteenByteCAligned: {} bytes",
        mem::align_of::<SixteenByteCAligned>()
    );
    println!(
        "Size of array of 10 SixteenCByteAligned: {} bytes",
        mem::size_of_val(&[SixteenByteCAligned { a: 1, b: 2, c: 3 }; 10])
    );

    println!();
    println!();
    println!("---- Array Alignment ----");
    println!(
        "Size of StructWithArray: {} bytes",
        mem::size_of::<StructWithArray>()
    );
    println!(
        "Alignment of StructWithArray: {} bytes",
        mem::align_of::<StructWithArray>()
    );
}
