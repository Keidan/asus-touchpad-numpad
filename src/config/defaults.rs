use crate::config::models::{BrightnessLevels, Config, Identify, Layout, Zone, Zones};

/// Expands a list of string literals into a `Vec<String>`.
///
/// # Example
///
/// ```rust
/// let row = keys!["KEY_KP1", "KEY_KP2", "KEY_KP3"];
/// assert_eq!(row, vec!["KEY_KP1".to_string(), "KEY_KP2".to_string(), "KEY_KP3".to_string()]);
/// ```
macro_rules! keys {
    [$($k:expr),*] => {
        vec![$($k.to_string()),*]
    };
}

/// Generates and writes the default application configuration.
///
/// `DefaultConfig` is a stateless helper that builds a ready-to-use [`Config`]
/// populated with known ASUS laptop numpad layouts and serializes it as a
/// pretty-printed JSON file.
///
/// # Bundled layouts
///
/// | Name      | Model                   | Grid    |
/// |-----------|-------------------------|---------|
/// | `g533qr`  | ASUS ROG Strix SCAR 15  | 5 × 4   |
/// | `ux433fa` | ASUS ZenBook 14         | 5 × 4   |
/// | `gx701`   | ASUS ROG Zephyrus S17   | 4 × 5   |
pub struct DefaultConfig;

