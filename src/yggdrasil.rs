use std::{
    fs,
    io::{Error, ErrorKind, Read},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{config::Config, error::{BlossomError, Result}};

/// Starts Yggdrasil with proper configuration and peer setup
/// 
/// # Arguments
/// * `port` - The port to check for conflicts
/// 
/// # Returns
/// A Child process handle for the running Yggdrasil instance
/// 
/// # Errors
/// Returns an error if any setup step fails
pub fn start(port: &str) -> Result<Child> {
    check_prerequisites(port)?;
    generate_conf()?;
    add_peers()?;
    run_yggdrasil()
}

/// Checks prerequisites before starting Yggdrasil
/// 
/// Verifies that:
/// - The specified port is not in use
/// - Yggdrasil is installed on the system
/// - Kills any existing Yggdrasil processes
/// 
/// # Arguments
/// * `port` - The port to check for conflicts
/// 
/// # Errors
/// Returns an error if prerequisites are not met
fn check_prerequisites(port: &str) -> Result<()> {
    // Validate and check if port is already in use
    let port_num: u16 = port.parse()
        .map_err(|e| BlossomError::InvalidData(format!("Invalid port number '{}': {}", port, e)))?;
    
    let port_check = Command::new("lsof")
        .arg(format!("-i:{}", port_num))
        .output()
        .map_err(|e| BlossomError::Command(format!("Failed to check port usage: {}", e)))?;
    
    if port_check.status.success() {
        return Err(BlossomError::Network(format!("Port {} is already in use", port_num)));
    }

    // Check if Yggdrasil is installed
    let ygg_check = Command::new("which")
        .arg("yggdrasil")
        .output()
        .map_err(|e| BlossomError::Command(format!("Failed to check Yggdrasil installation: {}", e)))?;
    
    if !ygg_check.status.success() {
        return Err(BlossomError::Yggdrasil(
            "Yggdrasil is not installed. Please install Yggdrasil to use this application.".to_string()
        ));
    }

    // Check if Yggdrasil is already running and kill it if necessary
    let ygg_running = Command::new("pgrep")
        .arg("yggdrasil")
        .output()
        .map_err(|e| BlossomError::Command(format!("Failed to check running processes: {}", e)))?;
    
    if ygg_running.status.success() {
        let kill_result = Command::new("sudo")
            .args(["killall", "yggdrasil"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .output()
            .map_err(|e| BlossomError::Command(format!("Failed to execute killall: {}", e)))?;

        if !kill_result.status.success() {
            return Err(BlossomError::Yggdrasil(
                "Failed to stop existing Yggdrasil process. Please stop it manually.".to_string()
            ));
        }
        
        // Give the process time to fully terminate
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}

/// Adds peer addresses to the Yggdrasil configuration file
/// 
/// # Errors
/// Returns an error if file operations fail
fn add_peers() -> Result<()> {
    let config_path = Config::YGGDRASIL_CONF_PATH;
    
    let content = fs::read_to_string(config_path)
        .map_err(|e| BlossomError::Config(format!("Failed to read Yggdrasil config: {}", e)))?;
    
    let updated_content = content.replace(
        "Peers: []",
        &format!(r#"Peers: [
        {}
    ]"#, Config::DEFAULT_YGGDRASIL_PEER)
    );

    fs::write(config_path, updated_content)
        .map_err(|e| BlossomError::Config(format!("Failed to write Yggdrasil config: {}", e)))?;

    Ok(())
}

/// Starts Yggdrasil with the generated configuration file
/// 
/// # Returns
/// A Child process handle for the running Yggdrasil instance
/// 
/// # Errors
/// Returns an error if the process fails to start
fn run_yggdrasil() -> Result<Child> {
    let command = format!(
        "sudo yggdrasil -useconffile {} -logto {}",
        Config::YGGDRASIL_CONF_PATH,
        Config::YGGDRASIL_LOG_PATH
    );
    
    Command::new("sh")
        .args(["-c", &command])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .map_err(|e| BlossomError::Yggdrasil(format!("Failed to start Yggdrasil: {}", e)))
}

/// Generates a configuration file for Yggdrasil
/// 
/// If the file already exists, it will be overwritten.
/// 
/// # Errors
/// Returns an error if the command fails
fn generate_conf() -> Result<()> {
    let command = format!("sudo yggdrasil -genconf > {}", Config::YGGDRASIL_CONF_PATH);
    
    let output = Command::new("sh")
        .args(["-c", &command])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output()
        .map_err(|e| BlossomError::Command(format!("Failed to generate Yggdrasil config: {}", e)))?;
    
    if !output.status.success() {
        return Err(BlossomError::Yggdrasil(
            "Failed to generate Yggdrasil configuration".to_string()
        ));
    }
    
    Ok(())
}

/// Deletes the Yggdrasil configuration file
/// 
/// # Errors
/// Returns an error if the file deletion fails
pub fn delconf() -> Result<()> {
    let command = format!("sudo rm {}", Config::YGGDRASIL_CONF_PATH);
    
    let output = Command::new("sh")
        .args(["-c", &command])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output()
        .map_err(|e| BlossomError::Command(format!("Failed to delete config file: {}", e)))?;
    
    if !output.status.success() {
        return Err(BlossomError::Config(
            "Failed to delete Yggdrasil configuration file".to_string()
        ));
    }
    
    Ok(())
}

/// Waits for Yggdrasil to start by monitoring the log file
/// 
/// # Errors
/// Returns an error if Yggdrasil doesn't start within the timeout period
pub fn wait_for_start() -> Result<()> {
    let start = Instant::now();
    let timeout = Config::yggdrasil_startup_timeout();
    let check_interval = Config::yggdrasil_log_check_interval();
    
    loop {
        thread::sleep(check_interval);
        
        if start.elapsed() > timeout {
            return Err(BlossomError::Yggdrasil(
                "Timeout waiting for Yggdrasil to start".to_string()
            ));
        }
        
        match fs::File::open(Config::YGGDRASIL_LOG_PATH) {
            Ok(mut file) => {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() && content.contains("Interface MTU") {
                    return Ok(());
                }
            }
            Err(_) => continue, // Log file not created yet, keep waiting
        }
    }
}

/// Waits for Yggdrasil to start and returns its IPv6 address
/// 
/// This function will block until Yggdrasil is running and ready to go.
/// The "Your IPv6 subnet is" line in the log file indicates that our IPv6 
/// address is ready. The "Interface MTU" line indicates that Yggdrasil is started.
/// 
/// # Returns
/// The IPv6 address of Yggdrasil for creating a server
/// 
/// # Errors
/// Returns an error if Yggdrasil doesn't start or the IPv6 address cannot be found
pub fn get_ipv6() -> Result<String> {
    // Wait for Yggdrasil to start
    wait_for_start()?;

    let content = fs::read_to_string(Config::YGGDRASIL_LOG_PATH)
        .map_err(|e| BlossomError::Yggdrasil(format!("Failed to read Yggdrasil log: {}", e)))?;

    for line in content.lines() {
        if line.contains("Your IPv6 subnet is") {
            if let Some(addr) = line.split("is ").nth(1) {
                return Ok(addr.replace("::/64", "::1313/64"));
            }
        }
    }

    Err(BlossomError::Yggdrasil(
        "Could not find IPv6 address in Yggdrasil log. Please check the log file.".to_string()
    ))
}


/// Adds the given IPv6 address to the loopback interface
/// 
/// This IPv6 address is used by Yggdrasil to listen for incoming connections.
/// 
/// # Arguments
/// * `addr` - The IPv6 address to add to the loopback interface
/// 
/// # Errors
/// Returns an error if the command fails
pub fn add_addr(addr: String) -> Result<()> {
    let command = format!("sudo ip -6 addr add {} dev lo", addr);
    
    let output = Command::new("sh")
        .args(["-c", &command])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output()
        .map_err(|e| BlossomError::Command(format!("Failed to add IPv6 address: {}", e)))?;
    
    if !output.status.success() {
        return Err(BlossomError::Network(
            format!("Failed to add IPv6 address {} to loopback interface", addr)
        ));
    }
    
    Ok(())
}

/// Removes the given IPv6 address from the loopback interface
/// 
/// # Arguments
/// * `addr` - The IPv6 address to remove from the loopback interface
/// 
/// # Errors
/// Returns an error if the command fails
pub fn del_addr(addr: String) -> Result<()> {
    let cleaned_addr = addr.replace(&format!(":{}", Config::SERVER_PORT), "");
    let command = format!("sudo ip -6 addr del {} dev lo", cleaned_addr);
    
    let output = Command::new("sh")
        .args(["-c", &command])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output()
        .map_err(|e| BlossomError::Command(format!("Failed to remove IPv6 address: {}", e)))?;
    
    if !output.status.success() {
        return Err(BlossomError::Network(
            format!("Failed to remove IPv6 address {} from loopback interface", cleaned_addr)
        ));
    }
    
    Ok(())
}

/// Deletes the Yggdrasil log file
/// 
/// This is important because keeping the log file will cause us to read older logs,
/// thus we will get the old IPv6 address when we create a server.
/// 
/// # Errors
/// Returns an error if the file deletion fails
pub fn del_log() -> Result<()> {
    let command = format!("sudo rm {}", Config::YGGDRASIL_LOG_PATH);
    
    let output = Command::new("sh")
        .args(["-c", &command])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .output()
        .map_err(|e| BlossomError::Command(format!("Failed to delete log file: {}", e)))?;
    
    if !output.status.success() {
        return Err(BlossomError::Config(
            "Failed to delete Yggdrasil log file".to_string()
        ));
    }
    
    Ok(())
}