use crate::app::{AppConfig, Destination};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    destinations: Vec<DestinationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DestinationConfig {
    name: String,
    path: String,
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config_file: ConfigFile = toml::from_str(&content)?;

    let destinations = config_file
        .destinations
        .into_iter()
        .map(|d| Destination {
            name: d.name,
            path: d.path,
        })
        .collect();

    Ok(AppConfig { destinations })
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;
    
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_file = ConfigFile {
        destinations: config
            .destinations
            .iter()
            .map(|d| DestinationConfig {
                name: d.name.clone(),
                path: d.path.clone(),
            })
            .collect(),
    };

    let content = toml::to_string_pretty(&config_file)?;
    fs::write(&config_path, content)?;

    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    if let Some(config_dir) = directories::ProjectDirs::from("", "", "scow") {
        Ok(config_dir.config_dir().join("config.toml"))
    } else {
        Ok(PathBuf::from(".scow.toml"))
    }
}

pub fn create_default_config() -> Result<()> {
    let config = AppConfig::default();
    save_config(&config)
}
