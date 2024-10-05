use std::cell::RefCell;
use std::collections::HashMap;
use std::env::temp_dir;
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::ops::Add;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

const APP_NAME: &str = "Evilyn";
const CONFIG_FILE_NAME: &str = "config";
pub const MODULE_NAMES: [&str; 1] = [crate::wallpaper::MODULE_NAME];

pub const SECOND: u32 = 1;
pub const MINUTE: u32 = 60 * SECOND;
pub const HOUR: u32 = 60 * MINUTE;
pub const MAN_DAY: u32 = 8 * HOUR;
pub const DAY: u32 = 24 * HOUR;
pub const WEEK: u32 = 7 * DAY;

pub const ANNOYANCE_LEVEL_INCREASE_INTERVAL: u32 = 3 * WEEK;

pub fn get_base_config() -> BaseConfig {
    log::debug!("Loading config...");

    let home_dir = home_dir().unwrap_or_else(|e| {
        panic!("Failed to retrieve the home directory: {}", e);
    });
    // now check for a config file

    load_base_config(&home_dir)
}

pub trait ModuleConfig
where
    Self: Debug + Serialize + DeserializeOwned + Sized,
{
    fn new(base_config: Rc<RefCell<BaseConfig>>) -> Self;
    fn refresh_base_config(&mut self, base_config: &BaseConfig);
    fn get_module_name(&self) -> &str;
    fn get_module_home(&self) -> &PathBuf;
    fn get_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);

    fn persist(&self) {
        save_module_config(self).unwrap_or_else(|e| {
            log::error!("Error saving config file: {}", e);
        });
    }
    fn construct_module_home(base_home_path: &PathBuf) -> PathBuf;
    fn config_exists(home: &PathBuf, name: &str) -> bool {
        let config_file_path = home.join(format!("{}.toml", name));
        config_file_path.exists()
    }
}

#[derive(Debug, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
pub struct BaseConfig {
    home_dir: PathBuf,
    main_loop_sleep: u64,
    module_statuses: HashMap<String, bool>,
    annoyance_level: u8,
    next_annoyance_level_increase: SystemTime,
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            home_dir: PathBuf::new(),
            main_loop_sleep: SECOND as u64,
            module_statuses: HashMap::new(),
            annoyance_level: 1,
            next_annoyance_level_increase: SystemTime::now().add(std::time::Duration::from_secs(ANNOYANCE_LEVEL_INCREASE_INTERVAL as u64)),
        }
    }
}

impl BaseConfig {
    pub fn new(home_dir: &PathBuf) -> Self {
        Self {
            home_dir: home_dir.clone(),
            main_loop_sleep: SECOND as u64,
            module_statuses: HashMap::from([(String::from(crate::wallpaper::MODULE_NAME), true)]),
            annoyance_level: 1,
            next_annoyance_level_increase: SystemTime::now().add(std::time::Duration::from_secs(ANNOYANCE_LEVEL_INCREASE_INTERVAL as u64)),
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

    pub fn is_module_enabled(&self, module_name: &str) -> bool {
        self.module_statuses.get(module_name).unwrap_or(&false).clone()
    }

    pub fn set_module_enabled(&mut self, module_name: &str, enabled: bool) {
        self.module_statuses.insert(String::from(module_name), enabled);
        self.persist();
    }

    pub fn get_annoyance_level(&self) -> u8 {
        self.annoyance_level
    }

    pub fn get_next_annoyance_level_increase(&self) -> SystemTime {
        self.next_annoyance_level_increase
    }

    pub fn set_next_annoyance_level_increase(&mut self, next_annoyance_level_increase: SystemTime) {
        self.next_annoyance_level_increase = next_annoyance_level_increase;
        self.persist();
    }

    /// Increases the annoyance level by one, sets the next increase time to the current time plus the interval and persists the changes.
    /// # Returns
    ///    u8 - The new annoyance level.
    pub fn increase_annoyance_level(&mut self) -> u8 {
        self.annoyance_level += 1;
        self.next_annoyance_level_increase = SystemTime::now().add(std::time::Duration::from_secs(ANNOYANCE_LEVEL_INCREASE_INTERVAL as u64));
        self.persist();
        self.annoyance_level
    }

    pub fn set_annoyance_level(&mut self, annoyance_level: u8) {
        self.annoyance_level = annoyance_level;
        self.persist();
    }

    fn persist(&self) {
        save_base_config(self).unwrap_or_else(|e| {
            log::error!("Error saving config file: {}", e);
        });
    }
}

pub fn save_module_config<T: ModuleConfig>(config: &T) -> Result<(), Box<dyn Error>> {
    log::debug!("Saving module '{}' config...", config.get_module_name());
    do_save(config, config.get_module_home(), config.get_module_name())
}

pub fn load_config<T: DeserializeOwned>(folder: &PathBuf, module_name: &str) -> Result<T, Box<dyn Error>> {
    log::debug!("Loading module '{}' config...", module_name);
    let config_file_path = folder.join(format!("{}.toml", module_name));
    log::debug!("Config file path: {:?}", config_file_path);
    let config: T;

    if config_file_path.exists() {
        log::debug!("Config file exists, loading...");
        let config_file = fs::read_to_string(config_file_path)?;
        // TODO maybe unwrap_or_else, remove the broken file and replace it with a new one?
        config = toml::from_str(&config_file)?;
    } else {
        return Err(Box::from(format!("Config file does not exist: {:?}", config_file_path)));
    };

    Ok(config)
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
fn load_base_config(home_dir: &PathBuf) -> BaseConfig {
    load_config(home_dir, CONFIG_FILE_NAME).unwrap_or_else(|_| {
        // silently ignore errors and replace it with None
        log::debug!("Config file does not exist, creating...");
        let config = BaseConfig::new(home_dir);
        save_base_config(&config).unwrap_or_else(|e| {
            log::error!("Error saving config file: {}", e);
        });
        config
    })
}

fn save_base_config(config: &BaseConfig) -> Result<(), Box<dyn Error>> {
    log::debug!("Saving base config...");
    do_save(config, config.get_home_dir(), CONFIG_FILE_NAME)
}


fn do_save<T: Serialize>(file: &T, folder: &PathBuf, file_name: &str) -> Result<(), Box<dyn Error>> {
    log::debug!("Doing the actual save...");
    fs::create_dir_all(folder)?; // ensure the folder exists
    let config_file_path = folder.join(format!("{}.toml", file_name));
    let config_file = toml::to_string(file)?;
    fs::write(config_file_path, config_file)?;
    Ok(())
}
