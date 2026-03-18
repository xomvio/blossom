use core::time;
use std::{io, net::UdpSocket, process::Child, sync::mpsc::Sender, thread};
use base64::{prelude::BASE64_STANDARD, Engine};
use ratatui::{buffer::Buffer, crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, layout::Rect, style::Stylize, symbols::border, text::Line, widgets::{Block, Paragraph, Widget}, Frame};

use crate::{crypt, server, yggdrasil, error::{Result, BlossomError}};

pub struct App {
    ui: UI,
    connectaddr: String,
    socket: UdpSocket,
    exit: bool,
    yggdr: Child,
    servershutter: Option<Sender<()>>,
}

struct UI {
    username: String,
    roomkey: String,
    roomusers: Vec<Line<'static>>,
    history: Vec<Line<'static>>,
    input: String,
    showkey: bool,
    showusers: bool
}



impl App {

    pub fn create_room(username: String, port: String) -> Result<Self> {
        let (connectaddr, yggdr, servershutter) = server::create(&port)?;
        let roomkeybytes = crypt::convert_to_32_bytes(&connectaddr);
        let socket = UdpSocket::bind(format!("[::]:{}", port))?;
        
        Ok(Self {
            ui: UI {
                username: username.clone(),
                roomkey: BASE64_STANDARD.encode(roomkeybytes),
                roomusers: vec![],
                history: Vec::new(),
                input: String::new(),
                showkey: true,
                showusers: true
            },
            connectaddr: connectaddr.clone(),
            socket,
            exit: false,
            yggdr,
            servershutter: Some(servershutter)
        })
    }

    pub fn join_room(username: String, roomkey: String, port: String) -> Result<Self> {
        let yggdr = yggdrasil::start(&port)?;
        let _ = yggdrasil::get_ipv6();

        let decodedroomkey = match BASE64_STANDARD.decode(roomkey.clone()) {
            Ok(decoded) => decoded,
            Err(e) => return Err(BlossomError::InvalidData(format!("Base64 decode error: {}", e)))
        };

        // Convert 32-byte array back to string (trim trailing 'g' padding)
        let connectaddr = match String::from_utf8(decodedroomkey.clone()) {
            Ok(decoded_str) => crypt::strip_padding(&decoded_str.as_bytes()),
            Err(e) => return Err(BlossomError::InvalidData(format!("UTF-8 conversion error: {}", e)))
        };
        
        let socket = UdpSocket::bind(format!("[::]:{}", port))?;

        Ok(Self {
            ui: UI {
                username: username.clone(),
                roomkey: roomkey.clone(),
                roomusers: vec![],
                history: Vec::new(),
                input: String::new(),
                showkey: true,
                showusers: true
            },
            connectaddr,
            socket,
            exit: false,
            yggdr,
            servershutter: None
        })
    }


    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        // Attempt to establish a connection to the specified address
        match self.socket.connect(self.connectaddr.clone()) {
            Ok(_) => {}
            Err(e) => {
                return Err(BlossomError::Io(e));
            }
        }
        
        match self.socket.set_nonblocking(true) {
            Ok(_) => {}
            Err(e) => {
                return Err(BlossomError::Io(e));
            }
        }

        let mut buffer = [0; 10240]; // Storing incoming data here. this limit may change

        thread::sleep(time::Duration::from_millis(3000));

        // Send the username as the initial message to the server (null-byte delimited)
        if self.socket.send(self.ui.username.as_bytes()).is_err() {
            eprintln!("Failed to send username");
        }

        // Main loop that runs until the exit flag is set
        while !self.exit {
            // Attempt to receive data from the socket
            match self.socket.recv_from(buffer.as_mut()) {
                Ok((size, _)) => {
                    // Check if this is a user join notification (first message from new client)
                    // In the server implementation, first messages are usernames only
                    // We can distinguish by checking for pipe delimiter which indicates regular chat messages
                    let received_data = match String::from_utf8(buffer[..size].to_vec()) {
                        Ok(data) => data,
                        Err(_) => {
                            eprintln!("Unicode error in received data - dropping");
                            continue;
                        }
                    };

                    // If this message contains a pipe, it's a chat message (username|message)
                    // Otherwise, treat as user join notification or other system messages
                    if received_data.contains('|') {
                        // This is a regular chat message
                        let (username, message) = match received_data.split_once('|') {
                            Some((u, m)) => (u, m),
                            None => {
                                // Missing delimiter. this is unexpected since we already check if data contains '|'
                                // Treat entire buffer as a message from "Unknown"
                                self.ui.history.push(Line::from(format!("[{}] {}", 
                                    "Unknown".cyan(), received_data.gray())));
                                continue;
                            }
                        };

                        // Add the message to the chat history
                        let username = username.cyan();
                        let message = message.gray();
                        self.ui.history.push(Line::from(format!("[{}] {}",username, message)));
                    } else {
                        // User join notification or system message
                        // The server sends the username as first message when a new client connects
                        let username = received_data.trim();
                        if !username.is_empty() {
                            // Add the new user to the room users list and history
                            self.ui.roomusers.push(Line::from(username.to_string()).red());
                            self.ui.history.append(&mut vec![Line::from(vec![username.to_owned().red(), " joined the room".red()])]);
                        }
                    }
                }
                // Handle the case where the socket would block, indicating no data is available
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Can perform other tasks here if needed
                }
                // Handle any other errors that occur during reception
                Err(e) => {
                    eprintln!("Socket error: {}", e);
                    break; // Exit the loop on error
                }
            }

