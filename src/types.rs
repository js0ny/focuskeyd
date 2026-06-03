use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Esc,
    C,
}

impl fmt::Display for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Esc => "esc",
            Self::C => "c",
        })
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "esc" | "escape" => Ok(Self::Esc),
            "c" => Ok(Self::C),
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
