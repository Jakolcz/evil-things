use std::env::temp_dir;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::wallpaper::WallpaperConfig;

const APP_NAME: &str = "Evilyn";
const CONFIG_FILE_NAME: &str = "config.toml";

pub fn get_config() -> BaseConfig {
    log::debug!("Loading config...");

    let home_dir = home_dir().unwrap_or_else(|e| {
        panic!("Failed to retrieve the home directory: {}", e);
    });
    // now check for a config file

    match load_config_file(&home_dir) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error loading config file: {}", e);
            create_default_config(&home_dir)
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseConfig {
    // local_storage_base: String,
    wallpaper_config: Option<WallpaperConfig>,
    home_dir: PathBuf,
}

impl BaseConfig {
    pub fn new(home_dir: &PathBuf) -> Self {
        Self {
            home_dir: home_dir.clone(),
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

    pub fn get_wallpaper_config(&self) -> &WallpaperConfig {
        &self.wallpaper_config.as_ref().unwrap()
    }
    pub fn set_wallpaper_config(&mut self, wallpaper_config: WallpaperConfig) {
        self.wallpaper_config = Some(wallpaper_config);
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
    config.set_wallpaper_config(WallpaperConfig::new(&config));

    config
}

