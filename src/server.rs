use core::time;
use std::{collections::HashMap, io::{Read, Result}, net::{SocketAddr, UdpSocket}, os::unix::fs, process::{Child, Command, Stdio}, sync::mpsc::Sender, thread::{self, spawn, JoinHandle}};
use std::sync::mpsc;

pub fn create() -> Result<(String, Sender<String>, Sender<String>, Sender<String>)> {
    let (yggtx, yggrx) = mpsc::channel();
    let yggdrasil = thread::spawn(move || {
        let mut ygg = Command::new("sh")
        .arg("-c")
        .arg("sudo yggdrasil -autoconf -logto yggdrasil.log")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("there is a problem with yggdrasil");

        let _ = yggrx.recv().unwrap();// wait for exit signal
        ygg.kill().unwrap();
        //return ygg;
    });

    let mut connectaddr = String::new();

    loop {
        let mut sawmtu = false;
        match std::fs::File::open("yggdrasil.log") {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf).unwrap();
                for line in buf.lines() {
                    if line.contains("Your IPv6 subnet is") {
                        if let Some(addr) = line.split("is ").nth(1) {
                            connectaddr = addr.to_string().replace("::/64", "::1313/64");
                        }
                    }
                    else if line.contains("Interface MTU") {
                        sawmtu = true;
                        break;
                    }
                }
            }
            Err(e) => {
                
            }
        }

        if sawmtu {
            break;
        }
    }

    //panic!("connectaddr is: {}", connectaddr);

    let (ipaddrtx , ipaddrrx) = mpsc::channel();
    let connectaddr_clone = connectaddr.clone();
    thread::spawn(move || {
        let ipaddrchange = Command::new("sh")
        .arg("-c")
        .arg(format!("sudo ip -6 addr add {} dev lo", connectaddr_clone))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("there is a problem with yggdrasil")
        .wait()
        .expect("there is a problem with yggdrasil(wait)");

        let _ = ipaddrrx.recv().unwrap();

        let ipaddrdel = Command::new("sh")
        .arg("-c")
        .arg(format!("sudo ip -6 addr del {} dev lo", connectaddr_clone))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("there is a problem with deleting ip address")
        .wait()
        .expect("there is a problem with deleting ip address(wait)");
    });
    
    thread::sleep(time::Duration::from_millis(2000));

    connectaddr = connectaddr.replace("/64", ":9595");
    let connectaddr_clone = connectaddr.clone();
    let (servertx, serverrx) = std::sync::mpsc::channel();
    let udpserver = thread::spawn(move || {
        run(connectaddr_clone, serverrx);
    });
    

    Ok((connectaddr, yggtx, servertx, ipaddrtx))
}


struct User {
    pub name: String,
    pub addr: SocketAddr
}

fn run(connect_addr: String, serverrx: std::sync::mpsc::Receiver<String>) {
    let socket = match UdpSocket::bind(connect_addr) {
        Ok(s) => s,
        Err(e) => panic!("Failed to bind to socket: {}", e),
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