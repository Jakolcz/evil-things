use std::collections::HashMap;
use std::env::temp_dir;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::wallpaper::WallpaperConfig;

const APP_NAME: &str = "Evilyn";
const CONFIG_FILE_NAME: &str = "config.toml";
pub const MODULE_NAMES: [&str; 1] = ["wallpaper"];

pub const SECOND: u32 = 1;
pub const MINUTE: u32 = 60 * SECOND;
pub const HOUR: u32 = 60 * MINUTE;
pub const MAN_DAY: u32 = 8 * HOUR;
pub const DAY: u32 = 24 * HOUR;
pub const WEEK: u32 = 7 * DAY;

pub fn get_base_config() -> BaseConfig {
    log::debug!("Loading config...");

    let home_dir = home_dir().unwrap_or_else(|e| {
        panic!("Failed to retrieve the home directory: {}", e);
    });
    // now check for a config file

    load_config_file(&home_dir).unwrap_or_else(|e| {
        log::error!("Error loading config file: {}", e);
        create_default_config(&home_dir)
    })
}

pub trait ModuleConfig {
    fn new(base_config: &BaseConfig) -> Self;
    fn refresh_base_config(&mut self, base_config: &BaseConfig);
    fn get_module_name(&self) -> &str;
    fn get_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}

#[derive(Debug, Default, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
pub struct BaseConfig {
    home_dir: PathBuf,
    main_loop_sleep: u64,
    module_statuses: HashMap<String, bool>,
}

impl BaseConfig {
    pub fn new(home_dir: &PathBuf) -> Self {
        Self {
            home_dir: home_dir.clone(),
            main_loop_sleep: SECOND as u64,
            module_statuses: HashMap::from([(String::from("wallpaper"), true)]),
            ..Default::default()
        }
    }

    pub fn set_home_dir(&mut self, home_dir: &PathBuf) {
        self.home_dir = home_dir.clone();
        self.persist();
    }
    pub fn get_home_dir(&self) -> &PathBuf {
        &self.home_dir
    }

    pub fn get_main_loop_sleep(&self) -> u64 {
        self.main_loop_sleep
    }

    pub fn get_module_statuses(&self) -> &HashMap<String, bool> {
        &self.module_statuses
    }

    pub fn is_module_enabled(&self) -> bool {
        self.module_statuses.get("wallpaper").unwrap_or(&false).clone()
    }

    pub fn set_module_enabled(&mut self, module_name: &str, enabled: bool) {
        self.module_statuses.insert(String::from(module_name), enabled);
        self.persist();
    }

    fn persist(&self) {
        save_config_file(self).unwrap_or_else(|e| {
            log::error!("Error saving config file: {}", e);
        });
    }
}

/// Returns the path to the home directory for the application, ensures the directory exists.
///
/// # Returns
///     PathBuf - The path to the home directory for the application.
fn home_dir() -> Result<PathBuf, Box<dyn Error>> {
    let the_path = temp_dir().join(APP_NAME);
    log::debug!("Home dir: {:?}", the_path);

    fs::create_dir_all(&the_path)?;
    Ok(the_path)
}

/// Loads the config file from the home directory. In case the file does not exist, it will be created and the default values will be used.
/// # Returns
///    Config - The loaded config.
fn load_config_file(home_dir: &PathBuf) -> Result<BaseConfig, Box<dyn Error>> {
    let config_file_path = home_dir.clone().join(CONFIG_FILE_NAME);
    log::debug!("Config file path: {:?}", config_file_path);
    let mut config: BaseConfig;

    if config_file_path.exists() {
        log::debug!("Config file exists, loading...");
        let config_file = fs::read_to_string(config_file_path)?;
        // TODO maybe unwrap_or_else, remove the broken file and replace it with a new one?
        config = toml::from_str(&config_file)?;
        config.set_home_dir(home_dir);
    } else {
        log::debug!("Config file does not exist, creating...");
        config = create_default_config(home_dir);
        save_config_file(&config)?;
    };

    log::debug!("Final config: {:?}", config);

    Ok(config)
}

fn save_config_file(config: &BaseConfig) -> Result<(), Box<dyn Error>> {
    log::debug!("Saving config...");
    let config_file_path = config.get_home_dir().join(CONFIG_FILE_NAME);
    let config_file = toml::to_string(&config)?;
    fs::write(config_file_path, config_file)?;
    Ok(())
}

fn create_default_config(home_dir: &PathBuf) -> BaseConfig {
    let mut config = BaseConfig::new(home_dir);
    let wallpaper_config = WallpaperConfig::new(&config);
    // config.set_wallpaper_config(WallpaperConfig::new(&config));

    config
}

