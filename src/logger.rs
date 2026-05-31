use log::{debug, error, info};
use std::sync::OnceLock;

/// Application-wide singleton logger with a fixed identifier prefix.
///
/// `Logger` wraps the `env_logger` backend and prepends a configurable
/// `ident` string to every log message (e.g. `"numpad: device found"`).
///
/// The singleton is stored in a [`OnceLock`] and must be initialized once
/// via [`Logger::init`] before any logging call. Use the convenience macros
/// [`crate::log_debug!`], [`crate::log_info!`], [`crate::log_error!`] and [`crate::log_fatal`]  for
/// formatted logging throughout the codebase.
pub struct Logger {
    /// Prefix prepended to every log message (e.g. the application name).
    ident: String,
}

/// Global singleton instance of [`Logger`], initialized at most once.
static INSTANCE: OnceLock<Logger> = OnceLock::new();

impl Logger {
    /// Initializes the global logger singleton.
    ///
    /// Configures `env_logger` with the given log level filter and stores a
    /// `Logger` instance with the provided `ident` in the global [`INSTANCE`].
    ///
    /// This function must be called once before any logging occurs. Subsequent
    /// calls are silently ignored (the `OnceLock` guarantees at-most-one
    /// initialization).
    ///
    /// # Arguments
    ///
    /// * `ident` - Prefix string prepended to all log messages.
    /// * `level` - Minimum log level to emit (e.g. `log::LevelFilter::Info`).
    ///
    /// # Example
    ///
    /// ```rust
    /// Logger::init("app", log::LevelFilter::Debug);
    /// ```
    pub fn init(ident: &str, level: log::LevelFilter) {
        env_logger::Builder::new().filter_level(level).init();
        INSTANCE
            .set(Logger {
                ident: ident.to_string(),
            })
            .ok();
    }

    /// Returns a reference to the global `Logger` singleton.
    ///
    /// # Panics
    ///
    /// Panics if [`Logger::init`] has not been called yet.
    ///
    /// # Example
    ///
    /// ```rust
    /// let logger = Logger::get();
    /// logger.info("ready");
    /// ```
    pub fn get() -> &'static Logger {
        INSTANCE
            .get()
            .expect("Logger not initialized; call Logger::init() first")
    }

    /// Emits a `DEBUG` level message prefixed with the logger's `ident`.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to log.
    pub fn debug(&self, msg: &str) {
        debug!("{}: {}", self.ident, msg);
    }

    /// Emits an `INFO` level message prefixed with the logger's `ident`.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to log.
    pub fn info(&self, msg: &str) {
        info!("{}: {}", self.ident, msg);
    }

    /// Emits an `ERROR` level message prefixed with the logger's `ident`.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to log.
    pub fn error(&self, msg: &str) {
        error!("{}: {}", self.ident, msg);
    }

    /// Converts a log level string to the corresponding [`log::LevelFilter`].
    ///
    /// The comparison is case-insensitive. Recognized values are `"debug"`,
    /// `"info"`, `"warn"`, and `"error"`. Any unrecognized string falls back
    /// to `LevelFilter::Info`.
    ///
    /// | Input string | Returned level        |
    /// |--------------|-----------------------|
    /// | `"debug"`    | `LevelFilter::Debug`  |
    /// | `"info"`     | `LevelFilter::Info`   |
    /// | `"warn"`     | `LevelFilter::Warn`   |
    /// | `"error"`    | `LevelFilter::Error`  |
    /// | *(other)*    | `LevelFilter::Info`   |
    ///
    /// # Arguments
    ///
    /// * `level` - A string representation of the desired log level.
    ///
    /// # Example
    ///
    /// ```rust
    /// let level = Logger::string_to_level("debug");
    /// Logger::init("app", level);
    /// ```
    pub fn string_to_level(level: &str) -> log::LevelFilter {
        let lower_case = level.to_lowercase();
        let mut log_level: log::LevelFilter = log::LevelFilter::Info;
        if lower_case == "debug" {
            log_level = log::LevelFilter::Debug;
        } else if lower_case == "info" {
            log_level = log::LevelFilter::Info;
        } else if lower_case == "warn" {
            log_level = log::LevelFilter::Warn;
        } else if lower_case == "error" {
            log_level = log::LevelFilter::Error;
        }
        log_level
    }
}

/// Logs a formatted `DEBUG` message via the global [`Logger`] singleton.
///
/// Accepts the same format arguments as [`format!`].
/// [`Logger::init`] must have been called before using this macro.
///
/// # Example
///
/// ```rust
/// log_debug!("key pressed: {} at [{}, {}]", key.name, row, col);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::Logger::get().debug(&format!($($arg)*))
    };
}

/// Logs a formatted `INFO` message via the global [`Logger`] singleton.
///
/// Accepts the same format arguments as [`format!`].
/// [`Logger::init`] must have been called before using this macro.
///
/// # Example
///
/// ```rust
/// log_info!("layout '{}' loaded successfully", layout.name);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::Logger::get().info(&format!($($arg)*))
    };
}

/// Logs a formatted `ERROR` message via the global [`Logger`] singleton.
///
/// Accepts the same format arguments as [`format!`].
/// [`Logger::init`] must have been called before using this macro.
///
/// # Example
///
/// ```rust
/// log_error!("failed to open device: {}", err);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::Logger::get().error(&format!($($arg)*))
    };
}

/// Logs a formatted `ERROR` message via the global [`Logger`] singleton.
///
/// Accepts the same format arguments as [`format!`].
/// [`Logger::init`] must have been called before using this macro.
///
/// Calls `std::process::exit(1) if an error occurs.
///
/// # Example
///
/// ```rust
/// log_fatal!("failed to open device: {}", err);
/// ```
#[macro_export]
macro_rules! log_fatal {
    ($($arg:tt)*) => {
        $crate::logger::Logger::get().error(&format!($($arg)*));
        std::process::exit(1)
    };
}
