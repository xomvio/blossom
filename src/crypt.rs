//use aes_gcm::{aead::{Aead, AeadCore, Key, KeyInit, OsRng}, Aes256Gcm, Nonce};
use rand::{rngs::OsRng, TryRngCore};

pub fn convert_to_32_bytes(roomkey: String) -> [u8; 32] {
    let roomkeybytes = roomkey.as_bytes();
    let mut base = [103u8; 32];
    if roomkeybytes.len() > 32 {
        base.copy_from_slice(&roomkeybytes[0..32]);
    }
    else {
        base[0..roomkeybytes.len()].copy_from_slice(roomkeybytes);
    }
    base
}

pub fn generate_rnd_str(length: usize) -> Result<String, rand::rand_core::OsError> {
    let mut rng = OsRng;
    let mut bytes = vec![0u8; length];
    rng.try_fill_bytes(&mut bytes)?;

    // process bytes to get alphanumeric characters
    let random_string: String = bytes
        .into_iter()
        .map(|b| {
            let index = b % 62; // generate random number between 0-61
            match index {
                0..=9 => (b'0' + index) as char,       // '0' - '9'
                10..=35 => (b'A' + index - 10) as char, // 'A' - 'Z'
                _ => (b'a' + index - 36) as char,      // 'a' - 'z'
            }
        })
        .collect();

    Ok(random_string)
}
