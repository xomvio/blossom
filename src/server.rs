use core::time;
use std::{collections::HashMap, io::Result, net::{SocketAddr, UdpSocket}, process::Child, sync::mpsc::{Receiver, Sender}, thread};
use std::sync::mpsc;

use crate::yggdrasil;

pub fn create() -> Result<(String, Child, Sender<()>)> {

    // start yggdrasil and wait for exit signal via yggtx
    //let yggtx = yggdrasil::start(); 
    let ygg = yggdrasil::start();

    // get yggdrasil ipv6 address
    let mut connectaddr = yggdrasil::get_ipv6(); 

    // add yggdrasil address to loopback
    yggdrasil::add_addr(connectaddr.clone());
    
    // include port to ipv6
    connectaddr = connectaddr.replace("/64", ":9595");
    

    let connectaddr_clone = connectaddr.clone();
    let (servertx, serverrx) = mpsc::channel();
    // start server
    thread::spawn(move || { run(connectaddr_clone, serverrx); });
    
    // initialize graceful thread killer
    //let killer = Killer { yggtx: Some(yggtx), servertx: Some(servertx), ipaddrtx: Some(ipaddrtx) };

    Ok((connectaddr, ygg, servertx))
}


struct User {
    pub name: String,
    pub addr: SocketAddr
}

fn run(connect_addr: String, serverrx: Receiver<()>) {
    //panic!("Yggdrasil IPv6 address: {}", connect_addr);
    thread::sleep(time::Duration::from_millis(2000));
    let socket = match UdpSocket::bind(connect_addr.clone()) {
        Ok(s) => s,
        Err(e) => panic!("Failed to bind to socket: {}\n{}", e, connect_addr),
    };

    let mut rooms: HashMap<String, Vec<User>> = HashMap::new();
    loop {
        let mut buffer = [0; 1024];
        match socket.recv_from(&mut buffer) {
            Ok((size, addr)) => {
                if serverrx.try_recv().is_ok() { // exit signal check
                    break;
                }
                //println!("Request: {}", String::from_utf8_lossy(&buffer));
                let room: String = String::from_utf8_lossy(&buffer[..32]).to_string();
                match rooms.get_mut(&room) {// is there a room with this id
                    Some(users) => {// there is
                        // check if the user is already in the room
                        if users.iter().find(|user| user.addr == addr).is_none() {// user is not in the room yet
                            // add him to the room
                            users.push(User { name: String::from_utf8_lossy(&buffer[32..size]).to_string(), addr });
                            // send all usernames in the room to the new user
                            for user in users.iter() {
                                if user.addr == addr {// don't send the new user his own name
                                    continue;
                                }
                                // send message
                                socket.send_to(user.name.as_bytes(), addr).unwrap();
                            }
                        }
                        
                        // send message to all users in the room
                        for user in users.iter() { // for each user
                            // send message
                            socket.send_to(&buffer[32..size], user.addr).unwrap();
                        }
                    },
                    None => {// there isn't
                        // create a new room
                        rooms.insert(room.clone(), vec![User { name: String::from_utf8_lossy(&buffer[32..size]).to_string(), addr }]);
                        // only send his own name to room creator because he's the only one in the room. lonely.
                        socket.send_to(&buffer[32..size], addr).unwrap();
                    }
                }
            },
            Err(e) => panic!("Failed to read from connection: {}", e),
        }
    }
}