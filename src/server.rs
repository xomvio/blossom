use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    process::Child,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};
use std::sync::mpsc;

use crate::{config, error::{BlossomError, Result}, yggdrasil};

/// Creates a new server instance with Yggdrasil networking
/// 
/// # Arguments
/// * `port` - The port to bind the server to
/// 
/// # Returns
/// A tuple containing (connection_address, yggdrasil_process, shutdown_sender)
/// 
/// # Errors
/// Returns an error if Yggdrasil fails to start or network setup fails
pub fn create(port: &str) -> Result<(String, Child, Sender<()>)> {
    // Start yggdrasil process
    let ygg_process = yggdrasil::start(port)?;

    // Get yggdrasil IPv6 address
    let mut connect_addr = yggdrasil::get_ipv6()?;

    // Add yggdrasil address to loopback interface
    yggdrasil::add_addr(connect_addr.clone())?;

    // Replace subnet notation with server port
    connect_addr = connect_addr.replace("/64", &format!(":{}", config::SERVER_PORT));

    let connect_addr_clone = connect_addr.clone();
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    
    // Start server in background thread
    thread::spawn(move || {
        if let Err(e) = run_server(connect_addr_clone, shutdown_rx) {
            eprintln!("Server error: {}", e);
        }
    });
    
    Ok((connect_addr, ygg_process, shutdown_tx))
}

/// Represents a connected user in the chat room
#[derive(Debug, Clone)]
struct User {
    name: String,
    addr: SocketAddr,
}

impl User {
    fn new(name: String, addr: SocketAddr) -> Self {
        Self { name, addr }
    }
}

/// Runs the main server loop handling client connections and message routing
/// 
/// # Arguments
/// * `connect_addr` - The address to bind the server socket to
/// * `shutdown_rx` - Channel receiver for shutdown signals
/// 
/// # Errors
/// Returns an error if socket operations fail
fn run_server(connect_addr: String, shutdown_rx: Receiver<()>) -> Result<()> {
    let socket = UdpSocket::bind(&connect_addr)
        .map_err(|e| BlossomError::Network(format!("Failed to bind to {}: {}", connect_addr, e)))?;
    
    // Set socket to non-blocking mode for better responsiveness
    socket.set_nonblocking(true)
        .map_err(|e| BlossomError::Network(format!("Failed to set socket non-blocking: {}", e)))?;

    let mut users: HashMap<SocketAddr, User> = HashMap::new();
    let mut buffer = [0u8; config::MAX_BUFFER_SIZE];
    
    loop {
        // Check for shutdown signal
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
        
        match socket.recv_from(&mut buffer) {
            Ok((size, addr)) => {
                handle_client_message(&socket, &mut users, &buffer[..size], addr)?;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available, sleep briefly to avoid busy waiting
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => {
                return Err(BlossomError::Network(format!("Failed to receive data: {}", e)));
            }
        }
    }
    
    Ok(())
}

/// Handles a message from a client
/// 
/// # Arguments
/// * `socket` - The UDP socket for sending responses
/// * `users` - HashMap of connected users
/// * `data` - The received message data
/// * `addr` - The sender's address
/// 
/// # Errors
/// Returns an error if network operations fail
fn handle_client_message(
    socket: &UdpSocket,
    users: &mut HashMap<SocketAddr, User>,
    data: &[u8],
    addr: SocketAddr,
) -> Result<()> {
    let message = String::from_utf8_lossy(data);
    
    // Check if this is a new user (first message is typically just the username)
    if !users.contains_key(&addr) {
        let username = message.trim().to_string();

        // Send existing users' names to the new user as join notifications
        for existing_user in users.values() {
            let join_msg = format!("J{}", existing_user.name);
            send_message(socket, join_msg.as_bytes(), addr)?;
        }

        users.insert(addr, User::new(username.clone(), addr));

        // Broadcast the new user's join to all clients (including themselves)
        let join_broadcast = format!("J{}", username);
        broadcast_message(socket, users, join_broadcast.as_bytes())?;
    } else {
        // Chat message — forward as-is (already has M prefix from client)
        broadcast_message(socket, users, data)?;
    }
    
    Ok(())
}

/// Broadcasts a message to all connected users
/// 
/// # Arguments
/// * `socket` - The UDP socket for sending messages
/// * `users` - HashMap of connected users
/// * `data` - The message data to broadcast
/// 
/// # Errors
/// Returns an error if any send operation fails
fn broadcast_message(
    socket: &UdpSocket,
    users: &HashMap<SocketAddr, User>,
    data: &[u8],
) -> Result<()> {
    for user in users.values() {
        send_message(socket, data, user.addr)?;
    }
    Ok(())
}

/// Sends a message to a specific address
/// 
/// # Arguments
/// * `socket` - The UDP socket for sending
/// * `data` - The message data to send
/// * `addr` - The destination address
/// 
/// # Errors
/// Returns an error if the send operation fails
fn send_message(socket: &UdpSocket, data: &[u8], addr: SocketAddr) -> Result<()> {
    socket.send_to(data, addr)
        .map_err(|e| BlossomError::Network(format!("Failed to send message to {}: {}", addr, e)))?;
    Ok(())
}