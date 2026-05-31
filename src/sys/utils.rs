use std::env;
use std::path::Path;
use std::path::PathBuf;

/// General-purpose utility functions.
///
/// `Utils` is a stateless struct grouping helper methods used across the
/// application for common operations such as checking file existence,
/// resolving directory paths, locating files relative to the running, etc...
/// executable.
pub struct Utils;

impl Utils {
    /// Checks whether a file exists at the given path.
    ///
    /// Uses `fs::metadata` to probe the path. Returns `true` if the metadata
    /// call succeeds (i.e. the file or directory exists and is accessible),
    /// `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the file or directory to check.
    ///
    /// # Example
    ///
    /// ```rust
    /// if Utils::file_exists("/opt/app/config.json") {
    ///     // load config
    /// }
    /// ```
    pub fn file_exists(file: &str) -> bool {
        let metadata = std::fs::metadata(file);
        return !metadata.is_err();
    }

    /// Returns the parent directory of the given executable path.
    ///
    /// If the path has no parent component (e.g. a bare filename with no
    /// directory part), falls back to `"/"`.
    ///
    /// # Arguments
    ///
    /// * `exe` - Full path to the executable, typically from [`env::current_exe`].
    ///
    /// # Example
    ///
    /// ```rust
    /// let exe = std::env::current_exe().unwrap();
    /// let dir = Utils::get_dirname(&exe);
    /// println!("Running from: {}", dir.display());
    /// ```
    pub fn get_dirname(exe: &PathBuf) -> &Path {
        if let Some(dir) = exe.parent() {
            return dir;
        }
        Path::new("/")
    }

    /// Resolves the absolute path of a file located in the same directory
    /// as the running executable.
    ///
    /// Retrieves the current executable path via [`env::current_exe`], extracts
    /// its parent directory with [`Utils::get_dirname`], then joins `name` to it.
    ///
    /// This is useful for locating bundled assets (config files, keymaps, etc.)
    /// that are distributed alongside the binary.
    ///
    /// # Arguments
    ///
    /// * `name` - Filename (or relative sub-path) to resolve against the
    ///   executable's directory.
    ///
    /// # Returns
    ///
    /// The resolved absolute path as a `String`.
    ///
    /// # Exits
    ///
    /// Calls `std::process::exit(1)` if the current executable path cannot
    /// be determined.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config_path = Utils::get_file_location("config.json");
    /// // e.g. "/opt/app/config.json" if the binary lives in /opt/app/
    /// ```
    pub fn get_file_location(name: &str) -> String {
        let file: String;
        match env::current_exe() {
            Ok(exe_path) => {
                println!();
                file = Utils::get_dirname(&exe_path)
                    .join(name)
                    .display()
                    .to_string();
            }
            Err(e) => {
                eprintln!("Failed to get current exe path: {e}");
                std::process::exit(1);
            }
        }
        file
    }

    /// Detects the current board name from the DMI system information.
    ///
    /// Reads `/sys/class/dmi/id/board_name`, trims whitespace, and returns
    /// the value as a lowercase string. The result is typically matched against
    /// known layout names to select the appropriate numpad [`crate::config::models::Layout`].
    ///
    /// # Returns
    ///
    /// The board name as a lowercase trimmed `String` (e.g. `"ux433fa"`).
    ///
    /// # Errors
    ///
    /// Returns an error if `/sys/class/dmi/id/board_name` cannot be read
    /// (e.g. non-Linux system, insufficient permissions, or missing DMI support).
    pub fn detect_layout() -> Result<String, Box<dyn std::error::Error>> {
        Ok(std::fs::read_to_string("/sys/class/dmi/id/board_name")?
            .trim()
            .to_lowercase()
            .to_string())
    }
}
