use serde::{Deserialize, Serialize};
use std::fs;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrConfig {
    pub capture_width: u32,
    pub capture_height: u32,
    pub capture_x: i32,
    pub capture_y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HotkeysConfig {
    pub toggle_box: String,
    pub toggle_action: String,
    pub exit_app: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub enable_overlay: bool,
    pub status_x: i32,
    pub status_y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ocr: OcrConfig,
    pub hotkeys: HotkeysConfig,
    pub ui: UiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ocr: OcrConfig {
                capture_width: 1162,
                capture_height: 586,
                capture_x: 122,
                capture_y: 40,
            },
            hotkeys: HotkeysConfig {
                toggle_box: "F3".to_string(),
                toggle_action: "F4".to_string(),
                exit_app: "F5".to_string(),
            },
            ui: UiConfig {
                enable_overlay: true,
                status_x: 85,
                status_y: 1,
            },
        }
    }
}

pub fn load_config() -> AppConfig {
    if let Ok(content) = fs::read_to_string("auto_shake.toml") {
        if let Ok(config) = toml::from_str(&content) {
            return config;
        }
    }
    let default = AppConfig::default();
    save_config(&default);
    default
}

pub fn save_config(config: &AppConfig) {
    if let Ok(toml_str) = toml::to_string_pretty(config) {
        let _ = fs::write("auto_shake.toml", toml_str);
    }
}
