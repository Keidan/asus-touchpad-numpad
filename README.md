# asus-touchpad-numpad
[![Build Status](https://github.com/Keidan/asus-touchpad-numpad/actions/workflows/build.yml/badge.svg)][build]
[![Release](https://img.shields.io/github/v/release/Keidan/asus-touchpad-numpad.svg?logo=github)][releases]
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)][license]

A userspace driver that turns the touchpad of supported ASUS laptops into a software numpad overlay. 

It reads raw input events from the touchpad, emits synthetic key events through a virtual device, and controls the numpad backlight over I2C.

---

## Supported models

The following layouts are included out of the box:

| Layout name | Laptop model             | Grid  |
|-------------|--------------------------|-------|
| `g533qr`    | ASUS ROG Strix SCAR 15   | 5 × 4 |
| `ux433fa`   | ASUS ZenBook 14          | 5 × 4 |
| `gx701`     | ASUS ROG Zephyrus S17    | 4 × 5 |

If your model is not listed, you can add your own layout directly in `config.json` — see [Adding a custom layout](#adding-a-custom-layout).

Layout detection is automatic by default: the driver reads `/sys/class/dmi/id/board_name` and selects the matching layout. You can also pin a layout explicitly by setting `layout_name` in `config.json`.

---

## Prerequisites

### Fedora / RHEL-based systems

Refresh your package metadata and upgrade existing packages, then install the Rust toolchain and the systemd development headers:

```bash
sudo dnf upgrade --refresh
sudo dnf install rust cargo systemd-devel
```

`systemd-devel` provides the headers required to compile crates that link against `libsystemd` (used transitively by some dependencies). Without it, the build will fail with missing header errors.

To build `.deb` and `.rpm` packages, install `fpm` and its dependencies:

```bash
sudo dnf install ruby ruby-devel gcc make rpm-build
sudo gem install fpm
```

### Debian / Ubuntu-based systems

Update your package metadata and upgrade existing packages, then install the Rust toolchain and the systemd development headers:

```bash
sudo apt update && sudo apt upgrade
sudo apt install rust-all cargo libsystemd-dev
```

`libsystemd-dev` provides the headers required to compile crates that link against `libsystemd` (used transitively by some dependencies). Without it, the build will fail with missing header errors.

To build `.deb` and `.rpm` packages, install `fpm` and its dependencies:

```bash
sudo apt install ruby ruby-dev gcc make
sudo gem install fpm
```

### Other distributions

Make sure the following are available:

- **Rust** (stable) and **Cargo** — install via [rustup](https://rustup.rs/) if your distro's package is outdated.
- **libi2c / i2c-tools** development headers — needed to communicate with the numpad controller over I2C.
- **libevdev** development headers — needed for raw input event access.

---

## Building

```bash
cargo build --release
```

The compiled binary is placed at `target/release/asus-touchpad-numpad`.

---

## Documentation

Generate and open the developer documentation locally with:

```bash
cargo doc --document-private-items --open
```

The generated HTML is placed at `target/doc/asus_touchpad_numpad/index.html`.

---

## Running

The binary requires access to:

- `/dev/input/event*` — to read raw touchpad events.
- `/dev/uinput` — to create the virtual key device.
- `/dev/i2c-*` — to send backlight control commands.

Your user must be a member of the `input`, `i2c`, `uinput` groups:
```bash
  sudo usermod -aG input,i2c,uinput $USER
  # log out and back in for the change to take effect
```

### Manual

Place the binary and `config.json` in the same directory, then run:

```bash
./asus-touchpad-numpad
```

Or with `sudo` if you have not configured group access yet:

```bash
sudo ./asus-touchpad-numpad

### System service

Run `manual_install.sh` to install the binary system-wide and register it as a systemd service that starts automatically on boot:

```bash
sudo ./manual_install.sh
```

### Packages

Run `create_package.sh` to build a `.deb` (Debian/Ubuntu) and a `.rpm` (Fedora/RHEL) package:

```bash
./create_package.sh
```

The generated packages can then be installed with your system's package manager and will handle installation and service registration automatically.

---

## Touchpad zones

The touchpad surface is divided into three areas:

```
┌──────────────────────────────────────────┐
│ [Brightness/Calc] ·····  [NumLock toggle]│  ← top bar
├──────────────────────────────────────────┤
│                                          │
│             Numpad key grid              │
│                                          │
└──────────────────────────────────────────┘
```

**Top-right corner — NumLock toggle**
Tap to enable or disable the numpad overlay. When enabled, the touchpad is grabbed exclusively (normal pointer movement is suspended) and the backlight turns on.

**Top-left corner — Brightness / Calculator**
- NumLock **on** → cycles the backlight through `Low → Medium → High → Low → …`
- NumLock **off** → launches the system calculator application (if `allow_calculator` is `true` in the layout config).

**Key grid**
Touch coordinates are mapped to a `(row, col)` cell and the corresponding key event is emitted. The key is held until the finger lifts.

---

## Configuration

The configuration file `config.json` must be placed in the same directory as the binary.

### Top-level fields

| Field         | Type   | Description                                                      |
|---------------|--------|------------------------------------------------------------------|
| `log_level`   | string | Minimum log verbosity: `"debug"`, `"info"`, `"warn"`, `"error"`. |
| `layout_name` | string | Name of the active layout, or `"auto"` for DMI-based detection.  |
| `identify`    | object | Device detection retry settings (see below).                     |
| `layouts`     | array  | List of all layout definitions (built-in and custom).            |

### `identify` object

| Field            | Type | Description                                          |
|------------------|------|------------------------------------------------------|
| `max_attempts`   | u32  | How many times to probe `/proc/bus/input/devices`.   |
| `retry_delay_ms` | u32  | Milliseconds to wait between probe attempts.         |

### Layout fields

| Field                         | Type   | Description                                                                   |
|-------------------------------|--------|-------------------------------------------------------------------------------|
| `name`                        | string | Unique identifier matching `layout_name` or the DMI board name for auto-mode. |
| `cols` / `rows`               | u8     | Dimensions of the key grid.                                                   |
| `keymap`                      | array  | Row-major grid of `evdev` key name strings (e.g. `"KEY_KP7"`).                |
| `top_offset`                  | f32    | Fraction of touchpad height reserved above the key grid (e.g. logo area).     |
| `double_tap_delay_ms`         | u32    | Double-tap window in ms for unlock-mode. Set to `0` to disable.               |
| `allow_calculator`            | bool   | Whether the top-left zone can launch the calculator when NumLock is off.       |
| `brightness_levels`           | object | Raw I2C values for `low`, `med`, and `high` brightness levels.                |
| `zones.numlock`               | object | Normalized coordinates (`x_min`, `x_max`, `y_min`, `y_max`) of the NumLock button zone.         |
| `zones.brightness_calculator` | object | Normalized coordinates (`x_min`, `x_max`, `y_min`, `y_max`) of the brightness/calc zone.        |

Zone coordinates are ratios in `[0.0, 1.0]` relative to the touchpad's physical dimensions. See [Calibrating touch zones](#calibrating-touch-zones) for how to compute them.

### Example `config.json` (excerpt)

```json
{
  "log_level": "info",
  "layout_name": "auto",
  "identify": {
    "max_attempts": 5,
    "retry_delay_ms": 100
  },
  "layouts": [
    {
      "name": "ux433fa",
      "cols": 5,
      "rows": 4,
      "zones": {
        "numlock": { "x_min": 0.95, "x_max": 1.0, "y_min": 0.0, "y_max": 0.09 },
        "brightness_calculator": { "x_min": 0.00, "x_max": 0.06, "y_min": 0.0, "y_max": 0.07 }
      },
      "double_tap_delay_ms": 250,
      "allow_calculator": true,
      "top_offset": 0.10,
      "keymap": [
        ["KEY_KP7", "KEY_KP8", "KEY_KP9", "KEY_KPSLASH",    "KEY_BACKSPACE"],
        ["KEY_KP4", "KEY_KP5", "KEY_KP6", "KEY_KPASTERISK", "KEY_BACKSPACE"],
        ["KEY_KP1", "KEY_KP2", "KEY_KP3", "KEY_KPMINUS",    "KEY_KPENTER" ],
        ["KEY_KP0", "KEY_KP0", "KEY_KPDOT","KEY_KPPLUS",    "KEY_KPENTER" ]
      ],
      "brightness_levels": { "low": 1, "med": 24, "high": 31 }
    }
  ]
}
```

---

## Adding a custom layout

To add support for a model not listed above:

1. Add a new entry to the `layouts` array in `config.json` with the appropriate `name`, `rows`, `cols`, `keymap`, and `zones`.
2. Set `layout_name` to your layout's `name`, or leave it as `"auto"` if the name matches the DMI board name returned by `/sys/class/dmi/id/board_name`.
3. Calibrate the touch zones as described in the next section.

---

## Calibrating touch zones

Zone coordinates are normalized ratios computed from raw touch values:

```
ratio = raw_value / max_value
```

For example, if the NumLock button's leftmost tap reads `x = 3243` and the touchpad reports `maxx = 4036`:

```
x_min = 3243 / 4036 = 0.803  →  use 0.80 (with a small safety margin)
```

**Step-by-step:**

1. Set `log_level` to `"debug"` in `config.json`.
2. Run the driver and tap repeatedly across the full area of each button.
3. Collect the raw coordinates from log lines:
   ```
   finger down at x <X> y <Y>
   ```
4. Collect the max values logged at startup:
   ```
   Touchpad min/max: x [0:<MAXX>], y [0:<MAXY>]
   ```
5. Compute the ratios:

   | Zone                    | Formula                          | Field    |
   |-------------------------|----------------------------------|----------|
   | NumLock left edge       | `min(x) / MAXX − 0.02`           | `x_min`  |
   | NumLock bottom edge     | `max(y) / MAXY + 0.02`           | `y_max`  |
   | Brightness right edge   | `max(x) / MAXX + 0.02`           | `x_max`  |
   | Brightness bottom edge  | `max(y) / MAXY + 0.02`           | `y_max`  |

6. Update the `zones` fields in `config.json`.

---

## How it works

1. **Configuration** — `config.json` is parsed; the active layout is resolved by name or via DMI auto-detection.
2. **Device detection** — `/proc/bus/input/devices` is scanned for a touchpad whose name starts with `ASUE` or `ELAN` and contains `"Touchpad"`.
3. **Keyboard layout detection** — `/etc/vconsole.conf` or `localectl status` is read to detect AZERTY vs QWERTY (affects the `%` key mapping).
4. **Virtual device** — A `uinput` virtual device named `"Asus Touchpad/Numpad"` is created with the required key capabilities.
5. **Event loop** — Raw `evdev` events are read continuously and dispatched to the appropriate zone handler or key emitter.

### Double-tap to unlock

When NumLock is off, a single tap is ignored. Two taps within `double_tap_delay_ms` milliseconds temporarily allow the next action (useful for quick calculator access without permanently enabling NumLock).

### Backlight brightness cycle

`Off → Low → Medium → High → Low → …`

Brightness values are raw bytes written directly to the numpad I2C controller at address `0x15`.

---

## Keyboard layout support

| Detected layout  | `%` key mapping            |
|------------------|----------------------------|
| QWERTY           | `KEY_5` + `SHIFT`          |
| AZERTY (fr, be)  | `KEY_APOSTROPHE` + `SHIFT` |

Detection reads `KEYMAP=` from `/etc/vconsole.conf`, then falls back to `localectl status`.

---*

## Contributors

[See the full list][contributors]

---*

## License

[GNU GPL v3 or later](https://github.com/Keidan/asus-touchpad-numpad/blob/master/license.txt)

[build]: https://github.com/Keidan/asus-touchpad-numpad/actions
[releases]: https://github.com/Keidan/asus-touchpad-numpad/releases
[license]: https://github.com/Keidan/asus-touchpad-numpad/blob/master/license.txt*
[contributors]: https://github.com/Keidan/asus-touchpad-numpad/blob/master/CONTRIBUTORS.md*