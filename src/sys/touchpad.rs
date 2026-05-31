use crate::config::models::Layout;
use crate::log_debug;
use crate::sys::keyboard::KeyboardLayout;
use evdev::uinput::VirtualDevice;
use evdev::{AbsoluteAxisCode, Device, EventType, InputEvent, KeyCode, SynchronizationCode};
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;

/// I2C slave address used to communicate with the touchpad/numpad controller.
const I2C_SLAVE_ADDRESS: u16 = 0x15;

/// Represents the backlight brightness level of the numpad overlay.
///
/// This enum is used to track and cycle through the different brightness
/// states that the numpad illumination supports.
#[derive(Debug, PartialEq)]
pub enum Brightness {
    /// Backlight is turned off (numlock disabled or initial state).
    Off,
    /// Low brightness level.
    Low,
    /// Medium brightness level.
    Medium,
    /// High brightness level.
    High,
}
/// Implements the [`Display`](std::fmt::Display) trait for [`crate::sys::touchpad::Brightness`].
///
/// Formats the brightness level as a human-readable string.
///
/// # Variants
///
/// | Variant | Output |
/// |---------|--------|
/// | `Brightness::Off` | `"Off"` |
/// | `Brightness::Low` | `"Low"` |
/// | `Brightness::Medium` | `"Medium"` |
/// | `Brightness::High` | `"High"` |
///
/// # Examples
///
/// ```rust
/// let b = Brightness::High;
/// assert_eq!(format!("{}", b), "High");
///
/// println!("Current brightness: {}", Brightness::Medium); // "Current brightness: Medium"
/// ```
impl std::fmt::Display for Brightness {
    /// Writes the string representation of the brightness level into the formatter.
    ///
    /// # Arguments
    ///
    /// * `f` - The [`std::fmt::Formatter`] used to write the output.
    ///
    /// # Returns
    ///
    /// Returns [`std::fmt::Result`] indicating whether the formatting succeeded.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Brightness::Off => write!(f, "Off"),
            Brightness::Low => write!(f, "Low"),
            Brightness::Medium => write!(f, "Medium"),
            Brightness::High => write!(f, "High"),
        }
    }
}

/// Represents the physical touchpad device, which also acts as a virtual numpad.
///
/// This struct encapsulates all state required to open, configure, and process
/// events from an Asus touchpad that supports a software numpad overlay.
/// It manages both the raw input device and a virtual `uinput` device used
/// to emit synthetic key events to the system.
pub struct Touchpad {
    /// Whether the underlying device has been successfully opened.
    opened: bool,
    /// Whether the underlying device has been successfully registered.
    registered: bool,
    /// I2C bus number used for brightness/numlock control commands.
    bus: i32,
    /// Name of the input event device (e.g. `"event5"`).
    event: String,
    /// Maximum absolute X coordinate reported by the touchpad.
    max_x: i32,
    /// Maximum absolute Y coordinate reported by the touchpad.
    max_y: i32,
    /// The key code used to emit the `%` (percentage) character,
    /// which differs between QWERTY and AZERTY keyboard layouts.
    percentage_key: KeyCode,
    /// The opened evdev input device.
    device: Option<Device>,
    /// The virtual uinput device used to emit synthetic key events.
    udevice: Option<VirtualDevice>,
    /// Layout configuration (zones, brightness levels, grid size, etc.).
    layout: Layout,
}

impl Touchpad {
    /// Creates a new `Touchpad` instance without opening the device.
    ///
    /// # Arguments
    ///
    /// * `bus` - The I2C bus number (e.g. `3` for `/dev/i2c-3`).
    /// * `event` - The input event name (e.g. `"event5"` for `/dev/input/event5`).
    /// * `layout` - A reference to the layout configuration to use.
    ///
    /// # Example
    ///
    /// ```rust
    /// let touchpad = Touchpad::new(3, "event5", &layout);
    /// ```
    pub fn new(bus: i32, event: &str, layout: &Layout) -> Self {
        Touchpad {
            opened: false,
            registered: false,
            bus,
            event: event.to_string(),
            max_x: 0,
            max_y: 0,
            percentage_key: KeyCode::KEY_5,
            device: None,
            udevice: None,
            layout: layout.clone(),
        }
    }

