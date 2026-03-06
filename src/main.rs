use std::io;
use clap::Parser;
mod crypt;
mod server;
mod yggdrasil;
mod app;
use app::App;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    username: Option<String>,
    #[arg(short, long)]
    roomkey: Option<String>,
    #[arg(short, long)]
    port: Option<String>,
}

//building a chat app here
fn main() -> io::Result<()> {

    let cli = Cli::parse();

    let mut terminal = ratatui::init();

    let username = cli.username.unwrap_or_else(|| crypt::generate_rnd_str(10).unwrap_or("Guest".to_string()));

    let app_result = match cli.roomkey {
        Some(roomkey) => App::join_room(username, roomkey, cli.port.unwrap_or( "9192".to_string()))?.run(&mut terminal),
        None => App::create_room(username, cli.port.unwrap_or("9191".to_string()))?.run(&mut terminal)
    };
    
    ratatui::restore();
    app_result
}
