#![allow(unused_imports)]
use std::{net::UdpSocket, process::{Command, Stdio}, thread, time};
use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, Key, KeyInit};
use rand::rngs::OsRng;
use crate::utils::generate_rnd_str;


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


#[test]
fn test_yggdrasil() {
    let x = Command::new("sh")
        .arg("-c")
        .arg("echo 'Hel lo!!!'")
        .output()
        .expect("failed to execute process");

    println!("{}", String::from_utf8_lossy(&x.stdout));

    Command::new("sudo")
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
        let log = Command::new("sh")
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

    let /*mut*/ socket = UdpSocket::bind("[::]:0").unwrap();
    socket.connect(connectaddr).unwrap();

    let mut buffer = [0; 1024];

    socket.send("hello from client".as_bytes()).unwrap();
    socket.recv(&mut buffer).unwrap();
    println!("{}", String::from_utf8_lossy(&buffer));


}