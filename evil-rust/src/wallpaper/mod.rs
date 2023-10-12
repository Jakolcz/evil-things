use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::config::BaseConfig;

const MODULE_NAME: &str = "wallpaper";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WallpaperConfig {
    enabled: bool,
    wallpaper_dir: PathBuf,
    source_http: String,
    frequency: u32,
}

impl WallpaperConfig {
    pub fn new(base_config: &BaseConfig) -> Self {
        Self {
            enabled: true,
            wallpaper_dir: base_config.get_home_dir().join(MODULE_NAME),
            frequency: 3600,
            ..Default::default()
        }
    }
}