use base64::{prelude::BASE64_STANDARD, Engine};
//use ring::rand::{SecureRandom, SystemRandom};
use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng, Key}, Aes256Gcm, Nonce};
use rand::RngCore;
use ring::rand::{SecureRandom, SystemRandom};

pub fn generate_aesgcm(roomkey: String) -> Aes256Gcm {
    let decoded = BASE64_STANDARD.decode(roomkey).unwrap();
    let key = Key::<Aes256Gcm>::from_slice(&decoded[32..64]);
    Aes256Gcm::new(&key)
}

pub fn generate_roomkey() -> [u8; 64] {
    let mut buf = [0u8; 64];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut buf);
    buf
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

    /// Encrypt a message with the given cipher.
    ///
    /// The message is encrypted with the given cipher using a randomly generated nonce.
    /// The nonce is prepended to the ciphertext, so the returned vector is 12 bytes longer
    /// than the input message.
    /// 
    /// If encryption fails, this function will panic.
pub fn encrypt(cipher: &Aes256Gcm, message: String) -> Vec<u8> {
    // Generate a random nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    // Encrypt the message
    let ciphertext = cipher
        .encrypt(&nonce, message.as_bytes())
        .expect("encryption failure");
    
    // Combine nonce and ciphertext into a single vector
    let mut encrypted = Vec::new();
    encrypted.extend_from_slice(&nonce);
    encrypted.extend_from_slice(&ciphertext);
    
    encrypted
}

    /// Decrypt a message with the given cipher.
    ///
    /// The message is decrypted with the given cipher. The first 12 bytes of the
    /// `encrypted_data` should be the nonce, and the remaining bytes should be the
    /// ciphertext.
    ///
    /// If decryption fails, this function will cry and panic.
    ///
pub fn decrypt(cipher: &Aes256Gcm, encrypted_data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    // The first 12 bytes should be the nonce
    if encrypted_data.len() < 12 {
        return Err("Encrypted data too short".into());
    }
    
    // Split the data into nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    // Decrypt the message
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption error: {}", e))?;
    
    // Convert the decrypted bytes to a string
    String::from_utf8(plaintext)
        .map_err(|e| e.into())
}
