use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};
use tiny_keccak::{Hasher, Xof};

/// Experiment to determine if the SHAKE Xof output is the same regardless of the amount of data
/// squeezed from it in one go. More specifically, if the SHAKE Xof output is the same when
/// squeezing 2 bytes at a time as when squeezing 256 bytes at a time.

fn shake_xof_2_bytes_per_iter(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut output = vec![0u8; output_len];
    let mut xof = tiny_keccak::Shake::v128();
    xof.update(input);
    for i in (0..output_len).step_by(2) {
        xof.squeeze(&mut output[i..i + 2]);
    }
    //    println!("{:?}", output);
    output
}

fn shake_xof_single_iter(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut output = vec![0u8; output_len];
    let mut xof = tiny_keccak::Shake::v128();
    xof.update(input);
    xof.squeeze(&mut output);
    //    println!("{:?}", output);
    output
}

fn using_tiny_keccak() {
    let input = b"Hello, world!";
    let a = shake_xof_2_bytes_per_iter(input, 256);
    let b = shake_xof_single_iter(input, 256);

    if a == b {
        println!("The SHAKE Xof output is the same regardless of the amount of data squeezed from it in one go.");
    } else {
        println!("The SHAKE Xof output is different depending on the amount of data squeezed from it in one go.");
    }
}

fn sha3_shake_xof(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut output = vec![0u8; output_len];
    let mut hasher = Shake128::default();
    hasher.update(input);
    hasher.finalize_xof().read(&mut output);
    output
}

fn sha3_shake_xof_multi_iter(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut output = vec![0u8; output_len];
    let mut hasher = Shake128::default();
    hasher.update(input);
    let mut reader = hasher.finalize_xof();
    for i in (0..output_len).step_by(2) {
        reader.read(&mut output[i..i + 2]);
    }
    output
}

fn using_sha3() {
    let input = b"Hello, world!";
    let a = sha3_shake_xof(input, 256);
    let b = sha3_shake_xof_multi_iter(input, 256);

    if a == b {
        println!("The SHAKE Xof output is the same regardless of the amount of data squeezed from it in one go.");
    } else {
        println!("The SHAKE Xof output is different depending on the amount of data squeezed from it in one go.");
    }
}

fn main() {
    using_tiny_keccak();
    using_sha3();
}
