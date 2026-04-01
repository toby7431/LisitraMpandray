use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Server,
    Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mode: AppMode,
    pub server_ip: String,
    pub server_port: u16,
}

impl AppConfig {
    pub fn server_url(&self) -> String {
        format!("http://{}:{}", self.server_ip, self.server_port)
    }
}

pub fn config_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("config.json")
}

pub fn load_config(app_data_dir: &PathBuf) -> Option<AppConfig> {
    let content = std::fs::read_to_string(config_path(app_data_dir)).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_config_to_disk(app_data_dir: &PathBuf, config: &AppConfig) -> Result<(), String> {
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(app_data_dir), content).map_err(|e| e.to_string())
}
