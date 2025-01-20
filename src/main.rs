use std::io;
mod crypt;
mod server;
mod tests;
mod yggdrasil;
mod app;
use app::App;

//building a chat app here
fn main() -> io::Result<()> {
    let mut username = String::new();
    let mut roomkey = String::new();
    let mut port = "9191".to_string();
    let help = r#"
Usage: program_name [OPTIONS]

Options:
  -u, --username <USERNAME>    Specify the username to display in the chat.
  -r, --roomkey <ROOMKEY>      Specify the room key to join a chat room.
  -p, --port <PORT>            Specify the port number to use for the connection.
  -h, --help                   Display this help message.
"#;

    for i in 1..std::env::args().len() {
        if let Some(arg) = std::env::args().nth(i) {
            match arg.as_str() {
                "--username" | "-u" => username = std::env::args().nth(i + 1).expect(help),
                "--roomkey" | "-r" => roomkey = std::env::args().nth(i + 1).expect(help),
                "--port" | "-p" => port = std::env::args().nth(i + 1).expect(help),
                "--help" | "-h" => return Ok(println!("{}", help)),
                _ => {}                
            }
        }
    }
    
    let mut terminal = ratatui::init();

    if username.is_empty() {
        username = crypt::generate_rnd_str(10);
    }

    let app_result = if roomkey.is_empty() {
        //BASE64_STANDARD.encode_string(crypt::generate_roomkey(), &mut roomkey);
        App::create_room(username, port)?.run(&mut terminal)
    }
    else {
        App::join_room(username, roomkey, port)?.run(&mut terminal)
    };
    
    ratatui::restore();
    app_result
}
