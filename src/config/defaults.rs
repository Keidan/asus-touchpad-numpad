use crate::config::models::{
    Zone, BrightnessLevels, Config, 
    Identify, Layout, Zones
};


/// Provides a default configuration file writer for the application.
///
/// `DefaultConfig` generates a ready-to-use `Config` struct populated with
/// known laptop numpad layouts and writes it as a pretty-printed JSON file.
pub struct DefaultConfig;

impl DefaultConfig {
    /// Writes the default application configuration to a JSON file.
    ///
    /// Generates a [`Config`] struct with `log_level` set to `"info"` and
    /// `layout_name` set to `"auto"`, which enables automatic layout selection
    /// based on the board name read from `/sys/class/dmi/id/board_name`.
    ///
    /// The following layouts are bundled by default:
    ///
    /// | Layout name | Note                          |
    /// |-------------|-------------------------------|
    /// | `g533qr`    | ASUS ROG Strix SCAR 15        |
    /// | `ux433fa`   | ASUS ZenBook 14               |
    /// | `gx701`     | ASUS ROG Zephyrus S17         |
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the output JSON file. Created or overwritten.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails or if the file cannot be written.
    ///
    /// # Example
    ///
    /// ```rust
    /// DefaultConfig::write("/etc/numpad/config.json").expect("Failed to write default config");
    /// ```
    pub fn write(file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = Config {
            log_level: "info".to_string(),
            layout_name: "auto".to_string(),
            identify: Identify { max_attempts: 5, retry_delay_ms: 100 },
            layouts: vec![
                Layout { 
                    name: "g533qr".to_string(),
                    cols: 5,
                    rows: 4,
                    zones: Zones {
                        numlock: Zone { x_min: 0.80, x_max: 1.0, y_min: 0.0, y_max: 0.24 },
                        brightness_calculator: Zone { x_min: 0.0, x_max: 0.06, y_min: 0.0, y_max: 0.07},
                    },
                    double_tap_delay_ms: 250,
                    allow_calculator: true,
                    top_offset: 0.10,
                    keymap: vec![
                        vec![ "KEY_KP7".to_string(), "KEY_KP8".to_string(), "KEY_KP9".to_string(), "KEY_KPSLASH".to_string(), "KEY_BACKSPACE".to_string() ],
                        vec![ "KEY_KP4".to_string(), "KEY_KP5".to_string(), "KEY_KP6".to_string(), "KEY_KPASTERISK".to_string(), "KEY_BACKSPACE".to_string() ],
                        vec![ "KEY_KP1".to_string(), "KEY_KP2".to_string(), "KEY_KP3".to_string(), "KEY_KPMINUS".to_string(), "KEY_KPENTER".to_string() ],
                        vec![ "KEY_KP0".to_string(), "KEY_KP0".to_string(), "KEY_KPDOT".to_string(), "KEY_KPPLUS".to_string(), "KEY_KPENTER".to_string() ]
                    ],
                    brightness_levels: BrightnessLevels { low: 1, med:  24, high: 31 }
                },
                Layout { 
                    name: "ux433fa".to_string(),
                    cols: 5,
                    rows: 4,
                    zones: Zones {
                        numlock: Zone { x_min: 0.95, x_max: 1.0, y_min: 0.0, y_max: 0.09 },
                        brightness_calculator: Zone { x_min: 0.0, x_max: 0.06, y_min: 0.0, y_max: 0.07},
                    },
                    double_tap_delay_ms: 250,
                    allow_calculator: true,
                    top_offset: 0.10,
                    keymap: vec![
                        vec![ "KEY_KP7".to_string(), "KEY_KP8".to_string(), "KEY_KP9".to_string(), "KEY_KPSLASH".to_string(), "KEY_BACKSPACE".to_string() ],
                        vec![ "KEY_KP4".to_string(), "KEY_KP5".to_string(), "KEY_KP6".to_string(), "KEY_KPASTERISK".to_string(), "KEY_BACKSPACE".to_string() ],
                        vec![ "KEY_KP1".to_string(), "KEY_KP2".to_string(), "KEY_KP3".to_string(), "KEY_KPMINUS".to_string(), "KEY_KPENTER".to_string() ],
                        vec![ "KEY_KP0".to_string(), "KEY_KP0".to_string(), "KEY_KPDOT".to_string(), "KEY_KPPLUS".to_string(), "KEY_KPENTER".to_string() ]
                    ],
                    brightness_levels: BrightnessLevels { low: 1, med:  24, high: 31 }
                },
                Layout { 
                    name: "gx701".to_string(),
                    cols: 4,
                    rows: 5,
                    zones: Zones {
                        numlock: Zone { x_min: 0.95, x_max: 1.0, y_min: 0.0, y_max: 0.09 },
                        brightness_calculator: Zone { x_min: 0.0, x_max: 0.06, y_min: 0.0, y_max: 0.07},
                    },
                    double_tap_delay_ms: 250,
                    allow_calculator: true,
                    top_offset: 0.,
                    keymap: vec![
                        vec![ "KEY_CALC".to_string(), "KEY_KPSLASH".to_string(), "KEY_KPASTERISK".to_string(), "KEY_KPMINUS".to_string() ],
                        vec![ "KEY_KP7".to_string(), "KEY_KP8".to_string(), "KEY_KP9".to_string(), "KEY_KPPLUS".to_string() ],
                        vec![ "KEY_KP4".to_string(), "KEY_KP5".to_string(), "KEY_KP6".to_string(), "KEY_KPPLUS".to_string() ],
                        vec![ "KEY_KP1".to_string(), "KEY_KP2".to_string(), "KEY_KP3".to_string(), "KEY_KPENTER".to_string() ],
                        vec![ "KEY_KP0".to_string(), "KEY_KP0".to_string(), "KEY_KPDOT".to_string(), "KEY_KPENTER".to_string() ]
                    ],
                    brightness_levels: BrightnessLevels { low: 1, med:  24, high: 31 }
                }
            ]
        };

        let json = serde_json::to_string_pretty(&cfg)?;
        std::fs::write(file, json)?;
        Ok(())
    }
}