            // Draw the current state of the terminal UI
            if let Err(e) = terminal.draw(|f| self.draw(f)) {
                eprintln!("UI render error: {}", e);
                continue;
            }

            // Handle any user input events
            if let Err(e) = self.handle_events() {
                eprintln!("Event handling error: {}", e);
                continue;
            }
        }
        
        // Perform a graceful shutdown of the application

        // Terminate the yggdrasil process
        if let Err(e) = self.yggdr.kill() {
            eprintln!("Failed to terminate yggdrasil process: {}", e);
        }
        
        if let Some(servershutter) = &self.servershutter {
            // Send a shutdown signal to the server
            if servershutter.send(()).is_err() {
                eprintln!("Failed to send shutdown signal to server");
            }
            
            // Delete the yggdrasil address
            if let Err(e) = yggdrasil::del_addr(self.connectaddr.clone()) {
                eprintln!("Failed to delete Yggdrasil address: {}", e);
            }
        }
        
        // Delete the configuration file
        if let Err(e) = yggdrasil::delconf() {
            eprintln!("Failed to delete config file: {}", e);
        }
        
        // Delete the log file
        if let Err(e) = yggdrasil::del_log() {
            eprintln!("Failed to delete log file: {}", e);
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if poll(time::Duration::from_millis(100))? {
            // It's guaranteed that `read` won't block, because `poll` returned
            // `Ok(true)`.
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                }
                _=> {}
            }
        } else {
            // Timeout expired, no `Event` is captured
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char('c') = key_event.code {
                self.exit();
            }
            // this will be used if we want to handle more keybinds
            /*match key_event.code {
                KeyCode::Char('c') => self.exit(),
                _ => {}
            }*/
            return
        }
        
        match key_event.code {
            KeyCode::F(1) => self.ui.showusers = !self.ui.showusers,
            KeyCode::F(2) => self.ui.showkey = !self.ui.showkey,
            KeyCode::Enter => {
                // Sending a message
                
                if self.ui.input.is_empty() { return; }

                let msg = format!("{}|{}", self.ui.username, self.ui.input);
                match self.socket.send(msg.as_bytes()) {
                    Ok(_) => {},
                    Err(e) => eprintln!("Failed to send message: {}", e),
                };
                self.ui.input.clear();
            },
            KeyCode::Backspace => {
                self.ui.input.pop();
            },
            KeyCode::Char(c) => self.ui.input.push(c),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(border::PLAIN);
        let style = ratatui::style::Style::default().fg(ratatui::style::Color::Cyan);

        let mut widthleft = area.width;
        let mut heightleft = area.height;
        
        // Room key panel (top)
        if self.ui.showkey {
            let room_key_height = 3.min(heightleft as u16);
            Paragraph::new(Line::from(self.ui.roomkey.clone()))
                .block(block.to_owned().title(" Room Key "))
                .style(style.to_owned())
                .render(Rect { x: 0, y: 0, width: widthleft, height: room_key_height }, buf);
            heightleft -= room_key_height;
        }

        // Users sidebar (left)
        if self.ui.showusers && !self.ui.roomusers.is_empty() {
            let users_width = 20.min(widthleft as u16 - 2);
            widthleft -= users_width;
            
            let users: Vec<Line<'static>> = self.ui.roomusers.iter().cloned().collect();
            Paragraph::new(users)
                .block(block.to_owned().title(" Users "))
                .style(style.to_owned())
                .render(Rect { x: 0, y: area.height - heightleft, width: users_width, height: heightleft }, buf);
        }

        // Chat history (center-right)
        Paragraph::new(self.ui.history.clone())
            .block(block.to_owned().title(Line::from(" Blossom ").centered()))
            .style(style.to_owned())
            .render(Rect { x: area.width - widthleft, y: area.height - heightleft, width: widthleft, height: heightleft.saturating_sub(4) }, buf);

        // Message input (bottom)
        let input_height = 4.min(heightleft as u16);
        Paragraph::new(&*self.ui.input)
            .block(block.title(" Message "))
            .style(style)
            .render(Rect { x: area.width - widthleft, y: area.height - input_height, width: widthleft, height: input_height }, buf);
    }
}
