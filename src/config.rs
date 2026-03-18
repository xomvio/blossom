/// Configuration constants for the Blossom chat application

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

/// Yggdrasil startup timeout in seconds
pub const YGGDRASIL_STARTUP_TIMEOUT_SECS: u64 = 10;

/// Yggdrasil log check interval in milliseconds
pub const YGGDRASIL_LOG_CHECK_INTERVAL_MS: u64 = 200;

/// Yggdrasil configuration file path
pub const YGGDRASIL_CONF_PATH: &'static str = "/tmp/yggdrasil.conf";

/// Yggdrasil log file path
pub const YGGDRASIL_LOG_PATH: &'static str = "/tmp/yggdrasil.log";

/// Default Yggdrasil peer (Armenian public peer)
pub const DEFAULT_YGGDRASIL_PEER: &'static str = "quic://37.186.113.100:1515";