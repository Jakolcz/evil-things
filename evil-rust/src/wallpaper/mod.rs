use std::ops::Range;
use std::path::PathBuf;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::config::{BaseConfig, load_config, MAN_DAY, MINUTE, ModuleConfig, save_module_config};

pub const MODULE_NAME: &str = "wallpaper";
const DEFAULT_SOURCE_HTTP: &str = "https://source.unsplash.com/random/1920x1080";

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperModule {
    enabled: bool,
    wallpaper_dir: PathBuf,
    source_http: String,
    frequency_range: Range<u32>,
}

impl ModuleConfig for WallpaperModule {
    fn new(base_config: &BaseConfig) -> Self {
        let module_home = WallpaperModule::construct_module_home(base_config.get_home_dir());
        load_config(&module_home, MODULE_NAME).unwrap_or_else(|_| {
            let default = Self {
                enabled: true,
                wallpaper_dir: module_home,
                source_http: String::from(DEFAULT_SOURCE_HTTP),
                #[cfg(debug_assertions)]
                frequency_range: (MINUTE..2 * MINUTE),
                #[cfg(not(debug_assertions))]
                frequency_range: (MINUTE..MAN_DAY),
            };

            save_module_config(&default).unwrap_or_else(|e| {
                log::error!("Error saving config file: {}", e);
            });

            default
        })
    }

    fn refresh_base_config(&mut self, base_config: &BaseConfig) {
        self.wallpaper_dir = WallpaperModule::construct_module_home(base_config.get_home_dir());
    }

    fn get_module_name(&self) -> &str {
        MODULE_NAME
    }

    fn get_module_home(&self) -> &PathBuf {
        &self.wallpaper_dir
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn construct_module_home(base_home_path: &PathBuf) -> PathBuf {
        base_home_path.join(MODULE_NAME)
    }
}

impl WallpaperModule {
    pub fn get_next_frequency(&self) -> u32 {
        rand::thread_rng().gen_range(self.frequency_range.clone())
    }

    pub fn get_frequency_range(&self) -> Range<u32> {
        self.frequency_range.clone()
    }
}