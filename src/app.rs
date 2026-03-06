use core::time;
use std::{io::{self, Error, ErrorKind}, net::UdpSocket, process::Child, sync::mpsc::Sender, thread};
use base64::{prelude::BASE64_STANDARD, Engine};
use ratatui::{buffer::Buffer, crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, layout::Rect, style::Stylize, symbols::border, text::Line, widgets::{Block, Paragraph, Widget}, Frame};
//use aes_gcm::Aes256Gcm;

use crate::{crypt::convert_to_32_bytes, server, yggdrasil};

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

    pub fn create_room(username: String, port: String) -> Result<Self, Error> {
        let (connectaddr, yggdr, servershutter) = server::create(&port)?;
        let roomkeybytes = convert_to_32_bytes(connectaddr.clone()); // gg(g) in the end
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

    pub fn join_room(username: String, roomkey: String, port: String) -> Result<Self, Error> {
        let yggdr = yggdrasil::start(&port)?;
        let _ = yggdrasil::get_ipv6();

        let decodedroomkey = match BASE64_STANDARD.decode(roomkey.clone()) {
            Ok(decoded) => decoded,
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e))
        };

        let connectaddr = match String::from_utf8(decodedroomkey.clone()) {
            Ok(decoded) => decoded.replace("g", ""),
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e))
        };
        
        //let roomkeybtes = turn_to_32_bytes(connectaddr.clone());

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


    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        let mut error: Option<Error>;
        // Attempt to establish a connection to the specified address
        match self.socket.connect(self.connectaddr.clone()) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
        
        match self.socket.set_nonblocking(true) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        let mut buffer = [0; 10240]; // Storing incoming data here. this limit may change

        // This had to be changed
        thread::sleep(time::Duration::from_millis(3000));

        // Send the username as the initial message to the server
        error = self.socket.send(self.ui.username.as_bytes()).err();

        // Main loop that runs until the exit flag is set
        while !self.exit {
            // Attempt to receive data from the socket
            match self.socket.recv_from(buffer.as_mut()) {
                Ok((size, _)) => {
                    // Check if the received data is smaller than 12 bytes, indicating a username
                    if size < 12 {
                        let username = match String::from_utf8(buffer[..size].to_vec()) {
                            Ok(username) => username,
                            Err(_) => {
                                error = Some(Error::other("An unexpected behaved connection occured when getting join information. Blossom will close itself for security reasons."));
                                break;
                            }
                        };
                        // Add the new user to the room users list and history
                        self.ui.roomusers.push(Line::from(username.clone()).red());
                        self.ui.history.append(&mut vec![Line::from(vec![username.to_owned().red(), " joined the room".red()])]);
                    } else {

                        // ENCRYPTION IS DEPRECATED
                        // Decrypt the received message
                        //let decrypted =crypt::decrypt(&self.cipher, buffer[..size].as_ref()).unwrap_or("Failed to decrypt this message".to_string());

                        let decrypted = String::from_utf8(buffer[..size].to_vec()).unwrap_or("Failed to get message".to_string());
                        // Split the decrypted message into username and message parts
                        let (username, message) = match decrypted.split_once('|') {
                            Some((username, message)) => (username, message),
                            None => {
                                error = Some(Error::other("An unexpected behaved connection occured when processing decrypted message. Blossom will close itself for security reasons."));
                                break;
                            }
                        };

                        // Add the message to the chat history
                        self.ui.history.append(&mut vec![Line::from(vec!["[".cyan(), username.to_owned().cyan(), "] ".cyan(), message.to_owned().gray()])]);
                    }
                }
                // Handle the case where the socket would block, indicating no data is available
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Can perform other tasks here if needed
                }
                // Handle any other errors that occur during reception
                Err(e) => {
                    println!("Error: ha!{}", e); // Log the error
                    break; // Exit the loop on error
                }
            }

            // Draw the current state of the terminal UI
            terminal.draw(|f| self.draw(f))?;

            // Handle any user input events
            error = self.handle_events().err();
            if error.is_some() { break; }
        }
        
        // Perform a graceful shutdown of the application

        // Terminate the yggdrasil process
        match self.yggdr.kill(){
            Ok(_) => {},
            Err(e) => error = Some(Error::other(format!("Failed to terminate yggdrasil process: {}\r\n{}", e, "Start and quit Blossom again to try to fix this.")))
        }
        
        if let Some(servershutter) = &self.servershutter {
            // Send a shutdown signal to the server
            match servershutter.send(()) {
                Ok(_) => {},
                Err(e) => error = Some(Error::other(format!("Failed to send shutdown signal to the server: {}\r\n{}", e, "Start and quit Blossom again to try to fix this.")))
            }
            // Delete the yggdrasil address
            match yggdrasil::del_addr(self.connectaddr.clone()) {
                Ok(_) => {},
                Err(e) => error = Some(Error::other(format!("Failed to delete yggdrasil address: {}\r\n{}", e, "Start and quit Blossom again to try to fix this.")))
            }
        }
        // Delete the configuration file        
        match yggdrasil::delconf() {
            Ok(_) => {},
            Err(e) => error = Some(Error::other(format!("Failed to delete configuration file: {}\r\n{}", e, "Start and quit Blossom again to try to fix this.")))
        }

        // Delete the log file
        match yggdrasil::del_log() {
            Ok(_) => {},
            Err(e) => error = Some(Error::other(format!("Failed to delete log file: {}\r\n{}", e, "Start and quit Blossom again to try to fix this.")))
        }

        if let Some(error) = error {
            return Err(error);
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
                
                // if the input is empty, do nothing
                if self.ui.input.is_empty() { return; }

                // ENCRYPTION IS DEPRECATED
                //let encrypted = crypt::encrypt(&self.cipher, self.ui.username.clone() + "|" + &self.ui.input);
                //self.socket.send(&encrypted).unwrap();

                self.socket.send(format!("{}|{}", self.ui.username, self.ui.input).as_bytes()).unwrap();
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
        
        if self.ui.showkey {
            heightleft -= 3;
            Paragraph::new(Line::from(self.ui.roomkey.clone()))
                .block(block.to_owned().title(" Room Key "))
                .style(style.to_owned())
                .render(Rect { x: 0, y: 0, width: widthleft, height: 3 }, buf);
        }

        if self.ui.showusers {
            widthleft -= 20;
            let mut users = Vec::new();
            for user in self.ui.roomusers.iter() {
                users.push(Line::from(user.clone().to_string()));
            }
            Paragraph::new(users)
                .block(block.to_owned().title(" Users "))
                .style(style.to_owned())
                .render(Rect { x: 0, y: area.height - heightleft, width: 20, height: heightleft }, buf);
        }

        let mut history = Vec::new();
        for message in &self.ui.history {
            history.push(message.to_owned());
        }
        if history.len() > (heightleft - 6) as usize {
            history.drain(0..(history.len() - (heightleft - 6) as usize));
        }
        Paragraph::new(history)
            .block(block.to_owned().title(Line::from(" Blossom ").centered()))
            .style(style.to_owned())
            .render(Rect { x: area.width - widthleft, y: area.height - heightleft, width: widthleft, height: heightleft - 4 }, buf);

        let input = Paragraph::new(self.ui.input.clone());
        input.block(block.title(" Message "))
            .style(style)
            .render(Rect { x: area.width - widthleft, y: area.height - 4, width: widthleft, height: 4 }, buf);
    }
}
