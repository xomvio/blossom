# Blossom

Blossom is a decentralized TUI chat application that utilizes the Yggdrasil network to establish connections between users. Blossom allows users to create or join chat rooms, and communicate with each other in real-time.

![image](https://github.com/user-attachments/assets/29b8ce7d-9fe8-433c-92dd-1d662c9b84cd)

## Warning
Blossom aims to be secure but it is very young and has vulnerabilities right now.

## Features
- Decentralized chat architecture using Yggdrasil network
- Create or join chat rooms with unique room keys
- Real-time communication between users in the same room
- End-to-end encryption using AES-256-GCM (plus Yggdrasil's encryption)

## Requirements
- Yggdrasil installed on the system.


## How is it Works
### Running the Blossom
Blossom must be run as sudo. This is because Blossom will handle Yggdrasil and IPv6 address every time you connect to a room.
This means every time you created or joined a room, you do it with a disposable IPv6 address in yggdrasil network.

### User Creates or Joins a Chat Room
Running Blossom without CLI arguments means creating and hosting a new chat session with a random username. You can specify the username using the --username argument. --roomkey argument is for joining an existing chat session. Roomkey is simply a base64 encoded version of the host's temporary IPv6 address.

### Generating a Unique Node ID
When the Blossom is ran, it generates a unique node ID using the Yggdrasil network's cryptographic algorithms. This node ID is used to find the host's machine on the network.

### Sending a message
When a user sends a message, Blossom encrypts the message using the AES-256-GCM encryption algorithm and sends it to the host's local server via Yggdrasil.

### Receiving Message at Host
Every user in the room sends their messages into host's local server. When the host's machine receives the message, it sends message to everyone in the chat room including self. Well... This may sounds like we are centralizing the host a little bit. But we can talk about it.

### Closing the Blossom
Using Ctrl + C is important to gracefully shutdown the application. This is:
- Stops Yggdrasil connection
- Deletes disposable Yggdrasil config
- Deletes IPv6 address from loopback. (if you are host)

## Usage

```bash
blossom [--username <username>] [--roomkey <roomkey>]
```

## Contributing
Contributions are welcome! If you'd like to contribute to the project, please fork the repository and submit a pull request with your changes.