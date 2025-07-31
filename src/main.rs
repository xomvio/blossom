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
    #[arg(short, long, default_value = "9191")]
    port: String,
}

//building a chat app here
fn main() -> io::Result<()> {

    let cli = Cli::parse();

    let mut terminal = ratatui::init();

    let username = cli.username.unwrap_or_else(|| crypt::generate_rnd_str(10));

    let app_result = match cli.roomkey {
        Some(roomkey) => App::join_room(username, roomkey, cli.port)?.run(&mut terminal),
        None => App::create_room(username, cli.port)?.run(&mut terminal)
    };
    
    ratatui::restore();
    app_result
}
