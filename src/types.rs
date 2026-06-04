use evdev::KeyCode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key(evdev::KeyCode);

impl Key {
    pub fn from_keycode(keycode: KeyCode) -> Self {
        Self(keycode)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = format!("{:?}", self.0).to_ascii_lowercase();
        f.write_str(name.strip_prefix("key_").unwrap_or(&name))
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(name) = normalise_key_name(value) else {
            return Err("key cannot be empty".to_string());
        };

        KeyCode::from_str(&name)
            .map(Self)
            .map_err(|_| format!("unsupported key {value}"))
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

fn normalise_key_name(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let value = value.replace('-', "_");
    let lower = value.to_ascii_lowercase();
    let upper = value.to_ascii_uppercase();

    if lower.starts_with("key_") || lower.starts_with("btn_") {
        return Some(upper);
    }

    let name = match lower.as_str() {
        "esc" | "escape" => "KEY_ESC".to_string(),
        "enter" | "return" => "KEY_ENTER".to_string(),
        "backspace" => "KEY_BACKSPACE".to_string(),
        "space" => "KEY_SPACE".to_string(),
        "tab" => "KEY_TAB".to_string(),
        "delete" | "del" => "KEY_DELETE".to_string(),
        "insert" | "ins" => "KEY_INSERT".to_string(),
        "pageup" | "page_up" | "pgup" => "KEY_PAGEUP".to_string(),
        "pagedown" | "page_down" | "pgdown" => "KEY_PAGEDOWN".to_string(),
        "up" => "KEY_UP".to_string(),
        "down" => "KEY_DOWN".to_string(),
        "left" => "KEY_LEFT".to_string(),
        "right" => "KEY_RIGHT".to_string(),
        "minus" => "KEY_MINUS".to_string(),
        "equal" | "equals" => "KEY_EQUAL".to_string(),
        "comma" => "KEY_COMMA".to_string(),
        "dot" | "period" => "KEY_DOT".to_string(),
        "slash" => "KEY_SLASH".to_string(),
        "backslash" => "KEY_BACKSLASH".to_string(),
        "semicolon" => "KEY_SEMICOLON".to_string(),
        "apostrophe" | "quote" => "KEY_APOSTROPHE".to_string(),
        "grave" | "backtick" => "KEY_GRAVE".to_string(),
        "leftbracket" | "left_bracket" => "KEY_LEFTBRACE".to_string(),
        "rightbracket" | "right_bracket" => "KEY_RIGHTBRACE".to_string(),
        _ => format!("KEY_{upper}"),
    };

    Some(name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Super,
}

impl FromStr for Modifier {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Ok(Self::Ctrl),
            "alt" => Ok(Self::Alt),
            "shift" => Ok(Self::Shift),
            "super" | "meta" => Ok(Self::Super),
            _ => Err(format!("unsupported modifier {value}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_short_key_names() {
        assert_eq!(Key::from_str("esc").unwrap(), Key(KeyCode::KEY_ESC));
        assert_eq!(Key::from_str("c").unwrap(), Key(KeyCode::KEY_C));
        assert_eq!(Key::from_str("f12").unwrap(), Key(KeyCode::KEY_F12));
        assert_eq!(
            Key::from_str("volumeup").unwrap(),
            Key(KeyCode::KEY_VOLUMEUP)
        );
    }

    #[test]
    fn parses_evdev_key_names() {
        assert_eq!(Key::from_str("KEY_ESC").unwrap(), Key(KeyCode::KEY_ESC));
        assert_eq!(Key::from_str("key_c").unwrap(), Key(KeyCode::KEY_C));
        assert_eq!(Key::from_str("BTN_LEFT").unwrap(), Key(KeyCode::BTN_LEFT));
    }

    #[test]
    fn display_roundtrips() {
        for key in [
            KeyCode::KEY_ESC,
            KeyCode::KEY_C,
            KeyCode::KEY_F12,
            KeyCode::BTN_LEFT,
        ] {
            let key = Key(key);
            assert_eq!(Key::from_str(&key.to_string()).unwrap(), key);
        }
    }
}
