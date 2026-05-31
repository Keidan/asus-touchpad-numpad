use crate::config::models::Config;
use crate::config::models::Layout;
use crate::sys::utils::Utils;
use evdev::KeyCode;
use std::str::FromStr;

/// Parses configuration and keymap files from disk.
///
/// `Parser` is a stateless struct that provides methods to deserialize
/// JSON files into `Config` structure. It also handles
/// basic file access checks before reading.
pub struct Parser;

impl Layout {
    /// Validates the keymap by resolving all key name strings into [`evdev::KeyCode`] values.
    ///
    /// Iterates over every cell of `self.keymap` and converts each string into a
    /// [`KeyCode`] via `evdev::KeyCode::from_str`. On success, returns a 2D grid of
    /// [`KeyCode`] values mirroring the structure of `self.keymap`.
    ///
    /// # Returns
    ///
    /// A `Vec<Vec<KeyCode>>` where each entry is the resolved keycode for that cell.
    ///
    /// # Errors
    ///
    /// Returns an error if any name in the grid is not a valid [`evdev::KeyCode`]
    /// variant name (e.g. a typo or an unsupported key).
    ///
    /// # Example
    ///
    /// ```rust
    /// let keycodes = layout.validate()?;
    /// // keycodes[row][col] is the KeyCode for that cell
    /// ```
    pub fn validate(&self) -> Result<Vec<Vec<KeyCode>>, Box<dyn std::error::Error>> {
        self.keymap
            .iter()
            .map(|row| {
                row.iter()
                    .map(|name| {
                        KeyCode::from_str(name)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                    })
                    .collect()
            })
            .collect()
    }
}

impl Config {
    /// Validates the configuration by checking that `layout_name` refers to an existing layout.
    ///
    /// Searches `self.layouts` for an entry whose `name` matches `self.layout_name`.
    /// Returns a reference to that `Layout` if found.
    ///
    /// # Errors
    ///
    /// Returns an error if no layout in `self.layouts` matches `self.layout_name`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let active_layout = config.validate()?;
    /// println!("Active layout: {}", active_layout.name);
    /// ```
    pub fn validate(&self) -> Result<&Layout, Box<dyn std::error::Error>> {
        let mut name = self.layout_name.clone();
        if name == "auto" {
            name = Utils::detect_layout()?;
        }
        // Check that layout_name exists in layouts
        self.layouts
            .iter()
            .find(|layout| layout.name == name)
            .ok_or_else(|| {
                format!(
                    "The name '{}' was not found in the list of layouts",
                    name
                )
                .into()
            })
    }
}

impl Parser {
    /// Creates a new `Parser` instance.
    ///
    /// `Parser` is stateless; this is a convenience constructor for
    /// consistency with the rest of the API.
    pub fn new() -> Self {
        Parser
    }

    /// Parses a `Config` struct from a JSON file.
    ///
    /// Reads the file at `path` and deserializes its contents into a `Config`.
    /// Delegates to `parse_from_file` for file access checks and deserialization.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON configuration file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not found, is read-only, or contains
    /// invalid JSON that does not match the `Config` structure.
    ///
    /// # Example
    ///
    /// ```rust
    /// let parser = Parser::new();
    /// let config = parser.parse_config("/opt/app/config.json")?;
    /// ```
    pub fn parse_config(&self, path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        self.parse_from_file(path)
    }

    /// Reads a file from disk and deserializes its JSON content into type `T`.
    ///
    /// Performs two checks before reading:
    /// 1. The file must exist (checked via `fs::metadata`).
    /// 2. The file must not be read-only.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements `serde::de::DeserializeOwned`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file to read and deserialize.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist.
    /// - The file permissions are read-only.
    /// - The file content cannot be read or deserialized into `T`.
    fn parse_from_file<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(path);

        if metadata.is_err() {
            return Err(format!("The file '{}' could not be found", path).into());
        }

        if metadata.unwrap().permissions().readonly() {
            return Err(format!("Permission denied for the '{}' file", path).into());
        }

        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
}
