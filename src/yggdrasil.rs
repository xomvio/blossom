use std::{io::Read, process::{Command, Stdio}, sync::mpsc::{self, Sender}, thread};


pub fn handle() -> Sender<bool> {
    let (yggtx, yggrx) = mpsc::channel();
    thread::spawn(move || {
        let mut ygg = Command::new("sh")
        .arg("-c")
        .arg("sudo yggdrasil -autoconf -logto yggdrasil.log")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("there is a problem with yggdrasil");

        let _ = yggrx.recv().unwrap(); // wait for exit signal
        ygg.kill().unwrap();

        Command::new("sh")
        .arg("-c")
        .arg("sudo rm yggdrasil.log")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("there is a problem with yggdrasil");
    });

    yggtx
}

pub fn get_ipv6() -> String {
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
                            connectaddr = addr.to_string().replace("::/64", "::1313/64"); // 1313 is dinle's endpoint. port will be 9595 later
                        }
                    }
                    else if line.contains("Interface MTU") {//last line when yggdrasil is just started
                        sawmtu = true;
                        break;
                    }
                }
            }
            Err(_) => {}
        }

        if sawmtu {
            break;
        }
    }    
    connectaddr
}

pub fn open_port(addr: String) -> Sender<bool> { 
    let (ipaddrtx , ipaddrrx) = mpsc::channel();
    thread::spawn(move || {
        Command::new("sh")
        .arg("-c")
        .arg(format!("sudo ip -6 addr add {} dev lo", addr))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("there is a problem with yggdrasil")
        .wait()
        .expect("there is a problem with yggdrasil(wait)");

        let _ = ipaddrrx.recv().unwrap();

        Command::new("sh")
        .arg("-c")
        .arg(format!("sudo ip -6 addr del {} dev lo", addr))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("there is a problem with deleting ip address")
        .wait()
        .expect("there is a problem with deleting ip address(wait)");
    });

    ipaddrtx
}