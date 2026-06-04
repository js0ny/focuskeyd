use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
};

use anyhow::{bail, Context, Result};
use evdev::{Device, EventSummary, KeyCode};
use tracing::{debug, info, warn};

use crate::types::{Key, Modifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub fn matches(self, required: &[Modifier]) -> bool {
        let wants_ctrl = required.contains(&Modifier::Ctrl);
        let wants_alt = required.contains(&Modifier::Alt);
        let wants_shift = required.contains(&Modifier::Shift);
        let wants_super = required.contains(&Modifier::Super);

        self.ctrl == wants_ctrl
            && self.alt == wants_alt
            && self.shift == wants_shift
            && self.super_key == wants_super
    }
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}

#[derive(Debug)]
struct RawKeyEvent {
    code: KeyCode,
    value: i32,
}

pub struct EvdevEventSource {
    receiver: Receiver<RawKeyEvent>,
    modifiers: Modifiers,
}

impl EvdevEventSource {
    pub fn new(configured_devices: &[String]) -> Result<Self> {
        let devices = if configured_devices.is_empty() {
            let devices = discover_devices()?;
            info!(devices = devices.len(), "discovered keyboard input devices");
            devices
        } else {
            let devices: Vec<_> = configured_devices.iter().map(PathBuf::from).collect();
            info!(devices = devices.len(), "using configured input devices");
            devices
        };

        if devices.is_empty() {
            bail!("no input devices found");
        }

        let (sender, receiver) = mpsc::channel();
        let mut opened = 0;

        for path in devices {
            match Device::open(&path) {
                Ok(device) => {
                    opened += 1;
                    info!(device = %path.display(), name = ?device.name(), "opened input device");
                    let sender = sender.clone();
                    thread::spawn(move || read_device(path, device, sender));
                }
                Err(error) => {
                    warn!(device = %path.display(), error = %error, "failed to open input device")
                }
            }
        }

        if opened == 0 {
            bail!("failed to open any input devices");
        }

        Ok(Self {
            receiver,
            modifiers: Modifiers {
                ctrl: false,
                alt: false,
                shift: false,
                super_key: false,
            },
        })
    }

    pub fn next_event(&mut self) -> Result<KeyEvent> {
        loop {
            let event = self.receiver.recv().context("input event source stopped")?;
            self.update_modifiers(event.code, event.value);

            if event.value != 1 {
                continue;
            }

            let key = Key::from_keycode(event.code);

            debug!(%key, modifiers = ?self.modifiers, "observed supported key press");

            return Ok(KeyEvent {
                key,
                modifiers: self.modifiers,
            });
        }
    }

    fn update_modifiers(&mut self, code: KeyCode, value: i32) {
        let pressed = value != 0;
        match code {
            KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL => {
                self.modifiers.ctrl = pressed;
                debug!(pressed, "ctrl modifier changed");
            }
            KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT => {
                self.modifiers.alt = pressed;
                debug!(pressed, "alt modifier changed");
            }
            KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT => {
                self.modifiers.shift = pressed;
                debug!(pressed, "shift modifier changed");
            }
            KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA => {
                self.modifiers.super_key = pressed;
                debug!(pressed, "super modifier changed");
            }
            _ => {}
        }
    }
}

fn discover_devices() -> Result<Vec<PathBuf>> {
    Ok(evdev::enumerate()
        .filter(|(_, device)| is_keyboard_device(device))
        .map(|(path, _)| path)
        .collect())
}

fn read_device(path: PathBuf, mut device: Device, sender: mpsc::Sender<RawKeyEvent>) {
    info!(device = %path.display(), "started input device reader");

    loop {
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(error) => {
                warn!(device = %path.display(), error = %error, "failed to read input device");
                return;
            }
        };

        for event in events {
            if let EventSummary::Key(_, code, value) = event.destructure() {
                if sender.send(RawKeyEvent { code, value }).is_err() {
                    warn!(device = %path.display(), "input event receiver stopped");
                    return;
                }
            }
        }
    }
}

fn is_keyboard_device(device: &Device) -> bool {
    device.supported_keys().is_some_and(|keys| {
        keys.contains(KeyCode::KEY_ESC)
            && keys.contains(KeyCode::KEY_C)
            && keys.contains(KeyCode::KEY_ENTER)
    })
}
