use std::ops::Range;
use std::path::PathBuf;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::config::{BaseConfig, MAN_DAY, MINUTE, ModuleConfig};

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

        Self {
            enabled: true,
            wallpaper_dir: WallpaperModule::construct_module_home(base_config.get_home_dir()),
            source_http: String::from(DEFAULT_SOURCE_HTTP),
            #[cfg(debug_assertions)]
            frequency_range: (MINUTE..2 * MINUTE),
            #[cfg(not(debug_assertions))]
            frequency_range: (MINUTE..MAN_DAY),
        }
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

    fn construct_module_home(base_home_path: &PathBuf) -> PathBuf {
        base_home_path.join(MODULE_NAME)
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
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