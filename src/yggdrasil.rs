use std::{io::Read, process::{Child, Command, Stdio}};

pub fn start() -> Child {

    genconf();

    let yggdrasil = useconf(); // spawn doesn't block

    yggdrasil
}

fn useconf() -> Child {
    Command::new("sh")
    .arg("-c")
    .arg("sudo yggdrasil -useconffile yggdrasil.conf -logto yggdrasil.log")
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .spawn() // spawn doesn't block
    .expect("there is a problem with yggdrasil")
}

fn genconf(){
    Command::new("sh")
    .arg("-c")
    .arg("sudo yggdrasil -genconf > yggdrasil.conf")
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .output()
    .expect("there is a problem with yggdrasil");
}

pub fn delconf() {
    Command::new("sh")
    .arg("-c")
    .arg("sudo rm yggdrasil.conf")
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .output()
    .expect("there is a problem with yggdrasil");
}

pub fn get_ipv6() -> String {
    //getsubnet().replace("::/64", "::1313/64") // 1313 is dinle's endpoint. port will be 9595 later
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

pub fn add_addr(addr: String) {
    Command::new("sh")
    .arg("-c")
    .arg(format!("sudo ip -6 addr add {} dev lo", addr))
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .output()
    .expect("there is a problem with yggdrasil");
}

pub fn del_addr(addr: String) {
    let addr = addr.replace(":9595", "");
    Command::new("sh")
    .arg("-c")
    .arg(format!("sudo ip -6 addr del {} dev lo", addr))
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .output()
    .expect("there is a problem with yggdrasil");    
}
