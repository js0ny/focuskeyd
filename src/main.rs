mod config;
mod event;
mod niri;
mod types;

use std::{env, path::PathBuf, process::Command};

use anyhow::{Context, Result};
use config::{Config, Rule};
use event::{EvdevEventSource, KeyEvent};
use niri::WindowInfo;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    init_logging();

    let config_path = config_path()?;
    info!(config = %config_path.display(), "loading config");

    let config = Config::load(&config_path)?;
    info!(backend = ?config.backend, rules = config.rules.len(), "config loaded");

    info!(shell = ?config.shell, "using shell for actions");

    let mut event_source = EvdevEventSource::new(&config.devices)?;
    info!(config = %config_path.display(), "focuskeyd started");

    loop {
        let event = event_source.next_event()?;
        debug!(key = %event.key, modifiers = ?event.modifiers, "interesting key event");

        if !has_trigger(&config.rules, &event) {
            debug!(key = %event.key, modifiers = ?event.modifiers, "no configured trigger matched");
            continue;
        }

        debug!("querying focused window");
        let window = match niri::focused_window() {
            Ok(window) => window,
            Err(error) => {
                warn!(error = %format_args!("{error:#}"), "failed to query focused window");
                continue;
            }
        };
        debug!(window = ?window, "focused window queried");

        for rule in matching_rules(&config.rules, &event, &window) {
            info!(rule = rule.name, key = %event.key, window = ?window, "rule matched");
            for action in &rule.actions {
                debug!(rule = rule.name, command = action.exec, "running action");
                if let Err(error) = run_exec(&action.exec, &config.shell) {
                    error!(rule = rule.name, error = %format_args!("{error:#}"), "action failed");
                } else {
                    debug!(rule = rule.name, command = action.exec, "action completed");
                }
            }
        }
    }
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn config_path() -> Result<PathBuf> {
    if let Some(path) = env::args_os().nth(1) {
        return Ok(path.into());
    }

    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home).join("focuskeyd/config.toml"));
    }

    let home = env::var_os("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".config/focuskeyd/config.toml"))
}

fn has_trigger(rules: &[Rule], event: &KeyEvent) -> bool {
    rules.iter().any(|rule| trigger_matches(rule, event))
}

fn matching_rules<'a>(rules: &'a [Rule], event: &KeyEvent, window: &WindowInfo) -> Vec<&'a Rule> {
    rules
        .iter()
        .filter(move |rule| trigger_matches(rule, event) && window_matches(rule, window))
        .collect()
}

fn trigger_matches(rule: &Rule, event: &KeyEvent) -> bool {
    rule.key == event.key && event.modifiers.matches(&rule.modifiers)
}

fn window_matches(rule: &Rule, window: &WindowInfo) -> bool {
    window
        .app_id
        .as_ref()
        .is_some_and(|app_id| rule.app_ids.iter().any(|expected| expected == app_id))
}

fn run_exec(command: &str, shell: &str) -> Result<()> {
    let status = Command::new(shell)
        .args(["-c", command])
        .status()
        .with_context(|| format!("failed to run {command}"))?;

    if !status.success() {
        anyhow::bail!("command exited with {status}");
    }

    Ok(())
}
