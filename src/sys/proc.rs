use regex::Regex;

/// Represents a single input device entry parsed from `/proc/bus/input/devices`.
///
/// Each field corresponds to a line prefix in the proc file:
///
/// | Field      | Prefix | Example value          |
/// |------------|--------|------------------------|
/// | `name`     | `N:`   | `"ASUS NumberPad"`     |
/// | `event`    | `H:`   | `eventN`               |
/// | `bus`      | `S:`   | `"3"`                  |
#[derive(Clone)]
pub struct ProcDevice {
    /// Bus identifier extracted from the `I:` line (e.g. `"3"` for USB).
    pub bus: i32,
    /// Human-readable device name extracted from the `N:` line, quotes stripped.
    pub name: String,
    /// Isolates the first event-type handler.
    pub event: String,
}

/// Reader and parser for the Linux `/proc/bus/input/devices` file.
///
/// `Proc` provides access to the list of input devices currently registered
/// with the kernel. It reads and parses the raw proc file format into a
/// collection of [`ProcDevice`] entries.
pub struct Proc {
    /// Path to the proc file. Defaults to `/proc/bus/input/devices`.
    path: String,
}
impl Proc {
    /// Creates a new `Proc` instance pointing to `/proc/bus/input/devices`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let proc = Proc::new();
    /// ```
    pub fn new() -> Self {
        Proc {
            path: "/proc/bus/input/devices".to_string(),
        }
    }

    /// Returns the path to the proc file used by this instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let proc = Proc::new();
    /// assert_eq!(proc.get_path(), "/proc/bus/input/devices");
    /// ```
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    /// Parses `/proc/bus/input/devices` and returns all recognized input devices.
    ///
    /// The proc file is made up of blank-line-separated blocks, one per device.
    /// Each block is parsed line by line according to its single-letter prefix:
    ///
    /// - `N:` — device name; surrounding quotes are stripped.
    /// - `S:` — sysfs path.
    /// - `H:` — space-separated list of event handlers.
    ///
    /// Blocks with an empty `name` field are silently skipped. All other blocks
    /// produce a `Some(ProcDevice)` entry in the returned vector.
    ///
    /// # Returns
    ///
    /// A `Vec<ProcDevice>` where each `Some` entry is a successfully
    /// parsed device.
    ///
    /// # Panics
    ///
    /// Panics if `/proc/bus/input/devices` cannot be read (e.g. on non-Linux
    /// systems or if the process lacks read permission).
    ///
    /// # Example
    ///
    /// ```rust
    /// let proc = Proc::new();
    /// for device in proc.parse().into_iter().flatten() {
    ///     println!("{} -> {:?}", device.name, device.handlers);
    /// }
    /// ```
    pub fn parse(&self) -> Vec<ProcDevice> {
        let re = Regex::new(r".*i2c-(\d+)/.*").unwrap();
        let content = std::fs::read_to_string("/proc/bus/input/devices").unwrap();
        let mut devices = Vec::new();

        for block in content.split("\n\n") {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }

            let mut bus = -1;
            let mut name = String::new();
            let mut event = String::new();

            for line in block.lines() {
                match line.splitn(2, ": ").collect::<Vec<_>>().as_slice() {
                    ["S", rest] => {
                        for part in rest.split_whitespace() {
                            if let Some(val) = part.strip_prefix("Sysfs=") {
                                let var = val.to_string();
                                let bus_id = re
                                    .captures(&var)
                                    .and_then(|c| c.get(1))
                                    .map(|m| m.as_str())
                                    .unwrap_or(&var);
                                match bus_id.parse::<i32>() {
                                    Ok(num) => {
                                        bus = num;
                                    }
                                    Err(_) => {
                                        bus = -1;
                                    }
                                }
                            }
                        }
                    }
                    ["N", rest] => {
                        name = rest
                            .strip_prefix("Name=")
                            .unwrap_or(rest)
                            .trim_matches('"')
                            .to_string();
                    }
                    ["H", rest] => {
                        if let Some(found) =
                            rest.split_whitespace().find(|s| s.starts_with("event"))
                        {
                            event = found.to_string();
                        }
                    }
                    _ => {}
                }
            }

            if !name.is_empty() {
                devices.push(ProcDevice { bus, name, event });
            }
        }

        devices
    }
}
