use aes_gcm::{aead::{Aead, AeadCore, Key, KeyInit, OsRng}, Aes256Gcm, Nonce};
use rand::RngCore;

pub fn generate_aesgcm(roomkeybtes: [u8; 32]) -> Aes256Gcm {
    let key = Key::<Aes256Gcm>::from_slice(&roomkeybtes);
    Aes256Gcm::new(&key)
}

pub fn turn_to_32_bytes(roomkey: String) -> [u8; 32] {
    //let mut base: [u8; 32] = OsRng.gen();
    let mut base = [103u8; 32];
    if roomkey.bytes().len() > 32 {
        for i in 0..32 {
            base[i] = roomkey.as_bytes()[i];
        }
    }
    else {
        for i in 0..roomkey.len() {
            base[i] = roomkey.as_bytes()[i];
        }
    }
    base
}

pub fn generate_rnd_str(length: usize) -> String {
    let mut rng = OsRng::default();
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);

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
