mod config;
mod logger;
mod sys;

use config::defaults::DefaultConfig;
use config::models::{Config, Identify, Layout};
use config::parser::Parser;
use evdev::KeyCode;
use logger::Logger;
use std::thread;
use std::time::Duration;
use sys::proc::{Proc, ProcDevice};
use sys::touchpad::Touchpad;
use sys::utils::Utils;

const NAME: &str = "asus-touchpad-numpad";
const CONFIG_FILE: &str = "config.json";

/// Loads, validates, and resolves the application configuration.
///
/// This function orchestrates the full startup configuration pipeline:
///
/// 1. Parses the JSON config file into a [`Config`] struct.
/// 2. Initializes the global [`Logger`] using the `log_level` from the config.
/// 3. Validates the config and resolves the active [`Layout`] by `layout_name`.
///
/// # Arguments
///
/// * `file_config` - Path to the JSON configuration file.
///
/// # Returns
///
/// A tuple `(Identify, Layout, Vec<Vec<KeyCode>>)` containing:
/// - identification behavior (maximum number of attempts and timeouts in case of failure).
/// - The active layout selected by `config.layout_name`.
/// - The resolved `rows × cols` key grid for that layout.
///
/// # Exits
///
/// Calls `std::process::exit(1)` if any of the following occur:
/// - The config file cannot be parsed.
/// - `config.layout_name` does not match any layout in `config.layouts`.
/// - The active layout fails validation against the keymap (unknown key name,
///   wrong row/column count).
fn load_configuration(file_config: &str) -> (Layout, Identify, Vec<Vec<KeyCode>>) {
    let parser = Parser::new();
    let config: Config;
    let layout: Layout;
    let identify: Identify;
    let keys: Vec<Vec<KeyCode>>;

    match parser.parse_config(&file_config) {
        Ok(cfg) => {
            config = cfg.clone();
            identify = config.identify.clone();
            Logger::init(NAME, Logger::string_to_level(&config.log_level.to_string()));
            log_debug!("Configuration loaded.");
        }
        Err(e) => {
            Logger::init(NAME, Logger::string_to_level("error"));
            log_fatal!("Error parsing the configuration file: {}", e);
        }
    }
    match config.validate() {
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
        Ok(lay) => {
            layout = lay.clone();
            log_info!("The '{}' layout was found", lay.name);
        }
    }

    match layout.validate() {
        Ok(resolved_keys) => keys = resolved_keys,
        Err(e) => {
            log_fatal!("Invalid layout: {}", e);
        }
    }
    (layout, identify, keys)
}

/// Detects the touchpad input device by scanning `/proc/bus/input/devices`.
///
/// Repeatedly parses the proc file looking for:
/// - A **touchpad** whose name starts with `"ASUE"` or `"ELAN"` and contains `"Touchpad"`.
///
/// The number of attempts and the delay between retries are controlled by
/// `layout.identify.max_attempts` and `layout.identify.retry_delay_ms`.
/// As soon as both devices are found, the function returns immediately without
/// waiting for the remaining attempts.
///
/// # Arguments
///
/// * `identify` - Define the identification sed to read `identify.max_attempts` and
///   `identify.retry_delay_ms`.
/// # Returns
///
/// `Option<ProcDevice>` where the element is the touchpad.
///
/// In practice this function either returns `(Some(_), Some(_))` or exits the
/// process — the `(None, None)` case is unreachable but required to satisfy
/// the compiler (the while loop is bounded by `max_attempts`).
///
/// # Exits
///
/// Calls `std::process::exit(1)` if any of the following occur:
/// - `/proc/bus/input/devices` returns an empty device list.
/// - After all attempts, the touchpad could not be found.
/// - After all attempts, the touchpad was found but its bus ID could not be resolved.
fn detect_devices(identify: &Identify) -> Option<ProcDevice> {
    // Parse /proc to find the touchpad input device.
    // Exit if no device is found, as there is nothing to handle.
    let pr = Proc::new();

    let mut attempt: u32 = identify.max_attempts;

    while attempt > 0 {
        let mut touchpad: Option<ProcDevice> = None;
        let devices = pr.parse();
        if devices.is_empty() {
            log_fatal!("Unable to parse file: {}", pr.get_path());
        }
        for device in &devices {
            if (device.name.starts_with("ASUE") || device.name.starts_with("ELAN"))
                && device.name.contains("Touchpad")
            {
                log_info!(
                    "Touchpad detected: '{}', bus: '{}', event: {}",
                    device.name,
                    device.bus,
                    device.event
                );
                touchpad = Some(device.clone());
            }
            if touchpad.is_some() {
                return touchpad;
            }
        }
        attempt -= 1;
        if 0 == attempt {
            if touchpad.is_none() {
                log_fatal!("Can't find the touchpad");
            }
            if touchpad.as_ref().map_or(true, |d| d.bus == -1) {
                log_fatal!("Can't find the bud ID");
            }
        }

        thread::sleep(Duration::from_millis(identify.retry_delay_ms.into()));
    }
    None
}

/// Main entry point of the application.
///
/// This function orchestrates the initialization and execution of the touchpad driver:
///
/// 1. **Configuration Setup**: Determines the configuration file path. If the file does not exist,
///    a default configuration is generated. The program terminates immediately if this step fails.
/// 2. **Configuration Loading**: Parses the configuration file to retrieve the keyboard layout,
///    device identification rules, and key mappings.
/// 3. **Device Detection**: Scans `/proc` (or the system's input device list) to locate the specific
///    touchpad device based on the identification rules.
/// 4. **Initialization**: Instantiates the [`crate::sys::touchpad::Touchpad`] struct with the detected device information.
/// 5. **Execution**: Opens the device, installs the key hooks, and starts the main event processing loop.
///
/// # Panics / Exit
///
/// - Exits with code `1` if the default configuration cannot be written when missing.
/// - Terminates via [`crate::log_fatal`] if the touchpad cannot be opened, if key hooks cannot be installed,
///   or if the event processing loop encounters a fatal error.
///
fn main() {
    // Build the config file path from the binary name (e.g. "config.json")
    let file_config = Utils::get_file_location(CONFIG_FILE);
    // If the config file does not exist, generate a default one.
    // Exit on failure since the program cannot run without a valid config.
    if !Utils::file_exists(&file_config) {
        DefaultConfig::write(&file_config).unwrap_or_else(|e| {
            eprintln!("Error writing the default config: {}", e);
            std::process::exit(1);
        });
    }
    // Load and parse config file.
    let (layout, identify, keys) = load_configuration(&file_config);
    // Parse /proc to find the touchpad input device.
    let proc_touchpad = detect_devices(&identify);

    let pt = proc_touchpad.unwrap();
    let mut touchpad: Touchpad = Touchpad::new(pt.bus, &pt.event, &layout);
    // Open the touchpad device file.

    if let Err(e) = touchpad.open() {
        log_fatal!("{}", e);
    }
    // Install the key mapping and hooks.

    if let Err(e) = touchpad.install(&keys) {
        log_fatal!("{}", e);
    }
    // Start the main event processing loop.

    if let Err(e) = touchpad.process_events(&keys) {
        log_fatal!("{}", e);
    }
}