    /// Opens the touchpad input device and reads its absolute axis capabilities.
    ///
    /// This method must be called before `install` or `process_events`.
    /// It detects the keyboard layout to determine the correct percentage key,
    /// opens the evdev device at `/dev/input/<event>`, and reads the min/max
    /// values for the X and Y absolute axes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The keyboard layout is unrecognized.
    /// - The input device cannot be opened.
    /// - The absolute axis state cannot be read.
    pub fn open(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match KeyboardLayout::detect() {
            KeyboardLayout::Qwerty => {
                self.percentage_key = KeyCode::KEY_5;
            }
            KeyboardLayout::Azerty => {
                self.percentage_key = KeyCode::KEY_APOSTROPHE;
            }
            KeyboardLayout::Unknown(k) => {
                return Err(format!("Unspecified keyboad layout: '{}'", k).into());
            }
        }

        self.device = Some(Device::open(format!("/dev/input/{}", self.event))?);
        let device = self.device.as_mut().unwrap();

        let abs = device.get_abs_state()?;

        let abs_x = abs[AbsoluteAxisCode::ABS_X.0 as usize];
        let abs_y = abs[AbsoluteAxisCode::ABS_Y.0 as usize];

        self.max_x = abs_x.maximum;
        self.max_y = abs_y.maximum;
        log_debug!(
            "Touchpad min/max: x [{}:{}], y [{}:{}]",
            abs_x.minimum,
            self.max_x,
            abs_y.minimum,
            self.max_y
        );
        self.opened = true;
        Ok(())
    }

    /// Creates and registers the virtual `uinput` device with the required key capabilities.
    ///
    /// This method builds a [`evdev::uinput::VirtualDevice`] named `"Asus Touchpad/Numpad"` and registers
    /// all keys that may be emitted: the entire numpad key grid, numlock, calculator,
    /// left shift, and (if needed) the layout-specific percentage key.
    ///
    /// `open` must be called successfully before calling this method.
    ///
    /// # Arguments
    ///
    /// * `keys_layout` - A 2D grid of [`evdev::KeyCode`] values representing the numpad layout,
    ///   where `keys_layout[row][col]` is the key at that position.
    ///
    /// # Errors
    ///
    /// Returns an error if the touchpad has not been opened, or if the virtual
    /// device cannot be created.
    pub fn install(
        &mut self,
        keys_layout: &Vec<Vec<KeyCode>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.opened {
            return Err(format!("The touchpad is not open!").into());
        }
        let mut keys = evdev::AttributeSet::<KeyCode>::new();
        keys.insert(KeyCode::KEY_LEFTSHIFT);
        keys.insert(KeyCode::KEY_NUMLOCK);
        keys.insert(KeyCode::KEY_CALC);

        for col in keys_layout {
            for key in col {
                keys.insert(*key);
            }
        }

        if self.percentage_key != KeyCode::KEY_5 {
            keys.insert(self.percentage_key);
        }

        self.udevice = Some(
            VirtualDevice::builder()?
                .name("Asus Touchpad/Numpad")
                .with_keys(&keys)?
                .build()?,
        );
        self.registered = true;
        Ok(())
    }

    /// Emits a numlock key-down event, grabs exclusive access to the device,
    /// and sends an I2C command to turn on the numpad backlight at the given brightness.
    ///
    /// # Arguments
    ///
    /// * `brightness` - The desired brightness level for the numpad backlight.
    ///
    /// # Errors
    ///
    /// Returns an error if the key event, device grab, or I2C write fails.
    fn activate_numlock(
        &mut self,
        brightness: &Brightness,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bright_val: u8 = self.brightness_to_u8(brightness);

        self.udevice.as_mut().unwrap().emit(&[
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_NUMLOCK.0, 1),
            InputEvent::new(
                EventType::SYNCHRONIZATION.0,
                SynchronizationCode::SYN_REPORT.0,
                0,
            ),
        ])?;

