use std::time::Duration;

/// Configuration constants for the Blossom chat application
pub struct Config;

impl Config {
    /// Default port for creating a room
    pub const DEFAULT_CREATE_PORT: &'static str = "9191";
    
    /// Default port for joining a room
    pub const DEFAULT_JOIN_PORT: &'static str = "9192";
    
    /// Server port for Yggdrasil communication
    pub const SERVER_PORT: &'static str = "9595";
    
    /// Maximum buffer size for network messages
    pub const MAX_BUFFER_SIZE: usize = 10240;
    
    /// Default username length when generating random usernames
    pub const DEFAULT_USERNAME_LENGTH: usize = 10;
    
    /// Default username fallback
    pub const DEFAULT_USERNAME: &'static str = "Guest";
    
    /// Event polling timeout in milliseconds
    pub const EVENT_POLL_TIMEOUT_MS: u64 = 100;
    
    /// Yggdrasil startup wait time in milliseconds
    pub const YGGDRASIL_STARTUP_WAIT_MS: u64 = 3000;
    
    /// Yggdrasil startup timeout in seconds
    pub const YGGDRASIL_STARTUP_TIMEOUT_SECS: u64 = 10;
    
    /// Yggdrasil log check interval in milliseconds
    pub const YGGDRASIL_LOG_CHECK_INTERVAL_MS: u64 = 200;
    
    /// Temporary directory for Yggdrasil files
    pub const YGGDRASIL_TEMP_DIR: &'static str = "/tmp";
    
    /// Yggdrasil configuration file path
    pub const YGGDRASIL_CONF_PATH: &'static str = "/tmp/yggdrasil.conf";
    
    /// Yggdrasil log file path
    pub const YGGDRASIL_LOG_PATH: &'static str = "/tmp/yggdrasil.log";
    
    /// Default Yggdrasil peer (Armenian public peer)
    pub const DEFAULT_YGGDRASIL_PEER: &'static str = "quic://37.186.113.100:1515";
    
    /// Maximum message history to keep in memory
    pub const MAX_MESSAGE_HISTORY: usize = 1000;
    
    /// UI sidebar width for users list
    pub const USERS_SIDEBAR_WIDTH: u16 = 20;
    
    /// UI room key panel height
    pub const ROOM_KEY_PANEL_HEIGHT: u16 = 3;
    
    /// UI message input panel height
    pub const MESSAGE_INPUT_PANEL_HEIGHT: u16 = 4;
}

/// Runtime configuration that can be modified
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub username: String,
    pub port: String,
    pub roomkey: Option<String>,
    pub show_users: bool,
    pub show_room_key: bool,
}

impl RuntimeConfig {
    pub fn new(username: String, port: String, roomkey: Option<String>) -> Self {
        Self {
            username,
            port,
            roomkey,
            show_users: true,
            show_room_key: true,
        }
    }
    
    pub fn event_poll_timeout() -> Duration {
        Duration::from_millis(Config::EVENT_POLL_TIMEOUT_MS)
    }
    
    pub fn yggdrasil_startup_wait() -> Duration {
        Duration::from_millis(Config::YGGDRASIL_STARTUP_WAIT_MS)
    }
    
    pub fn yggdrasil_startup_timeout() -> Duration {
        Duration::from_secs(Config::YGGDRASIL_STARTUP_TIMEOUT_SECS)
    }
    
    pub fn yggdrasil_log_check_interval() -> Duration {
        Duration::from_millis(Config::YGGDRASIL_LOG_CHECK_INTERVAL_MS)
    }
}
