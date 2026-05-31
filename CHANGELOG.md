# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [0.1.0] - 2026-05-31

### Added

- Initial release.
- Userspace numpad driver for supported ASUS laptops, written in Rust.
- `evdev`-based touchpad input reading and `uinput` virtual device output.
- I2C backlight control (on/off, three brightness levels: low, medium, high).
- Built-in layouts for `g533qr` (ROG Strix SCAR 15), `ux433fa` (ZenBook 14), and `gx701` (Zephyrus S17).
- DMI-based automatic layout detection via `/sys/class/dmi/id/board_name`.
- JSON configuration (`config.json`) with support for custom user-defined layouts.
- Top-right zone: NumLock toggle with touchpad grab/release and backlight switching.
- Top-left zone: brightness cycle when NumLock is on, calculator launch when off.
- Double-tap unlock mechanism for single-action access without enabling NumLock.
- AZERTY / QWERTY detection via `/etc/vconsole.conf` and `localectl` fallback, affecting the `%` key mapping.
- Touchpad device auto-detection by scanning `/proc/bus/input/devices` for `ASUE`/`ELAN` touchpad entries, with configurable retry attempts and delay.
- Structured logger with `debug`, `info`, `warn`, `error`, and `fatal` levels.
