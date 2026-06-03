use evdev::KeyCode;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key(evdev::KeyCode);

impl Key {
    pub fn from_keycode(keycode: KeyCode) -> Option<Self> {
        match keycode {
            KeyCode::KEY_ESC | KeyCode::KEY_C => Some(Self(keycode)),
            _ => None,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            KeyCode::KEY_ESC => "esc",
            KeyCode::KEY_C => "c",
            _ => unreachable!("unsupported key should not be constructed"),
        })
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "esc" | "escape" => Ok(Self(KeyCode::KEY_ESC)),
            "c" => Ok(Self(KeyCode::KEY_C)),
            _ => Err(format!("unsupported key {value}")),
        }
    }
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