impl DefaultConfig {
    /// Writes the default configuration to a JSON file at the given path.
    ///
    /// The generated config sets `log_level` to `"info"` and `layout_name` to
    /// `"auto"`, which triggers automatic layout selection based on the board
    /// name read from `/sys/class/dmi/id/board_name` at runtime.
    ///
    /// # Arguments
    ///
    /// * `file` — Destination path. The file is created if it does not exist,
    ///   or overwritten if it does.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails or if the file cannot be
    /// written (e.g. missing directory, permission denied).
    ///
    /// # Example
    ///
    /// ```rust
    /// DefaultConfig::write("/etc/numpad/config.json")
    ///     .expect("failed to write default config");
    /// ```
    pub fn write(file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = Config {
            log_level: "info".to_string(),
            layout_name: "auto".to_string(),
            identify: Identify {
                max_attempts: 5,
                retry_delay_ms: 100,
            },
            layouts: vec![
                Self::layout_g533qr(),
                Self::layout_ux433fa(),
                Self::layout_gx701(),
            ],
        };

        let json = serde_json::to_string_pretty(&cfg)?;
        std::fs::write(file, json)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Shared defaults
    // -------------------------------------------------------------------------

    /// Returns the brightness levels used by every bundled layout.
    ///
    /// | Level  | Value |
    /// |--------|-------|
    /// | low    | 1     |
    /// | med    | 24    |
    /// | high   | 31    |
    fn default_brightness_levels() -> BrightnessLevels {
        BrightnessLevels {
            low: 1,
            med: 24,
            high: 31,
        }
    }

    /// Returns the touch zone used to cycle brightness on every bundled layout.
    ///
    /// The zone covers the top-left corner of the touchpad
    /// (`x ∈ [0.0, 0.06]`, `y ∈ [0.0, 0.07]`).
    fn default_brightness_calculator_zone() -> Zone {
        Zone {
            x_min: 0.0,
            x_max: 0.06,
            y_min: 0.0,
            y_max: 0.07,
        }
    }

    /// Returns the numlock toggle zone shared by `ux433fa` and `gx701`.
    ///
    /// The zone covers the top-right corner of the touchpad
    /// (`x ∈ [0.95, 1.0]`, `y ∈ [0.0, 0.09]`).
    fn common_numlock_zone() -> Zone {
        Zone {
            x_min: 0.95,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 0.09,
        }
    }

    /// Returns the 5 × 4 keymap shared by the `g533qr` and `ux433fa` layouts.
    ///
    /// Layout (row-major order):
    ///
    /// ```text
    /// ┌───────┬───────┬───────┬────────────┬───────────┐
    /// │  KP7  │  KP8  │  KP9  │  KPSLASH   │ BACKSPACE │
    /// ├───────┼───────┼───────┼────────────┼───────────┤
    /// │  KP4  │  KP5  │  KP6  │ KPASTERISK │ BACKSPACE │
    /// ├───────┼───────┼───────┼────────────┼───────────┤
    /// │  KP1  │  KP2  │  KP3  │  KPMINUS   │  KPENTER  │
    /// ├───────┼───────┼───────┼────────────┼───────────┤
    /// │  KP0  │  KP0  │ KPDOT │   KPPLUS   │  KPENTER  │
    /// └───────┴───────┴───────┴────────────┴───────────┘
    /// ```
    fn keymap_5x4() -> Vec<Vec<String>> {
        vec![
            keys![
                "KEY_KP7",
                "KEY_KP8",
                "KEY_KP9",
                "KEY_KPSLASH",
                "KEY_BACKSPACE"
            ],
            keys![
                "KEY_KP4",
                "KEY_KP5",
                "KEY_KP6",
                "KEY_KPASTERISK",
                "KEY_BACKSPACE"
            ],
            keys![
                "KEY_KP1",
                "KEY_KP2",
                "KEY_KP3",
                "KEY_KPMINUS",
                "KEY_KPENTER"
            ],
            keys![
                "KEY_KP0",
                "KEY_KP0",
                "KEY_KPDOT",
                "KEY_KPPLUS",
                "KEY_KPENTER"
            ],
        ]
    }

    // -------------------------------------------------------------------------
    // Layouts
    // -------------------------------------------------------------------------

    /// Returns the layout for the **ASUS ROG Strix SCAR 15** (`g533qr`).
    ///
    /// Differences from the common defaults:
    /// - Numlock zone spans the wider area `x ∈ [0.80, 1.0]`, `y ∈ [0.0, 0.24]`.
    /// - `top_offset` is `0.10` to account for the model's dead strip.
    fn layout_g533qr() -> Layout {
        Layout {
            name: "g533qr".to_string(),
            cols: 5,
            rows: 4,
            zones: Zones {
                numlock: Zone {
                    x_min: 0.80,
                    x_max: 1.0,
                    y_min: 0.0,
                    y_max: 0.24,
                },
                brightness_calculator: Self::default_brightness_calculator_zone(),
            },
            double_tap_delay_ms: 250,
            allow_calculator: true,
            top_offset: 0.10,
            keymap: Self::keymap_5x4(),
            brightness_levels: Self::default_brightness_levels(),
        }
    }

    /// Returns the layout for the **ASUS ZenBook 14** (`ux433fa`).
    ///
    /// Uses [`Self::common_numlock_zone`] and [`Self::keymap_5x4`].
    fn layout_ux433fa() -> Layout {
        Layout {
            name: "ux433fa".to_string(),
            cols: 5,
            rows: 4,
            zones: Zones {
                numlock: Self::common_numlock_zone(),
                brightness_calculator: Self::default_brightness_calculator_zone(),
            },
            double_tap_delay_ms: 250,
            allow_calculator: true,
            top_offset: 0.10,
            keymap: Self::keymap_5x4(),
            brightness_levels: Self::default_brightness_levels(),
        }
    }

    /// Returns the layout for the **ASUS ROG Zephyrus S17** (`gx701`).
    ///
    /// Differences from the common defaults:
    /// - Grid is **4 × 5** (portrait orientation) instead of 5 × 4.
    /// - `top_offset` is `0.0` — no dead strip on this model.
    /// - The first row exposes a dedicated `KEY_CALC` key.
    ///
    /// Layout (row-major order):
    ///
    /// ```text
    /// ┌────────┬──────────┬─────────────┬──────────┐
    /// │  CALC  │ KPSLASH  │ KPASTERISK  │ KPMINUS  │
    /// ├────────┼──────────┼─────────────┼──────────┤
    /// │  KP7   │   KP8    │     KP9     │  KPPLUS  │
    /// ├────────┼──────────┼─────────────┼──────────┤
    /// │  KP4   │   KP5    │     KP6     │  KPPLUS  │
    /// ├────────┼──────────┼─────────────┼──────────┤
    /// │  KP1   │   KP2    │     KP3     │ KPENTER  │
    /// ├────────┼──────────┼─────────────┼──────────┤
    /// │  KP0   │   KP0    │    KPDOT    │ KPENTER  │
    /// └────────┴──────────┴─────────────┴──────────┘
    /// ```
    fn layout_gx701() -> Layout {
        Layout {
            name: "gx701".to_string(),
            cols: 4,
            rows: 5,
            zones: Zones {
                numlock: Self::common_numlock_zone(),
                brightness_calculator: Self::default_brightness_calculator_zone(),
            },
            double_tap_delay_ms: 250,
            allow_calculator: true,
            top_offset: 0.0,
            keymap: vec![
                keys!["KEY_CALC", "KEY_KPSLASH", "KEY_KPASTERISK", "KEY_KPMINUS"],
                keys!["KEY_KP7", "KEY_KP8", "KEY_KP9", "KEY_KPPLUS"],
                keys!["KEY_KP4", "KEY_KP5", "KEY_KP6", "KEY_KPPLUS"],
                keys!["KEY_KP1", "KEY_KP2", "KEY_KP3", "KEY_KPENTER"],
                keys!["KEY_KP0", "KEY_KP0", "KEY_KPDOT", "KEY_KPENTER"],
            ],
            brightness_levels: Self::default_brightness_levels(),
        }
    }
}
