use openssl::hash::MessageDigest;
use openssl::symm::Cipher;
use openssl_kdf::{KdfArgument, KdfKbMode, KdfMacType, KdfType, perform_kdf};

fn openssl_kdf_ctr_example() {
    let args = [
        &KdfArgument::KbMode(KdfKbMode::Counter),
        &KdfArgument::Mac(KdfMacType::Hmac(MessageDigest::sha256())),
        // Set the salt (called "Label" in SP800-108)
        &KdfArgument::Salt(&[0x12, 0x34]),
        // Set the kb info (called "Context" in SP800-108)
        &KdfArgument::KbInfo(&[0x9a, 0xbc]),
        // Set the key (called "Ki" in SP800-108)
        &KdfArgument::Key(&[0x56, 0x78]),
    ];

    let key_out = perform_kdf(KdfType::KeyBased, &args, 20).unwrap();
    println!("{:?}", hex::encode(key_out));
}
fn openssl_kdf_ctr(
    key: &[u8],
    salt: &[u8],
    info: &[u8],
    output_len: usize,
    ctr_len: u8,
    lbits: u8,
) -> Vec<u8> {
    let args = [
        &KdfArgument::KbMode(KdfKbMode::Counter),
        &KdfArgument::Mac(KdfMacType::Cmac(Cipher::aes_256_cbc())),
        &KdfArgument::Salt(salt),
        &KdfArgument::KbInfo(info),
        &KdfArgument::Key(key),
        &KdfArgument::R(ctr_len),   // Counter length in bits
        &KdfArgument::LBits(lbits), // Output length in bits
    ];

    perform_kdf(KdfType::KeyBased, &args, output_len).unwrap()
}

fn main() {
    openssl_kdf_ctr_example();
}

#[cfg(test)]
mod tests {
    use super::*;

    const MASTER_KEYS: [[u8; 32]; 2] = [
        [
            0x34, 0x44, 0x8a, 0x06, 0x42, 0x92, 0x60, 0x1b, 0x11, 0xa0, 0x97, 0x8f, 0x56, 0xa2,
            0xd3, 0x4c, 0xf3, 0xfc, 0x35, 0xed, 0xe1, 0xa6, 0xbc, 0x04, 0xf8, 0xdb, 0x3e, 0x52,
            0x43, 0xa2, 0xb0, 0xca,
        ],
        [
            0x56, 0x39, 0x52, 0x56, 0x5d, 0x3a, 0x78, 0xae, 0x77, 0x3e, 0xc1, 0xb7, 0x79, 0xf2,
            0xf2, 0xd9, 0x9f, 0x4a, 0x7f, 0x53, 0xa6, 0xfb, 0xb9, 0xb0, 0x7d, 0x5b, 0x71, 0xf3,
            0x93, 0x64, 0xd7, 0x39,
        ],
    ];

    #[test]
    fn test_openssl_kdf_ctr_pspv0_mk0() {
        let expected = vec![
            0x96, 0xc2, 0x2d, 0xc7, 0x99, 0x19, 0x80, 0x90, 0xb7, 0x4b, 0x70, 0xae, 0x46, 0x8e,
            0x4e, 0x30,
        ];
        let actual = openssl_kdf_ctr(
            MASTER_KEYS[0].as_ref(),
            &[0x50, 0x76, 0x30],
            0x12345678u32.to_be_bytes().as_ref(),
            16,
            32,
            32,
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_openssl_kdf_ctr_pspv0_mk1() {
        let expected = vec![
            0x39, 0x46, 0xda, 0x25, 0x54, 0xea, 0xe4, 0x6a, 0xd1, 0xef, 0x77, 0xa6, 0x43, 0x72,
            0xed, 0xc4,
        ];
        let actual = openssl_kdf_ctr(
            MASTER_KEYS[1].as_ref(),
            &[0x50, 0x76, 0x30],
            0x9A345678u32.to_be_bytes().as_ref(),
            16,
            32,
            32,
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_openssl_kdf_ctr_pspv1() {
        let expected = vec![
            0x2b, 0x7d, 0x72, 0x07, 0x4e, 0x42, 0xca, 0x33, 0x44, 0x87, 0xf2, 0x99, 0x0e, 0x3f,
            0x8c, 0x40, 0x37, 0xe4, 0x36, 0xf3, 0x82, 0x83, 0x44, 0x9b, 0x76, 0x46, 0x3e, 0x9b,
            0x7f, 0xb2, 0xe3, 0xde,
        ];
        let actual = openssl_kdf_ctr(
            MASTER_KEYS[0].as_ref(),
            &[0x50, 0x76, 0x31],
            0x12345678u32.to_be_bytes().as_ref(),
            32,
            32,
            32,
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_openssl_kdf_ctr_tss_cluster() {
        let expected = vec![
            0x15, 0x1b, 0x4d, 0xdb, 0x30, 0x11, 0x29, 0x71, 0xdd, 0xef, 0xf3, 0x21, 0x30, 0x00,
            0xee, 0x74, 0xd8, 0xf1, 0x8a, 0xac, 0x21, 0x35, 0x60, 0x1f, 0x1e, 0x52, 0x15, 0xe5,
            0x05, 0xfe, 0xd4, 0x49, 0xa7, 0xc6, 0x99, 0x2c, 0x26, 0xb0, 0xbd, 0x5d, 0xd5, 0xc2,
            0x0e, 0x0c,
        ];
        let actual = openssl_kdf_ctr(
            MASTER_KEYS[0].as_ref(),
            &[0x55, 0x31], // "U1"; the KDF appends the 0x00 separator
            &[0x1d, 0xc0, 0x00, 0x00, 0x74, 0xe5, 0xc0, 0xa8, 0x2a, 0x01],
            44,
            8,
            16,
        );
        assert_eq!(actual, expected);
    }
}
