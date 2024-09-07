mod tampering;

use std::collections::HashMap;
use std::error::Error;
use std::ops::Add;
use std::path::PathBuf;
use std::time::SystemTime;
use clipboard::{ClipboardContext, ClipboardProvider};
use tampering::{ClipboardTampering, get_tampering_functions};
use serde::{Deserialize, Serialize};
use crate::config::{load_config, save_module_config, BaseConfig, ModuleConfig};
use crate::module::Module;

pub const MODULE_NAME: &str = "clipboard";

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardModule {
    enabled: bool,
    read_content: bool,
    write_content: bool,
    home_dir: PathBuf,
    next_tampering_trigger: SystemTime,
    #[serde(skip)]
    tampering_functions: HashMap<String, ClipboardTampering>,
}

impl ModuleConfig for ClipboardModule {
    fn new(base_config: &BaseConfig) -> Self {
        let module_home = ClipboardModule::construct_module_home(base_config.get_home_dir());

        load_config(&module_home, MODULE_NAME).unwrap_or_else(|_| {
            let default = Self {
                enabled: true,
                read_content: true,
                write_content: true,
                home_dir: module_home,
                next_tampering_trigger: SystemTime::now(),
                tampering_functions: get_tampering_functions(),
            };

            save_module_config(&default).unwrap_or_else(|e| {
                log::error!("Error saving config file: {}", e);
            });

            default
        })
    }

    fn refresh_base_config(&mut self, base_config: &BaseConfig) {
        self.home_dir = ClipboardModule::construct_module_home(base_config.get_home_dir());
        self.persist();
    }

    fn get_module_name(&self) -> &str {
        MODULE_NAME
    }

    fn get_module_home(&self) -> &PathBuf {
        &self.home_dir
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.persist();
    }

    fn construct_module_home(base_home_path: &PathBuf) -> PathBuf {
        base_home_path.join(MODULE_NAME)
    }
}

impl Module for ClipboardModule {
    fn trigger(&mut self) {
        if !self.enabled {
            return;
        }

        self.tamper_with_clipboard();
    }
}
impl ClipboardModule {
    pub fn get_clipboard_content(&self) -> Result<String, Box<dyn Error>> {
        let mut clipboard_ctx: ClipboardContext = match ClipboardProvider::new() {
            Ok(ctx) => ctx,
            Err(e) => return Err(e)
        };

        match clipboard_ctx.get_contents() {
            Ok(content) => Ok(content),
            Err(e) => Err(e)
        }
    }

    pub fn set_clipboard_content(&self, content: String) -> Result<(), Box<dyn Error>> {
        let mut clipboard_ctx: ClipboardContext = match ClipboardProvider::new() {
            Ok(ctx) => ctx,
            Err(e) => return Err(e)
        };

        match clipboard_ctx.set_contents(content) {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    // TODO add functions to change the tampering settings

    pub fn tamper_with_clipboard(&mut self) {
        let mut clipboard_ctx: ClipboardContext = match ClipboardProvider::new() {
            Ok(ctx) => ctx,
            Err(e) => {
                log::error!("Error getting clipboard context: {}", e);
                return;
            }
        };

        let mut content = match clipboard_ctx.get_contents() {
            Ok(content) => content,
            Err(e) => {
                log::error!("Error getting clipboard content: {}", e);
                return;
            }
        };

        let now = SystemTime::now();
        for tampering in self.tampering_functions.values_mut() {
            if !tampering.enabled {
                continue;
            }

            let probability = rand::random::<f32>();        // 0.0 <= probability < 1.0
            // TODO make the probability threshold configurable
            if probability < 0.7 {      // 70% chance to skip tampering
                continue;
            }
            if tampering.trigger.lt(&now) {
                (tampering.tamper)(&mut content);
                tampering.trigger = now.add(std::time::Duration::from_secs(tampering.cooldown as u64));
            }
        }
    }
}