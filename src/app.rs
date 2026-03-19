use core::time;
use std::{io, net::UdpSocket, process::Child, sync::mpsc::Sender};
use base64::{prelude::BASE64_STANDARD, Engine};
use ratatui::{buffer::Buffer, crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, layout::Rect, style::Stylize, symbols::border, text::Line, widgets::{Block, Paragraph, Widget}, Frame};

use crate::{codec, server, yggdrasil, error::{Result, BlossomError}};

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
        let roomkeybytes = codec::convert_to_32_bytes(&connectaddr);
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
        if let Err(e) = yggdrasil::get_ipv6() {
            eprintln!("Warning: Yggdrasil may not be ready: {}", e);
        }

        let decodedroomkey = match BASE64_STANDARD.decode(roomkey.clone()) {
            Ok(decoded) => decoded,
            Err(e) => return Err(BlossomError::InvalidData(format!("Base64 decode error: {}", e)))
        };

        // Convert 32-byte array back to string (trim trailing 'g' padding)
        let connectaddr = match String::from_utf8(decodedroomkey.clone()) {
            Ok(decoded_str) => codec::strip_padding(&decoded_str.as_bytes()),
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

        // Send the username as the initial message to the server
        if self.socket.send(self.ui.username.as_bytes()).is_err() {
            eprintln!("Failed to send username");
        }

        // Main loop that runs until the exit flag is set
        while !self.exit {
            // Attempt to receive data from the socket
            match self.socket.recv_from(buffer.as_mut()) {
                Ok((size, _)) => {
                    if size == 0 { continue; }

                    match buffer[0] {
                        b'J' => {
                            // Join notification: J{username}
                            let username = match String::from_utf8(buffer[1..size].to_vec()) {
                                Ok(u) => u.trim().to_string(),
                                Err(_) => { eprintln!("Unicode error in join message"); continue; }
                            };
                            if !username.is_empty() {
                                self.ui.roomusers.push(Line::from(username.clone()).red());
                                self.ui.history.push(Line::from(vec![
                                    username.red(),
                                    " joined the room".red(),
                                ]));
                            }
                        }
                        b'M' => {
                            // Chat message: M{username}\x00{message}
                            let data = &buffer[1..size];
                            if let Some(sep) = data.iter().position(|&b| b == 0) {
                                let username = match String::from_utf8(data[..sep].to_vec()) {
                                    Ok(u) => u,
                                    Err(_) => { eprintln!("Unicode error in username"); continue; }
                                };
                                let message = match String::from_utf8(data[sep + 1..].to_vec()) {
                                    Ok(m) => m,
                                    Err(_) => { eprintln!("Unicode error in message"); continue; }
                                };
                                self.ui.history.push(Line::from(vec![
                                    format!("[{}] ", username).cyan(),
                                    message.gray(),
                                ]));
                            } else {
                                eprintln!("Malformed chat message: missing null separator");
                            }
                        }
                        _ => {
                            eprintln!("Unknown message type: 0x{:02x}", buffer[0]);
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
        
        // Graceful shutdown
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

                let msg = format!("M{}\x00{}", self.ui.username, self.ui.input);
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
        let chat_height = heightleft - 4;
        let chat_inner_height = chat_height - 2; // lines of chat history = window height - message input height - border height
        let scroll_offset = (self.ui.history.len() as u16).saturating_sub(chat_inner_height);
        Paragraph::new(self.ui.history.clone())
            .block(block.to_owned().title(Line::from(" Blossom ").centered()))
            .style(style.to_owned())
            .scroll((scroll_offset, 0))
            .render(Rect { x: area.width - widthleft, y: area.height - heightleft, width: widthleft, height: chat_height }, buf);

        // Message input (bottom)
        let input_height = 4.min(heightleft as u16);
        Paragraph::new(&*self.ui.input)
            .block(block.title(" Message "))
            .style(style)
            .render(Rect { x: area.width - widthleft, y: area.height - input_height, width: widthleft, height: input_height }, buf);
    }
}
