use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;
use tracing::debug;

#[derive(Debug)]
#[allow(dead_code)]
pub struct WindowInfo {
    pub app_id: Option<String>,
    pub title: Option<String>,
    pub id: Option<String>,
    pub pid: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct NiriWindow {
    app_id: Option<String>,
    title: Option<String>,
    id: Option<Value>,
    pid: Option<u32>,
}

pub fn focused_window() -> Result<WindowInfo> {
    debug!("running niri focused-window command");
    let output = Command::new("niri")
        .args(["msg", "--json", "focused-window"])
        .output()
        .context("failed to run niri msg --json focused-window")?;

    if !output.status.success() {
        bail!(
            "niri focused-window failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let window: NiriWindow = serde_json::from_slice(&output.stdout)
        .context("failed to parse niri focused-window output")?;

    let window = WindowInfo {
        app_id: window.app_id,
        title: window.title,
        id: window.id.map(value_to_string),
        pid: window.pid,
    };

    debug!(window = ?window, "parsed niri focused window");
    Ok(window)
}

fn value_to_string(value: Value) -> String {
    match value {
        Value::String(value) => value,
        other => other.to_string(),
    }
}
