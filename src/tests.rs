#[cfg(test)]
mod encryption {
    use crate::crypt::{decrypt, encrypt};
    use aes_gcm::{Key, KeyInit, Aes256Gcm};

    #[test]
    fn encrypt_valid_cipher_and_message() {
        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
        let cipher = Aes256Gcm::new(key);
        let message = "Hello, World!".to_string();
        let encrypted = encrypt(&cipher, message.clone());
        assert!(encrypted.len() == message.len() + 28);
    }

    #[test]
    fn encrypt_empty_message() {
        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
        let cipher = Aes256Gcm::new(key);
        let message = "".to_string();
        let encrypted = encrypt(&cipher, message);
        assert!(encrypted.len() == 28);
    }
    
    #[test]
    #[should_panic]
    fn encrypt_malformed_cipher() {
        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 31]); // invalid key size
        let cipher = Aes256Gcm::new(key);
        let message = "Hello, World!".to_string();
        encrypt(&cipher, message);
    }


    #[test]
    fn decrypt_valid_cipher_and_data() {
        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
        let cipher = Aes256Gcm::new(key);
        let message = "Hello, World!".to_string();
        let encrypted = encrypt(&cipher, message.clone());
        let decrypted = decrypt(&cipher, &encrypted).unwrap();
        assert_eq!(decrypted, message);
    }

    /// This is fails and had to be fixed
    /// decrypt function has problems
    #[test]
    fn decrypt_invalid_cipher() {
        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 31]); // invalid key size
        let cipher = Aes256Gcm::new(key);
        let message = "Hello, World!".to_string();
        let encrypted = encrypt(&cipher, message.clone());
        assert!(decrypt(&cipher, &encrypted).is_err());
    }

    #[test]
    fn decrypt_too_short_data() {
        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
        let cipher = Aes256Gcm::new(key);
        let encrypted: [u8; 10] = [0u8; 10];
        assert_eq!(decrypt(&cipher, &encrypted).unwrap_err().to_string(), "Encrypted data too short");
    }
}