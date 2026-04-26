//! Configuration handling for PIR repositories.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_PIR_DIR: &str = "doc/pir";
pub const LEGACY_CONFIG_FILE: &str = ".pir-dir";
pub const CONFIG_FILE: &str = "pirs.toml";
pub const ENV_PIR_DIRECTORY: &str = "PIR_DIRECTORY";
pub const ENV_PIRS_CONFIG: &str = "PIRS_CONFIG";

/// Configuration for a PIR repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub pir_dir: PathBuf,
    #[serde(default)]
    pub templates: TemplateConfig,
    #[serde(default)]
    pub privacy: PrivacyConfig,
    #[serde(default)]
    pub mcp: McpConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pir_dir: PathBuf::from(DEFAULT_PIR_DIR),
            templates: TemplateConfig::default(),
            privacy: PrivacyConfig::default(),
            mcp: McpConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TemplateConfig {
    /// Default template variant for new PIRs (`development`, `production`, ...).
    pub default: Option<String>,
    /// Optional path to a custom template file.
    pub custom: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacyConfig {
    /// Regex patterns for redacting captured output (opt-in).
    pub redaction_patterns: Vec<String>,
    /// Field names whose values should be redacted on `--redact` exports.
    pub sensitive_fields: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    /// Whether the MCP HTTP transport is enabled.
    pub http_enabled: bool,
    /// Bind address for the MCP HTTP transport.
    pub http_bind: Option<String>,
}

impl Config {
    /// Load configuration from the given root directory.
    pub fn load(root: &Path) -> Result<Self> {
        let config_path = root.join(CONFIG_FILE);
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            if config.pir_dir.as_os_str().is_empty() {
                return Err(Error::ConfigError("pir_dir cannot be empty".into()));
            }
            return Ok(config);
        }
        let legacy_path = root.join(LEGACY_CONFIG_FILE);
        if legacy_path.exists() {
            let pir_dir = std::fs::read_to_string(&legacy_path)?.trim().to_string();
            if pir_dir.is_empty() {
                return Err(Error::ConfigError("PIR directory path is empty".into()));
            }
            return Ok(Self {
                pir_dir: PathBuf::from(pir_dir),
                ..Default::default()
            });
        }
        let default_dir = root.join(DEFAULT_PIR_DIR);
        if default_dir.exists() {
            return Ok(Self::default());
        }
        Err(Error::PirDirNotFound)
    }

    pub fn load_or_default(root: &Path) -> Self {
        Self::load(root).unwrap_or_default()
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let path = root.join(CONFIG_FILE);
        let content =
            toml::to_string_pretty(self).map_err(|e| Error::ConfigError(e.to_string()))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn pir_path(&self, root: &Path) -> PathBuf {
        root.join(&self.pir_dir)
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredConfig {
    pub config: Config,
    pub root: PathBuf,
    pub source: ConfigSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    Project(PathBuf),
    Environment,
    Default,
}

/// Discover configuration walking upward from `start_dir`.
pub fn discover(start_dir: &Path) -> Result<DiscoveredConfig> {
    if let Ok(env_path) = std::env::var(ENV_PIRS_CONFIG) {
        let path = PathBuf::from(&env_path);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let mut config: Config = toml::from_str(&content)?;
            apply_env_overrides(&mut config);
            return Ok(DiscoveredConfig {
                config,
                root: path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| start_dir.to_path_buf()),
                source: ConfigSource::Environment,
            });
        }
    }

    let mut current = start_dir.to_path_buf();
    loop {
        let toml_path = current.join(CONFIG_FILE);
        let legacy_path = current.join(LEGACY_CONFIG_FILE);
        if toml_path.exists() || legacy_path.exists() {
            let mut config = Config::load(&current)?;
            apply_env_overrides(&mut config);
            return Ok(DiscoveredConfig {
                config,
                root: current.clone(),
                source: ConfigSource::Project(if toml_path.exists() {
                    toml_path
                } else {
                    legacy_path
                }),
            });
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }

    let mut config = Config::default();
    apply_env_overrides(&mut config);
    Ok(DiscoveredConfig {
        config,
        root: start_dir.to_path_buf(),
        source: ConfigSource::Default,
    })
}

fn apply_env_overrides(config: &mut Config) {
    if let Ok(dir) = std::env::var(ENV_PIR_DIRECTORY)
        && !dir.is_empty()
    {
        config.pir_dir = PathBuf::from(dir);
    }
}
