/// Represents the detected keyboard layout family.
///
/// Used to adapt key label rendering or input handling to the user's locale.
/// Detection is performed via [`KeyboardLayout::detect`].
#[derive(Debug, PartialEq)]
pub enum KeyboardLayout {
    /// French and Belgian AZERTY layout (fr, be). Detected for keymaps starting with `"fr"` or `"be"`.
    Azerty,
    /// English QWERTY layout.
    Qwerty,
    /// Layout could not be mapped to a known family.
    /// Contains the raw keymap string, or `"undetected"` if no keymap source was available.
    Unknown(String),
}

impl KeyboardLayout {
    /// Detects the current system keyboard layout.
    ///
    /// Tries two sources in order:
    ///
    /// 1. **`/etc/vconsole.conf`** — reads the `KEYMAP=` field (quotes stripped).
    /// 2. **`localectl status`** — parses `X11 Layout:` or `VC Keymap:` from the output.
    ///
    /// The first successfully parsed keymap is passed to [`KeyboardLayout::from_keymap`]
    /// and returned. If neither source yields a result,
    /// returns `KeyboardLayout::Unknown("undetected")`.
    ///
    /// # Example
    ///
    /// ```rust
    /// match KeyboardLayout::detect() {
    ///     KeyboardLayout::Azerty  => println!("AZERTY layout"),
    ///     KeyboardLayout::Qwerty  => println!("QWERTY layout"),
    ///     KeyboardLayout::Unknown(k) => println!("Unknown layout: {}", k),
    /// }
    /// ```
    pub fn detect() -> Self {
        if let Ok(content) = std::fs::read_to_string("/etc/vconsole.conf") {
            for line in content.lines() {
                if let Some(keymap) = line.strip_prefix("KEYMAP=") {
                    let keymap = keymap.trim_matches('"').trim_matches('\'');
                    return Self::from_keymap(keymap);
                }
            }
        }

        // Fallback to localectl
        if let Ok(output) = std::process::Command::new("localectl")
            .arg("status")
            .output()
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines() {
                    let line = line.trim();
                    if let Some(layout) = line.strip_prefix("X11 Layout:") {
                        return Self::from_keymap(layout.trim());
                    }
                    if let Some(layout) = line.strip_prefix("VC Keymap:") {
                        return Self::from_keymap(layout.trim());
                    }
                }
            }
        }

        KeyboardLayout::Unknown(String::from("undetected"))
    }

    /// Maps a raw keymap string to a [`KeyboardLayout`] variant.
    ///
    /// Matching is prefix-based and case-sensitive. `Qwerty` is the default
    /// for any keymap not explicitly matched:
    ///
    /// | Condition                        | Returned variant    |
    /// |----------------------------------|---------------------|
    /// | Starts with `"fr"` or `"be"`     | `Azerty`            |
    /// | `"undetected"` or empty string   | `Unknown(keymap)`   |
    /// | Anything else                    | `Qwerty`            |
    ///
    /// # Arguments
    ///
    /// * `keymap` - Raw keymap identifier string (e.g. `"fr"`, `"us"`, `"fr-latin9"`).
    fn from_keymap(keymap: &str) -> Self {
        match keymap {
            k if k.starts_with("fr") | k.starts_with("be") => Self::Azerty,
            "undetected" | "" => Self::Unknown(keymap.to_string()),
            _ => Self::Qwerty,
        }
    }
}
