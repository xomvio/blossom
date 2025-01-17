use core::panic;
use std::{fs, io::{Error, Read}, process::{Child, Command, Stdio}, thread, time::{Duration, Instant}};

pub fn start() -> Child {

    match genconf() {
        Ok(_) => {},
        Err(e) => panic!("failed to generate yggdrasil.conf: {}", e)
    }

    add_peers();

    match useconf() {
        Ok(child) => {
            child
        },
        Err(e) => {
            panic!("failed to start yggdrasil: {}", e);
        }
    }
}

/// edits yggdrasil.conf file to add peer address (adds only armenian public peer because its close to me lol)
pub fn add_peers() {
    let mut file = fs::File::open("yggdrasil.conf").unwrap();
    
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    //its not had to be quic, tcp is also ok
    content = content.replace("Peers: []", r#"Peers: [
        quic://37.186.113.100:1515
    ]"#);

    fs::write("yggdrasil.conf", content).unwrap();
}

/// Starts yggdrasil with the configuration file created by `genconf` and
/// logs output to yggdrasil.log.
fn useconf() -> Result<Child, Error> {
    Command::new("sh")
    .arg("-c")
    .arg("sudo yggdrasil -useconffile yggdrasil.conf -logto yggdrasil.log")
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .spawn() // spawn doesn't block
}

    /// Generates a configuration file for yggdrasil at yggdrasil.conf.
    ///
    /// If the file already exists, it will be overwritten.
    ///
    /// This function runs the command `sudo yggdrasil -genconf > yggdrasil.conf`, and
    /// returns an error if the command fails.
fn genconf() -> Result<(), Error> {
    match Command::new("sh")
        .arg("-c")
        .arg("sudo yggdrasil -genconf > yggdrasil.conf")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output() {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
    Ok(())
}

pub fn delconf() {
    match Command::new("sh")
        .arg("-c")
        .arg("sudo rm yggdrasil.conf")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output(){
            Ok(_) => {},
            Err(e) => panic!("failed to delete yggdrasil.conf: {}", e)
        }
}

pub fn wait_for_start() {
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    loop {
        thread::sleep(Duration::from_millis(200));
        if start.elapsed() > timeout {
            panic!("timed out waiting for yggdrasil to start");
        }
        let mut file = match fs::File::open("yggdrasil.log") {
            Ok(file) => file,
            Err(_) => continue  // this file is not created yet. try again
        };
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        if buf.contains("Interface MTU") {
            return;
        }
    }
}

    /// Waits for yggdrasil to start and returns its IPv6 address. This
    /// function will block until yggdrasil is running and ready to go.
    /// If yggdrasil doesn't start within 5 seconds, the function will
    /// return an error.
    /// 
    /// This function will block until yggdrasil is running and ready to go. 
    /// The "Your IPv6 subnet is" line in the log file idicates that our ipv6 
    /// address is ready. However, this doesn't mean that yggdrasil is started.
    /// The "Interface MTU" line indicates that yggdrasil is started.
    /// 
    /// Returns the IPv6 address of yggdrasil for creating a server.
pub fn get_ipv6() -> Result<String, Error> {

    // Wait for yggdrasil to start    
    wait_for_start();

    match std::fs::File::open("yggdrasil.log") {
        Ok(mut file) => {

            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();

            for line in buf.lines() {
                if line.contains("Your IPv6 subnet is") {
                    // If the line starts with "Your IPv6 subnet is",
                    // then the second part of the line is the IPv6
                    // subnet that we're interested in.
                    if let Some(addr) = line.split("is ").nth(1) {
                        return Ok(addr
                            .to_string()
                            .replace("::/64", "::1313/64"));
                    }
                }
            }

            return Err(Error::new(std::io::ErrorKind::NotFound, "yggdrasil.log looks strange..."));
        }
        Err(_) => {
            // this error is unexpected. Because we just checked if the file exists in wait_for_start function
            return Err(Error::new(std::io::ErrorKind::NotFound, "yggdrasil.log not found or not readable"));
        }
    }
}


    /// Adds the given IPv6 address to the loopback interface using the
    /// `ip -6 addr add` command. This ipv6 address is used by yggdrasil
    /// to listen for incoming connections.
    ///
    /// This function will block until the command has finished, and will
    /// return an error if the command fails.
pub fn add_addr(addr: String) -> Result<(), Error> {
    match Command::new("sh")
        .arg("-c")
        .arg(format!("sudo ip -6 addr add {} dev lo", addr))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output() {
            Ok(_) => Ok(()),
            Err(e) => return Err(e)
        }
}

    /// Removes the given IPv6 address from the loopback 
    /// interface using the `ip -6 addr del` command.
pub fn del_addr(addr: String) -> Result<(), Error> {
    let addr = addr.replace(":9595", "");
    match Command::new("sh")
        .arg("-c")
        .arg(format!("sudo ip -6 addr del {} dev lo", addr))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output() {
            Ok(_) => return Ok(()),
            Err(e) => return Err(e)
        }
}

    /// Deletes the yggdrasil.log file. Usage of function is important 
    /// because keeping the log file will cause us to read older logs
    /// thus we will get the old ipv6 address when we create a server.
pub fn del_log() {
    match Command::new("sh")
        .arg("-c")
        .arg("sudo rm yggdrasil.log")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output(){
            Ok(_) => {},
            Err(e) => panic!("failed to delete yggdrasil.log: {}", e)
        }
}