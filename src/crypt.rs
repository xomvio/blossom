//use aes_gcm::{aead::{Aead, AeadCore, Key, KeyInit, OsRng}, Aes256Gcm, Nonce};
use rand::{rngs::OsRng, TryRngCore};

use crate::{config::Config, error::Result};

/// Converts a room key string to a fixed 32-byte array for cryptographic use
/// 
/// If the input is longer than 32 bytes, it will be truncated.
/// If shorter, it will be padded with the value 103 (ASCII 'g').
/// 
/// # Arguments
/// * `roomkey` - The room key string to convert
/// 
/// # Returns
/// A 32-byte array suitable for cryptographic operations
pub fn convert_to_32_bytes(roomkey: &str) -> [u8; 32] {
    const PADDING_BYTE: u8 = 103; // ASCII 'g'
    let roomkey_bytes = roomkey.as_bytes();
    let mut result = [PADDING_BYTE; 32];
    
    let copy_len = roomkey_bytes.len().min(32);
    result[..copy_len].copy_from_slice(&roomkey_bytes[..copy_len]);
    
    result
}

/// Strips trailing padding bytes (ASCII 'g' = 103) from a 32-byte array
/// 
/// This is the inverse of convert_to_32_bytes, used to reconstruct strings
/// that were padded for cryptographic operations.
/// 
/// # Arguments
/// * `bytes` - A 32-byte array potentially containing trailing 'g' padding
/// 
/// # Returns
/// The original string with trailing padding removed
pub fn strip_padding(bytes: &[u8]) -> String {
    let trimmed_len = bytes.iter().rposition(|&b| b != 103)
        .map_or(bytes.len(), |pos| pos + 1);
    
    String::from_utf8_lossy(&bytes[..trimmed_len]).to_string()
}

/// Generates a cryptographically secure random alphanumeric string
/// 
/// The generated string contains characters from [0-9A-Za-z].
/// 
/// # Arguments
/// * `length` - The desired length of the random string
/// 
/// # Returns
/// A random alphanumeric string of the specified length
/// 
/// # Errors
/// Returns an error if the random number generator fails
pub fn generate_random_string(length: usize) -> Result<String> {
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    
    let mut rng = OsRng;
    let mut bytes = vec![0u8; length];
    rng.try_fill_bytes(&mut bytes)?;

    let random_string: String = bytes
        .into_iter()
        .map(|b| CHARSET[(b as usize) % CHARSET.len()] as char)
        .collect();

    Ok(random_string)
}

/// Generates a random username with the default length
/// 
/// # Returns
/// A random alphanumeric username
/// 
/// # Errors
/// Returns an error if the random number generator fails
pub fn generate_random_username() -> Result<String> {
    generate_random_string(Config::DEFAULT_USERNAME_LENGTH)
}
