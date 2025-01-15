use core::time;
use std::{borrow::Cow, io::{stdin, stdout, BufReader, Read, Write}, net::{TcpListener, UdpSocket}, process::{Command, Stdio}, thread::{self, spawn}};

use base64::{prelude::BASE64_STANDARD, Engine};
use ring::{rand::{SecureRandom, SystemRandom}};
use aes_gcm::{
    aead::{Aead, AeadCore, AeadMut, KeyInit, OsRng, Key}, aes::cipher, Aes256Gcm, Nonce // Or `Aes128Gcm`
};
use rand::{Rng, RngCore};

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


#[test]
fn test_ygg_conn() {
    use std::io::{Read, Write};

    //let mut socket = TcpStream::connect("200:da14:11e:8701:d17f:eb9c:581d:14ae:9595").unwrap();    
    let mut socket = UdpSocket::bind("[::]:0").unwrap();
    socket.connect("303:c8b5:db69:fc6d::3131:9595").unwrap();

    let mut buffer = [0; 1024];

    socket.send("hello from client".as_bytes()).unwrap();
    socket.recv(&mut buffer).unwrap();
    println!("{}", String::from_utf8_lossy(&buffer));
}

#[test]
fn test_yggdrasil() {
    let x = Command::new("sh")
        .arg("-c")
        .arg("echo 'Hel lo!!!'")
        .output()
        .expect("failed to execute process");

    println!("{}", String::from_utf8_lossy(&x.stdout));

    let mut ygg = Command::new("sudo")
        .arg("yggdrasil")
        .arg("-autoconf")
        .arg("-logto")
        .arg("yggdrasil.log")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("there is a problem with yggdrasil");
    
    println!("Yggdrasil started");
    let mut connectaddr = String::new();

    loop {
        println!("read yggdrasil.log");
        thread::sleep(time::Duration::from_millis(500));
        let mut log = Command::new("sh")
        .arg("-c")
        .arg("cat yggdrasil.log")
        .output()
        .expect("failed to execute process");

        for line in String::from_utf8_lossy(&log.stdout).lines() {
            println!("{}", line);
            if line.contains("Your IPv6 subnet is") {
                let mut split = line.split("is ");
                split.next();
                connectaddr = split.next().unwrap().to_string().replace("::/64", "::1313:9595"); //301:1453:fe64:76b5::/64
                break;
            }
        }

        if !connectaddr.is_empty() {
            break;
        }
    }
    println!("connectaddr is: {}", connectaddr);

    let mut socket = UdpSocket::bind("[::]:0").unwrap();
    socket.connect(connectaddr).unwrap();

    let mut buffer = [0; 1024];

    socket.send("hello from client".as_bytes()).unwrap();
    socket.recv(&mut buffer).unwrap();
    println!("{}", String::from_utf8_lossy(&buffer));


}//.arg("yggdrasil -p 303:c8b5:db69:fc6d::3131:9595")