use clap::Parser;

mod app;
mod config;
mod crypt;
mod error;
mod server;
mod yggdrasil;

use app::App;
use config::{Config, RuntimeConfig};
use error::Result;

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

/// Entry point for the Blossom chat application
fn main() -> Result<()> {
    let result = run();
    
    // Ensure terminal is always restored, even on error
    ratatui::restore();
    
    result
}

/// Main application logic separated for better error handling
fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut terminal = ratatui::init();
    
    let username = resolve_username(cli.username)?;
    let port = cli.port.unwrap_or_else(|| get_default_port(cli.roomkey.is_some()));
    
    let mut app = if let Some(roomkey) = cli.roomkey {
        App::join_room(username, roomkey, port.clone())?
    } else {
        App::create_room(username, port)?
    };
    
    Ok(app.run(&mut terminal)?)
}

/// Resolves the username from CLI args or generates a random one
fn resolve_username(username_opt: Option<String>) -> Result<String> {
    match username_opt {
        Some(username) => Ok(username),
        None => crypt::generate_random_username()
            .or_else(|_| Ok(Config::DEFAULT_USERNAME.to_string())),
    }
}

/// Gets the appropriate default port based on whether joining or creating
fn get_default_port(is_joining: bool) -> String {
    if is_joining {
        Config::DEFAULT_JOIN_PORT.to_string()
    } else {
        Config::DEFAULT_CREATE_PORT.to_string()
    }
}
