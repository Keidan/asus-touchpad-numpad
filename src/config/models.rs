use serde::{Deserialize, Serialize};

/// Device identification parameters used when detecting the numpad interface.
///
/// Controls how many times the application attempts to locate the input
/// device and how long it waits between retries.
#[derive(Clone, Deserialize, Serialize)]
pub struct Identify {
    /// Maximum number of attempts to identify the device interface before giving up.
    pub max_attempts: u32,
    /// Delay in milliseconds between consecutive identification attempts on failure.
    pub retry_delay_ms: u32,
}

/// Represents the three backlight brightness levels of the numpad.
///
/// Each field holds a raw value written directly to the brightness control
/// interface of the device. The meaning of the values is hardware-dependent;
/// higher is not necessarily brighter — refer to the device's brightness
/// register documentation.
///
/// Typically populated from the `brightness_levels` field of a [`Layout`],
/// where values are ordered as `[high, med, low]`.
#[derive(Clone, Deserialize, Serialize)]
pub struct BrightnessLevels {
    // Low brightness level.
    pub low: u8,
    /// Medium brightness level.
    pub med: u8,
    /// High brightness level.
    pub high: u8,
}

/// Defines a rectangular touch zone on the touchpad, expressed as normalized
/// coordinate ratios of the touchpad's physical dimensions (`[0.0, 1.0]`).
///
/// A touch event falls within the zone if **all** of the following hold:
/// - `x >= x_min` — touch is to the right of the left edge.
/// - `x <= x_max` — touch is to the left of the right edge.
/// - `y >= y_min` — touch is below the top edge.
/// - `y <= y_max` — touch is above the bottom edge.
///
/// Unused edges can be set to `0.0` (for `x_min` / `y_min`) or `1.0`
/// (for `x_max` / `y_max`) to make them non-restrictive.
///
/// # Calibration
///
/// To calibrate these values for a specific model:
///
/// 1. Enable debug logging.
/// 2. Tap repeatedly across the entire target button area.
/// 3. Note the min/max `x` and `y` values from the logs:
///    `"finger down at x <X> y <Y>"`.
/// 4. Divide by the touchpad max values logged at startup:
///    `"Touchpad min/max: x [0:<MAXX>], y [0:<MAXY>]"`.
/// 5. Apply a small margin (e.g. `±0.02`) to each boundary to account for
///    calibration variance.
///
/// **Example:** `x = 3243`, `maxx = 4036` → `3243 / 4036 ≈ 0.803` → `x_min = 0.80`
#[derive(Clone, Deserialize, Serialize)]
pub struct Zone {
    /// Minimum X ratio (left edge). A touch qualifies if its normalized X is `>= x_min`.
    pub x_min: f32,
    /// Maximum X ratio (right edge). A touch qualifies if its normalized X is `<= x_max`.
    pub x_max: f32,
    /// Minimum Y ratio (top edge). A touch qualifies if its normalized Y is `>= y_min`.
    pub y_min: f32,
    /// Maximum Y ratio (bottom edge). A touch qualifies if its normalized Y is `<= y_max`.
    pub y_max: f32,
}

/// Groups all special touch zones defined on the touchpad surface.
///
/// Each zone corresponds to a dedicated button area outside the main key grid.
/// A touch event is checked against these zones before being interpreted as a
/// numpad key press.
#[derive(Clone, Deserialize, Serialize)]
pub struct Zones {
    /// Touch zone for the Num Lock button (top-right corner of the touchpad).
    pub numlock: Zone,
    /// Touch zone for the brightness/calculator button (top-left corner of the touchpad).
    pub brightness_calculator: Zone,
}

/// Describes the physical layout and key mapping of a numpad touchpad.
///
/// A `Layout` defines the grid dimensions of the numpad area, the fraction
/// of the touchpad reserved above it, and a 2D grid of key names that maps
/// each cell to an entry in the keymap.
///
/// The `keymap` field is a row-major grid of `rows x cols` key name strings.
/// Each name must match a `name` value in the loaded keymap.
///
/// `brightness_levels` is optional and omitted from serialized JSON when
/// absent (`#[serde(skip_serializing_if = "Option::is_none")]`).
#[derive(Clone, Deserialize, Serialize)]
pub struct Layout {
    /// Unique name identifying this layout (e.g. `"ux433fa"`, `"gx701"`).
    pub name: String,
    /// Number of columns in the key grid.
    pub cols: u8,
    /// Number of rows in the key grid.
    pub rows: u8,
    /// Touch zones for the special buttons outside the main key grid
    /// (Num Lock, brightness/calculator and coactivator).
    pub zones: Zones,
    /// Delay in milliseconds for double-tap activation (0 = disabled)
    pub double_tap_delay_ms: u32,
    /// Whether the calculator can be launched from the top-left touchpad zone.
    ///
    /// When `true` and Num Lock is inactive, tapping the
    /// [`Zone`] launches the system calculator.
    /// When `false`, that zone only controls brightness (Num Lock on)
    /// or does nothing (Num Lock off).
    pub allow_calculator: bool,
    /// Row offset subtracted when computing the key row from a touch Y coordinate.
    /// A positive value shifts the effective grid downward, effectively
    /// reserving space at the top of the touchpad (e.g. for a logo or gesture area).
    pub top_offset: f32,
    /// Row-major grid of key names (`rows × cols`). Each entry must be a valid
    /// [`evdev::KeyCode`] variant name (e.g. `"KEY_KP0"`, `"KEY_BACKSPACE"`),
    /// as names are resolved via `evdev::KeyCode::from_str` at validation time.
    /// A key name may appear more than once (e.g. a wide `0` key spanning two cells).
    pub keymap: Vec<Vec<String>>,
    /// Supported backlight brightness steps.
    pub brightness_levels: BrightnessLevels,
}

/// Top-level application configuration.
///
/// Loaded from the main JSON configuration file. Specifies the log verbosity,
/// the active layout, and the full list of known layouts.
///
/// The `layout_name` field must match the `name` of one entry in `layouts` ;
/// this is enforced at runtime by [`Config::validate`].
/// `layout_name` can have the value `auto`,
/// in which case detection will be performed based on the file
/// `/sys/class/dmi/id/board_name` (the [`Config::validate`] constraint will also be applied).
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    /// Minimum log level to emit. Recognized values: `"debug"`, `"info"`,
    /// `"warn"`, `"error"`. Defaults to `"info"` for unrecognized values.
    pub log_level: String,
    /// Device identification parameters for the layouts.
    pub identify: Identify,
    /// Name of the layout to activate on startup. Must match a `name` field
    /// in `layouts`.
    pub layout_name: String,
    /// Full list of available numpad layouts.
    pub layouts: Vec<Layout>,
}
