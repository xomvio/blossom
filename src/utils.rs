use std::borrow::Cow;

//use ring::{self, aead, rand::{SecureRandom, SystemRandom}};
//use magic_crypt::{new_magic_crypt, MagicCryptTrait};

//use magic_crypt::generic_array::GenericArray;
use ring::{rand::{SecureRandom, SystemRandom}};
use aes_gcm::{
    aead::{Aead, AeadCore, AeadMut, KeyInit, OsRng}, aes::cipher, Aes256Gcm, Key, Nonce // Or `Aes128Gcm`
};
#[test]
pub fn test() {
    let roomkey = generate_rnd_str(32);
    let keyref = roomkey.as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(keyref);
    
    
    // Alternatively, the key can be transformed directly from a byte slice
    // (panicks on length mismatch):
    let key = Key::<Aes256Gcm>::from_slice(&key);
    
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref()).unwrap();
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
    assert_eq!(&plaintext, b"plaintext message");
}

pub fn generate_aesgcm(roomkey: String) -> Aes256Gcm {
    let key = Key::<Aes256Gcm>::from_slice(roomkey.as_bytes());
    let cipher = Aes256Gcm::new(&key);
    cipher
}


pub fn generate_rnd_str(length: usize) -> String {
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; length];
    rng.fill(&mut bytes).unwrap(); 

    // process bytes to get alphanumeric characters
    let random_string: String = bytes
        .into_iter()
        .map(|b| {
            let index = (b % 62) as u8; // 0-61 arası bir sayı üret
            match index {
                0..=9 => (b'0' + index) as char,       // '0' - '9'
                10..=35 => (b'A' + index - 10) as char, // 'A' - 'Z'
                _ => (b'a' + index - 36) as char,      // 'a' - 'z'
            }
        })
        .collect();

    random_string
}
