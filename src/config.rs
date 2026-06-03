use std::{fs, path::Path, str::FromStr};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::types::{Key, Modifier};

#[derive(Debug)]
pub struct Config {
    pub backend: Backend,
    pub shell: String,
    pub devices: Vec<String>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    Niri,
}

#[derive(Debug)]
pub struct Rule {
    pub name: String,
    pub app_ids: Vec<String>,
    pub key: Key,
    pub modifiers: Vec<Modifier>,
    pub actions: Vec<ActionConfig>,
}

#[derive(Debug)]
pub struct ActionConfig {
    pub exec: String,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    backend: Backend,
    shell: Option<String>,
    #[serde(default)]
    devices: Vec<String>,
    rules: Vec<RawRule>,
}

#[derive(Debug, Deserialize)]
struct RawRule {
    name: String,
    #[serde(rename = "match")]
    app_ids: Vec<String>,
    key: String,
    #[serde(default)]
    modifiers: Vec<String>,
    actions: Vec<RawActionConfig>,
}

#[derive(Debug, Deserialize)]
struct RawActionConfig {
    exec: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let raw: RawConfig = toml::from_str(&source)
            .with_context(|| format!("failed to parse config {}", path.display()))?;
        raw.try_into()
            .with_context(|| format!("failed to validate config {}", path.display()))
    }
}

impl TryFrom<RawConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(raw: RawConfig) -> Result<Self> {
        if raw.rules.is_empty() {
            bail!("config must contain at least one rule");
        }

        let rules = raw
            .rules
            .into_iter()
            .map(Rule::try_from)
            .collect::<Result<Vec<_>>>()?;

        let shell = raw
            .shell
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()));

        Ok(Self {
            backend: raw.backend,
            shell,
            devices: raw.devices,
            rules,
        })
    }
}

impl TryFrom<RawRule> for Rule {
    type Error = anyhow::Error;

    fn try_from(raw: RawRule) -> Result<Self> {
        let name = raw.name.trim().to_string();
        if name.is_empty() {
            bail!("rule name cannot be empty");
        }

        let app_ids = raw
            .app_ids
            .into_iter()
            .map(|app_id| app_id.trim().to_string())
            .collect::<Vec<_>>();

        if app_ids.is_empty() || app_ids.iter().any(|app_id| app_id.is_empty()) {
            bail!("rule {name} must contain at least one non-empty app_id match");
        }

        let key = Key::from_str(&raw.key).map_err(anyhow::Error::msg)?;
        let modifiers = raw
            .modifiers
            .into_iter()
            .map(|modifier| Modifier::from_str(&modifier).map_err(anyhow::Error::msg))
            .collect::<Result<Vec<_>>>()?;

        if raw.actions.is_empty() {
            bail!("rule {name} must contain at least one action");
        }

        let actions = raw
            .actions
            .into_iter()
            .map(ActionConfig::try_from)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            name,
            app_ids,
            key,
            modifiers,
            actions,
        })
    }
}

impl TryFrom<RawActionConfig> for ActionConfig {
    type Error = anyhow::Error;

    fn try_from(raw: RawActionConfig) -> Result<Self> {
        if raw.exec.trim().is_empty() {
            bail!("exec action cannot be empty");
        }

        Ok(Self { exec: raw.exec })
    }
}
