use std::{
    fs::{self, File},
    io::Read,
    mem,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
};

use anyhow::{bail, Context, Result};
use tracing::{debug, info, warn};

use crate::types::{Key, Modifier};

const EV_KEY: u16 = 0x01;
const KEY_ESC: u16 = 1;
const KEY_C: u16 = 46;
const KEY_LEFTCTRL: u16 = 29;
const KEY_RIGHTCTRL: u16 = 97;
const KEY_LEFTSHIFT: u16 = 42;
const KEY_RIGHTSHIFT: u16 = 54;
const KEY_LEFTALT: u16 = 56;
const KEY_RIGHTALT: u16 = 100;
const KEY_LEFTMETA: u16 = 125;
const KEY_RIGHTMETA: u16 = 126;

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

/// Corresponds to kernel's `struct input_event` but only contains the fields we care about.
#[derive(Debug)]
struct RawKeyEvent {
    code: u16,
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
            info!(devices = devices.len(), "discovered input devices");
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
            match File::open(&path) {
                Ok(file) => {
                    opened += 1;
                    info!(device = %path.display(), "opened input device");
                    let sender = sender.clone();
                    thread::spawn(move || read_device(path, file, sender));
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

            let Some(key) = key_name(event.code) else {
                continue;
            };

            debug!(%key, modifiers = ?self.modifiers, "observed supported key press");

            return Ok(KeyEvent {
                key,
                modifiers: self.modifiers,
            });
        }
    }

    fn update_modifiers(&mut self, code: u16, value: i32) {
        let pressed = value != 0;
        match code {
            KEY_LEFTCTRL | KEY_RIGHTCTRL => {
                self.modifiers.ctrl = pressed;
                debug!(pressed, "ctrl modifier changed");
            }
            KEY_LEFTALT | KEY_RIGHTALT => {
                self.modifiers.alt = pressed;
                debug!(pressed, "alt modifier changed");
            }
            KEY_LEFTSHIFT | KEY_RIGHTSHIFT => {
                self.modifiers.shift = pressed;
                debug!(pressed, "shift modifier changed");
            }
            KEY_LEFTMETA | KEY_RIGHTMETA => {
                self.modifiers.super_key = pressed;
                debug!(pressed, "super modifier changed");
            }
            _ => {}
        }
    }
}

fn discover_devices() -> Result<Vec<PathBuf>> {
    let mut devices = Vec::new();

    for entry in fs::read_dir("/dev/input").context("failed to read /dev/input")? {
        let entry = entry?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("event"))
        {
            devices.push(path);
        }
    }

    Ok(devices)
}

fn read_device(path: PathBuf, mut file: File, sender: mpsc::Sender<RawKeyEvent>) {
    let event_size = mem::size_of::<libc::input_event>();
    let mut buffer = vec![0_u8; event_size];

    loop {
        if let Err(error) = file.read_exact(&mut buffer) {
            warn!(device = %path.display(), error = %error, "failed to read input device");
            return;
        }

        let event = unsafe { (buffer.as_ptr() as *const libc::input_event).read_unaligned() };
        if event.type_ == EV_KEY {
            if sender
                .send(RawKeyEvent {
                    code: event.code,
                    value: event.value,
                })
                .is_err()
            {
                warn!(device = %path.display(), "input event receiver stopped");
                return;
            }
        }
    }
}

fn key_name(code: u16) -> Option<Key> {
    match code {
        KEY_ESC => Some(Key::Esc),
        KEY_C => Some(Key::C),
        _ => None,
    }
}