        self.device.as_mut().unwrap().grab()?;

        self.write_i2c(&[
            0x05, 0x00, 0x3d, 0x03, 0x06, 0x00, 0x07, 0x00, 0x0d, 0x14, 0x03, bright_val, 0xad,
        ])?;

        Ok(())
    }

    /// Emits a numlock key-up event, releases the exclusive device grab,
    /// and sends an I2C command to turn off the numpad backlight.
    ///
    /// # Errors
    ///
    /// Returns an error if the key event, device ungrab, or I2C write fails.
    fn deactivate_numlock(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.udevice.as_mut().unwrap().emit(&[
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_NUMLOCK.0, 0),
            InputEvent::new(
                EventType::SYNCHRONIZATION.0,
                SynchronizationCode::SYN_REPORT.0,
                0,
            ),
        ])?;

        self.device.as_mut().unwrap().ungrab()?;

        // i2c direct
        self.write_i2c(&[
            0x05, 0x00, 0x3d, 0x03, 0x06, 0x00, 0x07, 0x00, 0x0d, 0x14, 0x03, 0x00, 0xad,
        ])?;

        Ok(())
    }

    /// Emits a calculator key press-and-release event via the virtual device,
    /// which causes the system to open the default calculator application.
    ///
    /// # Errors
    ///
    /// Returns an error if the key events cannot be emitted.
    fn launch_calculator(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.udevice.as_mut().unwrap().emit(&[
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_CALC.0, 1),
            InputEvent::new(
                EventType::SYNCHRONIZATION.0,
                SynchronizationCode::SYN_REPORT.0,
                0,
            ),
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_CALC.0, 0),
            InputEvent::new(
                EventType::SYNCHRONIZATION.0,
                SynchronizationCode::SYN_REPORT.0,
                0,
            ),
        ])?;
        Ok(())
    }

    /// Cycles the backlight brightness to the next level and sends the corresponding
    /// I2C command to apply it.
    ///
    /// The cycle order is: `Off` | `High` → `Low` → `Medium` → `High`.
    /// The `brightness` argument is mutated in place.
    ///
    /// # Arguments
    ///
    /// * `brightness` - A mutable reference to the current [`Brightness`] level,
    ///   which will be updated to the next level in the cycle.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C write fails.
    fn change_brightness(
        &self,
        brightness: &mut Brightness,
    ) -> Result<(), Box<dyn std::error::Error>> {
        *brightness = match brightness {
            Brightness::Off | Brightness::High => Brightness::Low,
            Brightness::Low => Brightness::Medium,
            Brightness::Medium => Brightness::High,
        };
        let bright_val: u8 = self.brightness_to_u8(brightness);

        self.write_i2c(&[
            0x05, 0x00, 0x3d, 0x03, 0x06, 0x00, 0x07, 0x00, 0x0d, 0x14, 0x03, bright_val, 0xad,
        ])?;
        Ok(())
    }

    /// Converts a [`Brightness`] variant into the corresponding raw `u8` value
    /// as defined in the layout configuration.
    ///
    /// Returns `0` for [`Brightness::Off`].
    fn brightness_to_u8(&self, brightness: &Brightness) -> u8 {
        log_debug!("New touchpad brightness: {}", brightness);
        if Brightness::Low == *brightness {
            return self.layout.brightness_levels.low;
        } else if Brightness::Medium == *brightness {
            return self.layout.brightness_levels.med;
        } else if Brightness::High == *brightness {
            return self.layout.brightness_levels.high;
        } else {
            return 0;
        }
    }

    /// Writes a raw byte slice to the touchpad controller over I2C.
    ///
    /// Opens `/dev/i2c-<bus>` using a forced (non-exclusive) connection to
    /// [`I2C_SLAVE_ADDRESS`] and writes `data` directly.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw byte payload to send (protocol-specific command bytes).
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C device cannot be opened or the write fails.
    ///
    /// # Safety
    ///
    /// Uses `LinuxI2CDevice::force_new`, which bypasses kernel driver binding checks.
    /// This is required because the touchpad driver holds the device but does not
    /// expose brightness control through a standard interface.
    fn write_i2c(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut dev = unsafe {
            LinuxI2CDevice::force_new(format!("/dev/i2c-{}", self.bus), I2C_SLAVE_ADDRESS)?
        };
        dev.write(data)?;
        Ok(())
    }

    /// Main event loop: reads touchpad input events and dispatches actions.
    ///
    /// This blocking function continuously polls the touchpad device for finger
    /// touch events and translates them into one of the following actions:
    ///
    /// - **Numlock toggle**: tapping the top-right corner enables or disables the
    ///   numpad overlay, grabbing/releasing the device and toggling the backlight.
    /// - **Brightness / Calculator**: tapping the top-left corner cycles the
    ///   backlight brightness (when numlock is on) or launches the calculator
    ///   (when numlock is off, if allowed by the layout config).
    /// - **Numpad key press**: tapping anywhere in the grid maps the touch
    ///   coordinates to a `(row, col)` cell and emits the corresponding key event.
    ///   The `%` key automatically includes a `KEY_LEFTSHIFT` modifier.
    /// - **Double-tap to unlock**: when numlock is off, a double-tap within
    ///   [`Layout::double_tap_delay_ms`] milliseconds temporarily enables a
    ///   single action (e.g. open calculator or toggle numlock).
    ///
    /// # Key release
    ///
    /// A key-down event is held until the finger is lifted (`BTN_TOOL_FINGER` value `0`),
    /// at which point a key-up event is emitted for the currently pressed key.
    ///
    /// # Arguments
    ///
    /// * `keys_layout` - A 2D grid (`[row][col]`) of [`KeyCode`] values defining
    ///   the numpad key mapping. `KEY_5` entries are remapped to the
    ///   layout-appropriate percentage key.
    ///
    /// # Errors
    ///
    /// Returns an error if any event fetch, key emit, or I2C command fails.
    ///
    /// # Panics
    ///
    /// Returns an error if the touchpad has not been opened and registered.
    pub fn process_events(
        &mut self,
        keys_layout: &Vec<Vec<KeyCode>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.opened {
            return Err(format!("The touchpad is not open!").into());
        }
        if !self.registered {
            return Err(format!("The touchpad is not registered!").into());
        }
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut button_pressed: Option<KeyCode> = None;
        let mut numlock: bool = false;
        let mut brightness: Brightness = Brightness::Off;
        // --- DOUBLE TAP STATE ---
        let mut last_down = std::time::Instant::now();
        let mut mode_enabled = false;
        let double_tap_delay =
            std::time::Duration::from_millis(self.layout.double_tap_delay_ms.into());

        loop {
            let events: Vec<InputEvent> = self.device.as_mut().unwrap().fetch_events()?.collect();

            for e in events {
                let ev_type = e.event_type();
                let code = e.code();
                let value = e.value();

                let is_pos_x =
                    ev_type == EventType::ABSOLUTE && code == AbsoluteAxisCode::ABS_MT_POSITION_X.0;
                let is_pos_y =
                    ev_type == EventType::ABSOLUTE && code == AbsoluteAxisCode::ABS_MT_POSITION_Y.0;
                let is_finger = ev_type == EventType::KEY && code == KeyCode::BTN_TOOL_FINGER.0;

                if !is_pos_x && !is_pos_y && !is_finger {
                    continue;
                }

                if is_pos_x {
                    x = value;
                    continue;
                }

                if is_pos_y {
                    y = value;
                    continue;
                }
                // --- BTN_TOOL_FINGER event ---
                if value == 0 {
                    // Finger lifted: release the currently held key
                    log_debug!("finger up at x {} y {}", x, y);

                    // Release key if needed
                    if let Some(key) = button_pressed {
                        log_debug!("send key up event {:?}", key);
                        if let Err(err) = self.udevice.as_mut().unwrap().emit(&[
                            InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTSHIFT.0, 0),
                            InputEvent::new(EventType::KEY.0, key.0, 0),
                            InputEvent::new(
                                EventType::SYNCHRONIZATION.0,
                                SynchronizationCode::SYN_REPORT.0,
                                0,
                            ),
                        ]) {
                            return Err(format!("Cannot send release event: {}", err).into());
                        }
                        button_pressed = None;
                    }
                } else if value == 1 && button_pressed.is_none() {
                    let now = std::time::Instant::now();
                    if now.duration_since(last_down) < double_tap_delay {
                        mode_enabled = !mode_enabled;
                    }
                    last_down = now;
                    // Finger placed: determine which action or key to trigger
                    log_debug!(
                        "finger down at x {} y {} - mode_enabled {}",
                        x,
                        y,
                        mode_enabled
                    );

                    let fx = x as f32;
                    let fy = y as f32;
                    let fmaxx = self.max_x as f32;
                    let fmaxy = self.max_y as f32;

                    // --- BLOCK EVERYTHING IF MODE NOT ENABLED ---
                    if !numlock && !mode_enabled {
                        continue;
                    }
                    mode_enabled = false;

                    // Top-right corner: toggle numlock
                    if fx >= self.layout.zones.numlock.x_min * fmaxx
                        && fx <= self.layout.zones.numlock.x_max * fmaxx
                        && fy >= self.layout.zones.numlock.y_min * fmaxy
                        && fy <= self.layout.zones.numlock.y_max * fmaxy
                    {
                        numlock = !numlock;
                        if numlock {
                            if Brightness::Off == brightness {
                                brightness = Brightness::High;
                            }
                            self.activate_numlock(&brightness)?;
                        } else {
                            self.deactivate_numlock()?;
                        }
                        continue;
                    }

                    // Top-left corner: cycle brightness (numlock on) or open calculator
                    if fx >= self.layout.zones.brightness_calculator.x_min * fmaxx
                        && fx <= self.layout.zones.brightness_calculator.x_max * fmaxx
                        && fy >= self.layout.zones.brightness_calculator.y_min * fmaxy
                        && fy <= self.layout.zones.brightness_calculator.y_max * fmaxy
                    {
                        if numlock {
                            self.change_brightness(&mut brightness)?;
                        } else if self.layout.allow_calculator {
                            self.launch_calculator()?;
                        }
                        continue;
                    }

                    // Outside numpad mode, let the touchpad handle the event normally
                    if !numlock {
                        continue;
                    }

                    // Map touch coordinates to a key grid cell
                    let col = (self.layout.cols as f32 * fx / (fmaxx + 1.0)).floor() as usize;
                    let row_raw = (self.layout.rows as f32 * fy / fmaxy) - self.layout.top_offset;

                    // Ignore taps in the top-offset reserved area
                    if row_raw < 0.0 {
                        continue;
                    }
                    let row = row_raw.floor() as usize;

                    let mut key = match keys_layout.get(row).and_then(|r| r.get(col)) {
                        Some(k) => *k,
                        None => {
                            log_debug!(
                                "Unhandled col/row {}/{} for position {}-{}",
                                col,
                                row,
                                x,
                                y
                            );
                            continue;
                        }
                    };

                    // Remap KEY_5 to the layout-appropriate percentage key
                    if key == KeyCode::KEY_5 {
                        key = self.percentage_key;
                    }

                    button_pressed = Some(key);
                    log_debug!("send press key event {:?}", key);

                    // Percentage key requires SHIFT to be held down
                    if let Err(err) = if key == self.percentage_key {
                        self.udevice.as_mut().unwrap().emit(&[
                            InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTSHIFT.0, 1),
                            InputEvent::new(EventType::KEY.0, key.0, 1),
                            InputEvent::new(
                                EventType::SYNCHRONIZATION.0,
                                SynchronizationCode::SYN_REPORT.0,
                                0,
                            ),
                        ])
                    } else {
                        self.udevice.as_mut().unwrap().emit(&[
                            InputEvent::new(EventType::KEY.0, key.0, 1),
                            InputEvent::new(
                                EventType::SYNCHRONIZATION.0,
                                SynchronizationCode::SYN_REPORT.0,
                                0,
                            ),
                        ])
                    } {
                        return Err(format!("Cannot send press event: {}", err).into());
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
