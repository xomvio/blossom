# Blossom
A decentralized, end-to-end encrypted TUI chat application built in Rust.

Blossom runs over the Yggdrasil network which is a self-arranging encrypted mesh network. Each session uses a disposable Yggdrasil node identity. no persistent address, no central server, no metadata trail.

![image](https://github.com/user-attachments/assets/29b8ce7d-9fe8-433c-92dd-1d662c9b84cd)

## Features
- Decentralized chat architecture using Yggdrasil network
- Create or join chat rooms with unique room keys
- Real-time communication between users in the same room
- End-to-end encryption from Yggdrasil

## How It Works
One peer acts as host. Others connect via a room key (a base64-encoded form of the host's temporary Yggdrasil IPv6 address). All traffic is encrypted in transit by Yggdrasil's built-in public-key cryptography.

The host's address is visible to room participants. Participant addresses are visible only to the host. These adresses is not persistent across sessions.

On clean exit (Ctrl+C), blossom tears down the disposable Yggdrasil config and removes the ephemeral IPv6 address from loopback. If the process is killed ungracefully, re-running and then exiting cleanly will restore the system state.

## Requirements
* Linux (or Unix-like operating system which spesifically compatible with blossom)
* Yggdrasil installed and accessible
* IPv6 support is enabled on the system (IPv6 connection not needed)

## Usage
Blossom requires sudo because it manages Yggdrasil configuration and IPv6 addresses at runtime.
```bash
sudo blossom [--roomkey <roomkey>] [--username <username>] [--port <port>]

# Host a new room (random username, random room key printed on start)
sudo blossom

# Join an existing room
sudo blossom --roomkey <base64-key>

# With custom username and port
sudo blossom --roomkey <base64-key> --username alice --port 9000

```
Share the printed room key with anyone you want to invite.

## Contributing
Contributions are welcome! If you'd like to contribute to the project, please fork the repository and submit a pull request with your changes.
