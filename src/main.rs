use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream, UdpSocket}, thread, time, vec};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use magic_crypt::generic_array::GenericArray;
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::Line, widgets::{Block, Paragraph, Widget}, Frame};
use aes_gcm::{
    aead::{Aead, AeadCore, AeadMut, KeyInit, OsRng, Key}, aes::cipher, Aes256Gcm, Nonce // Or `Aes128Gcm`
};


use utils::generate_aesgcm;
mod utils;

//building a chat app here
fn main() -> io::Result<()> {
    let mut username = String::new();
    let mut roomkey = String::new();
    for i in 1..std::env::args().len() {
        match std::env::args().nth(i) {
            Some(arg) => {
                match arg.as_str() {
                    "--username" | "-u" => username = std::env::args().nth(i + 1).unwrap(),
                    "--roomkey" | "-r" => roomkey = std::env::args().nth(i + 1).unwrap(),
                    _ => {}
                }
            }
            None => {}
        }
    }
    
    let mut terminal = ratatui::init();
    let app_result;
   

    if username.is_empty() {
        username = utils::generate_rnd_str(10);
    }
    if roomkey.is_empty() {
        roomkey = utils::generate_rnd_str(32);
    }

    app_result = if std::env::args().nth(1).is_some() {
        App::create_room(username, roomkey).run(&mut terminal)
    }
    else {
        App::join_room(username, roomkey).run(&mut terminal)
    };
    
    ratatui::restore();
    app_result
}

struct App {
    username: String,
    roomkey: String,
    roomusers: Vec<Line<'static>>,
    history: Vec<Line<'static>>,
    socket: UdpSocket,
    cipher: Aes256Gcm,
    //server: String,
    input: String,
    showkey: bool,
    showusers: bool,
    exit: bool,
}

impl App {

    fn create_room(username: String, roomkey: String) -> Self {
        Self {
            username: username.clone(),
            roomkey: roomkey.clone(),
            roomusers: vec![Line::from(username)],
            history: Vec::new(),
            socket: UdpSocket::bind("127.0.0.1:9090").unwrap(),
            cipher: generate_aesgcm(),
            //server: "127.0.0.1:9595".to_string(),
            input: String::new(),
            showkey: false,
            showusers: false,
            exit: false,
        }
    }

    fn join_room(username: String, roomkey: String) -> Self {
        Self {
            username: username.clone(),
            roomkey: roomkey.clone(),
            roomusers: vec![Line::from(username)],
            history: Vec::new(),
            socket: UdpSocket::bind("127.0.0.1:9191").unwrap(),
            cipher: generate_aesgcm(),
            //server: "127.0.0.1:9595".to_string(),
            input: String::new(),
            showkey: false,
            showusers: false,
            exit: false,
        }
    }

    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {

        self.socket.connect("127.0.0.1:9595").unwrap();
        self.socket.set_nonblocking(true)?;

        /*let socket = UdpSocket::bind("127.0.0.1:9090").unwrap();
        socket.connect(self.server.as_str()).unwrap();*/

        let mut buffer = [0; 1024];
        
        while !self.exit {

            match self.socket.recv_from(buffer.as_mut()) {
                Ok((size, src)) => {
                    //let decrypted = utils::decrypt(&self.cipher, buffer[..size].as_ref()).unwrap();
                    let decrypted = self.decrypt(&buffer[..size]).unwrap();
                    self.history.push(Line::from(decrypted));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // no incoming data, can do other things
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }

            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _=> {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('c') => self.exit(),
                _ => {}
            }
            return
        }
        
        match key_event.code {
            KeyCode::F(1) => self.showusers = !self.showusers,
            KeyCode::F(2) => self.showkey = !self.showkey,
            KeyCode::Enter => {
                self.history.push(Line::from(vec!["[".red().bold(), self.username.clone().into(), "] ".red().bold(), self.input.clone().into()]));
                let encrypted = utils::encrypt(&self.cipher, self.username.clone() + "|" + &self.input);
                //self.history.push(Line::from(encrypted.len().to_string()));
                //let encrypted = self.encrypt(self.username.clone() + "|" + &self.input);
                self.socket.send(&encrypted).unwrap();
                self.input.clear();
            },
            KeyCode::Backspace => {
                self.input.pop();
            },
            KeyCode::Char(c) => self.input.push(c),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }


    pub fn encrypt(&mut self, message: String) -> Vec<u8> {
        // Generate a random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt the message
        let ciphertext = self.cipher
            .encrypt(&nonce, message.as_bytes())
            .expect("encryption failure");
        
        // Combine nonce and ciphertext into a single vector
        let mut encrypted = Vec::new();
        encrypted.extend_from_slice(&nonce);
        encrypted.extend_from_slice(&ciphertext);
        
        encrypted
    }

    pub fn decrypt(&mut self, encrypted_data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        // The first 12 bytes should be the nonce
        if encrypted_data.len() < 12 {
            return Err("Encrypted data too short".into());
        }
        
        // Split the data into nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt the message
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption error: {}", e))?;
        
        // Convert the decrypted bytes to a string
        String::from_utf8(plaintext)
            .map_err(|e| e.into())
    }

    /*pub fn encrypt(&mut self, message: String) -> Vec<u8> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        /*let mut xmessage= Vec::new();
        xmessage.extend_from_slice(message.as_bytes());*/
        let nonce: [u8; 12] = [0; 12];
        self.cipher.encrypt(&nonce.into(), message.as_bytes()).unwrap()
    }

    pub fn decrypt(&mut self, /*nonce: &GenericArray<u8, cipher::consts::U12>,*/ message: Vec<u8>) -> Vec<u8> {
        //let nonce = GenericArray::from_slice(&nonce);
        let nonce: [u8; 12] = [0; 12];
        self.cipher.decrypt(&nonce.into(), message.as_slice()).unwrap()
    }*/
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(border::PLAIN);
        let style = ratatui::style::Style::default().fg(ratatui::style::Color::Cyan);

        let mut widthleft = area.width;
        let mut heightleft = area.height;
        
        if self.showkey {
            //widthleft -= 6;
            heightleft -= 3;
            Paragraph::new(Line::from(self.roomkey.clone()))
                .block(block.to_owned().title(" Room Key "))
                .style(style.to_owned())
                .render(Rect { x: 0, y: 0, width: widthleft, height: 3 }, buf);
        }

        if self.showusers {
            widthleft -= 20;
            let mut users = Vec::new();
            for user in self.roomusers.iter() {
                users.push(Line::from(user.clone().to_string()));
            }
            Paragraph::new(users)
                .block(block.to_owned().title(" Users "))
                .style(style.to_owned())
                .render(Rect { x: 0, y: area.height - heightleft, width: 20, height: heightleft }, buf);
        }

        let mut history = Vec::new();
        for message in &self.history {
            history.push(Line::from(message.to_owned()));
        }
        Paragraph::new(history)
            .block(block.to_owned().title(Line::from(" ArtiChat ").centered()))
            .style(style.to_owned())
            .render(Rect { x: area.width - widthleft, y: area.height - heightleft, width: widthleft, height: heightleft - 4 }, buf);

        let input = Paragraph::new(self.input.clone());
        input.block(block.title(" Message "))
            .style(style)
            .render(Rect { x: area.width - widthleft, y: area.height - 4, width: widthleft, height: 4 }, buf);
    }
}
