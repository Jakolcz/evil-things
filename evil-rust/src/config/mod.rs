use std::env::temp_dir;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "Evilyn";
const CONFIG_FILE_NAME: &str = "config.toml";

pub fn get_config() -> Config {
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
pub struct Config {
    local_storage_base: String,
    #[serde(skip)]
    home_dir: PathBuf,
}

impl Config {
    pub fn new(local_storage_base: String, home_dir: PathBuf) -> Self {
        Config { local_storage_base, home_dir }
    }
    pub fn set_home_dir(&mut self, home_dir: &PathBuf) {
        self.home_dir = home_dir.clone();
    }
    pub fn get_home_dir(&self) -> &PathBuf {
        &self.home_dir
    }
    pub fn local_storage_base(&self) -> &String {
        &self.local_storage_base
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
fn load_config_file(home_dir: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_file_path = home_dir.clone().join(CONFIG_FILE_NAME);
    log::debug!("Config file path: {:?}", config_file_path);
    let mut config: Config;

    if config_file_path.exists() {
        log::debug!("Config file exists, loading...");
        let config_file = fs::read_to_string(config_file_path)?;
        config = toml::from_str(&config_file)?;
        config.set_home_dir(home_dir);
    } else {
        log::debug!("Config file does not exist, creating...");
        config = create_default_config(home_dir);
        let config_file = toml::to_string(&config)?;
        fs::write(config_file_path, config_file)?;
    };

    log::debug!("Final config: {:?}", config);

    Ok(config)
}

fn create_default_config(home_dir: &PathBuf) -> Config {
    Config { home_dir: home_dir.clone(), ..Default::default() }
}

