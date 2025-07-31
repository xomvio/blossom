# Blossom

Blossom is a decentralized TUI chat application that utilizes the Yggdrasil network to establish connections between users. Every connection is made with an automated single-use configuration. Blossom allows users to  create or join chat rooms, and communicate with each other in real-time.

![image](https://github.com/user-attachments/assets/29b8ce7d-9fe8-433c-92dd-1d662c9b84cd)

## Features
- Decentralized chat architecture using Yggdrasil network
- Create or join chat rooms with unique room keys
- Real-time communication between users in the same room
- End-to-end encryption from Yggdrasil

## Requirements
- A computer with Linux (or Unix-like operating system which spesifically compatible with blossom).
- Yggdrasil already installed on the system.
- IPv6 support is enabled. (IPv6 connection is not needed)

## How Does it Work
### Running the Blossom
Blossom must be run as sudo. This is because Blossom will handle Yggdrasil and IPv6 address every time you connect to a room.
This means every time you created or joined a room, you do it with a disposable IPv6 address in yggdrasil network.

### User Creates or Joins a Chat Room
Running Blossom without CLI arguments means creating and hosting a new chat session with a random username. If you want to join an existing room, you should use --roomkey argument which your host will give you. Roomkey is simply a base64 encoded version of the host's temporary IPv6 address.

### Generating a Unique Node ID
When the Blossom is ran, it generates a unique node ID using the Yggdrasil network's cryptographic algorithms. This node ID is used to find the host's machine on the network.

### Sending and Receiving Messages
Each user in the room sends their messages to the host's machine. When the host's machine receives the message, it sends the message to everyone in the chat room, including itself. That is, not everyone is both a server and a client at the same time. Which means only host will reveal its temporary address to everyone in the room. The addresses of the room participants are only revealed to the host.

## Usage
```bash
sudo blossom [--roomkey <roomkey>] [--username <username>] [--port <port>]
```
### Closing the Blossom
Using Ctrl + C is important to gracefully shutdown the application. This is:
- Stops Yggdrasil connection
- Deletes disposable Yggdrasil config
- Deletes IPv6 address from loopback. (if you are host)

If you somehow close Blossom ungracefully, That's okay. Just start Blossom and shutdown again with Ctrl+C. Blossom will fix itself.

## Contributing
Contributions are welcome! If you'd like to contribute to the project, please fork the repository and submit a pull request with your changes.
